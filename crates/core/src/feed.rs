//! The ingestion boundary.
//!
//! A [`Feed`] is *anything that yields on-chain events*. `og-core` defines the
//! contract; concrete sources (Yellowstone gRPC now, a file replay later) live
//! in the `indexer` crate and implement this trait. Keeping the contract here —
//! free of any gRPC / transport types — is what lets the pipeline depend on
//! "a feed" without caring where the bytes come from (locked decision §5).

use async_trait::async_trait;

/// A single event pulled off a chain feed.
///
/// Phase 1B keeps this deliberately minimal — just enough to *prove the stream*
/// end-to-end. On an empty local `solana-test-validator` the only thing that
/// reliably ticks is slots, so [`FeedEvent::Slot`] is the heartbeat we watch;
/// [`FeedEvent::Transaction`] is wired for when we point `GRPC_URL` at a real
/// provider. It grows richer in Phase 2 (raw tx bytes, block_time) as the
/// pipeline needs more — nullable-by-design, no throwaway (§7).
#[derive(Debug, Clone)]
pub enum FeedEvent {
    /// A new slot was produced.
    Slot { slot: u64 },
    /// A transaction landed. `signature` is base58 (human-readable); decoding
    /// the payload into a normalized swap is deferred to Phase 5.
    Transaction { signature: String, slot: u64 },
}

/// A pull-based source of [`FeedEvent`]s.
///
/// Pull-based (rather than returning a `Stream`) so it drops straight into the
/// Phase 2 fetcher loop and stays trivially swappable behind `dyn Feed` between
/// the real gRPC feed and a test source. `#[async_trait]` because `async fn` in
/// traits is not yet object-safe for `dyn Feed`.
#[async_trait]
pub trait Feed: Send {
    /// Returns the next event, `Ok(None)` when the stream ends cleanly, or an
    /// error on transport failure. Callers loop on this.
    async fn next_event(&mut self) -> anyhow::Result<Option<FeedEvent>>;
}
