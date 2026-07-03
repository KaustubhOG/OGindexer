//! `og-core` — framework-free shared layer for OGindexer.
//!
//! It defines the *contracts* the rest of the workspace depends on (the `Feed`
//! ingestion boundary today; the `Repository` / domain models later). It knows
//! nothing about gRPC, Axum, or Postgres — concrete implementations live in the
//! `indexer` / `api` crates.

pub mod feed;

pub use feed::{Feed, FeedEvent};
