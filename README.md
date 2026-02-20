# pathfinder
A Bitcoin UTXO tracing tool for the PEOPLE

## Status

**V0.1 - In Progress**

- [x] Trait-based blockchain data source abstraction
- [x] Bitcoin Core RPC client (basic transaction fetching)
- [x] Esplora API client (with spending transaction lookup)
- [x] TTL-based caching layer (decorator pattern)
- [] UTXO tracing engine (next up)

## Architecture

Library-first design with swappable backends:
- `BlockchainDataSource` trait - async interface for blockchain queries
- `EsploraClient` - mempool.space API integration with rate limiting
- `CachingDataSource<C>` - generic caching wrapper (300s TTL default)

## Usage

```rust
use pathfinder::blockchain::{EsploraClient, CachingDataSource};
use std::time::Duration;

let client = EsploraClient::new("https://mempool.space/api".to_string());
let cached = CachingDataSource::new(client, Duration::from_secs(300));

// All BlockchainDataSource methods now use cache
let tx = cached.get_transaction(txid).await?;

Testing

cargo run  # Run test harness in main.rs
cargo test # Run unit tests

