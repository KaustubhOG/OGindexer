//! OGindexer ingestion binary.
//!
//! Phase 1B: connect to the Yellowstone gRPC feed and print what streams in.
//! No buffer, no Postgres, no decoding yet — that arrives in Phase 2 / Phase 5.

mod yellowstone;

use og_core::{Feed, FeedEvent};
use tracing_subscriber::EnvFilter;
use yellowstone::YellowstoneFeed;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let endpoint =
        std::env::var("GRPC_URL").unwrap_or_else(|_| "http://127.0.0.1:10000".to_string());
    let x_token = std::env::var("GRPC_TOKEN").ok();

    tracing::info!(%endpoint, "connecting to Yellowstone gRPC");
    let mut feed = YellowstoneFeed::connect(endpoint, x_token).await?;
    tracing::info!("subscribed — streaming events (Ctrl-C to stop)");

    loop {
        match feed.next_event().await? {
            Some(FeedEvent::Slot { slot }) => tracing::info!(slot, "slot"),
            Some(FeedEvent::Transaction { signature, slot }) => {
                tracing::info!(slot, %signature, "tx")
            }
            None => {
                tracing::warn!("stream ended");
                break;
            }
        }
    }

    Ok(())
}
