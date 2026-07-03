//! Yellowstone gRPC implementation of the [`Feed`] contract.
//!
//! Connects to a Geyser gRPC endpoint, opens a subscription for slots +
//! transactions, and maps each `SubscribeUpdate` into our transport-free
//! [`FeedEvent`]. This is the concrete "how" behind `og-core`'s "what".

use std::collections::HashMap;
use std::pin::Pin;

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use og_core::{Feed, FeedEvent};
use tonic::Status;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::geyser::{
    SubscribeRequest, SubscribeRequestFilterSlots, SubscribeRequestFilterTransactions,
    SubscribeUpdate, subscribe_update::UpdateOneof,
};

/// Boxed stream of updates coming back from the subscription. Boxed because
/// `subscribe_once` returns an opaque `impl Stream` we need to store in a field.
type UpdateStream = Pin<Box<dyn Stream<Item = Result<SubscribeUpdate, Status>> + Send>>;

/// A live Yellowstone gRPC feed.
///
/// We keep only the update `stream`. The client/`Channel` is intentionally
/// dropped after subscribing: the returned `Streaming` owns what it needs to
/// keep receiving (the same pattern the reference `sol-indexer` runs in prod).
pub struct YellowstoneFeed {
    stream: UpdateStream,
}

impl YellowstoneFeed {
    /// Connect to `endpoint` (e.g. `http://127.0.0.1:10000`) and subscribe.
    ///
    /// `x_token` is the auth header some hosted providers require; `None` for
    /// our local test-validator. We subscribe to **slots** (the heartbeat on an
    /// empty chain) and **transactions** (skipping vote + failed txs).
    pub async fn connect(endpoint: String, x_token: Option<String>) -> anyhow::Result<Self> {
        let mut client = GeyserGrpcClient::build_from_shared(endpoint)?
            .x_token(x_token)?
            .connect()
            .await?;

        // Server-side filters. The map key is just a client-chosen filter label.
        let mut slots = HashMap::new();
        slots.insert("client".to_owned(), SubscribeRequestFilterSlots::default());

        let mut transactions = HashMap::new();
        transactions.insert(
            "client".to_owned(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                ..Default::default()
            },
        );

        let request = SubscribeRequest {
            slots,
            transactions,
            ..Default::default()
        };

        let stream = client.subscribe_once(request).await?;
        Ok(Self {
            stream: Box::pin(stream),
        })
    }
}

#[async_trait]
impl Feed for YellowstoneFeed {
    async fn next_event(&mut self) -> anyhow::Result<Option<FeedEvent>> {
        // Loop past updates we don't surface yet (pings, block meta, …) until we
        // get a slot/transaction or the stream ends.
        while let Some(update) = self.stream.next().await {
            let update = update?;
            match update.update_oneof {
                Some(UpdateOneof::Slot(slot)) => {
                    return Ok(Some(FeedEvent::Slot { slot: slot.slot }));
                }
                Some(UpdateOneof::Transaction(tx)) => {
                    let slot = tx.slot;
                    if let Some(info) = tx.transaction {
                        let signature = bs58::encode(info.signature).into_string();
                        return Ok(Some(FeedEvent::Transaction { signature, slot }));
                    }
                }
                _ => continue,
            }
        }
        Ok(None)
    }
}
