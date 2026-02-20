use bitcoin::{OutPoint, Txid};
use pathfinder::blockchain::{BlockchainDataSource, CachingDataSource, EsploraClient, Result};
use std::{
    str::FromStr,
    time::{Duration, Instant},
};

pub async fn mempool_esplora(_outpoint: OutPoint) -> Result<Option<bitcoin::Transaction>> {
    todo!()
}

#[tokio::main]
async fn main() -> Result<()> {
    let txid =
        Txid::from_str("15e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8521").unwrap();

    let outpoint = OutPoint::new(txid, 3);

    let client = EsploraClient::new("https://mempool.space/api".to_string());
    let cache = CachingDataSource::new(client, Duration::from_secs(300));

    println!("=== Testing get_transaction caching ===\n");

    // First fetch (cache miss)
    let start = Instant::now();
    cache.get_transaction(txid).await?;
    println!("UNCACHED: {} ms", start.elapsed().as_millis());

    // Second fetch (cache hit)
    let start = Instant::now();
    cache.get_transaction(txid).await?;
    println!("CACHED:   {} ms", start.elapsed().as_millis());

    println!("\n=== Testing get_spending_transaction caching ===\n");

    // First fetch (cache miss)
    let start = Instant::now();
    let spending = cache.get_spending_transaction(outpoint).await?;
    println!("UNCACHED: {} ms", start.elapsed().as_millis());

    // Second fetch (cache hit)
    let start = Instant::now();
    cache.get_spending_transaction(outpoint).await?;
    println!("CACHED:   {} ms", start.elapsed().as_millis());
    match spending {
        Some(tx) => println!("\nOutput spent by: {}", tx.compute_txid()),
        None => println!("\nOutput is unspent"),
    }
    println!("\n--- Testing TTL expiration ---");

    let short_ttl_cache = CachingDataSource::new(
        EsploraClient::new("https://mempool.space/api/".to_string()),
        Duration::from_secs(2), // 2-second TTL
    );

    println!("Fetching tx (will cache)...");
    short_ttl_cache.get_transaction(txid).await.unwrap();

    println!("Fetching again immediately (should be cached)...");
    let now = Instant::now();
    short_ttl_cache.get_transaction(txid).await.unwrap();
    println!("Cached: {} ms", now.elapsed().as_millis());

    println!("Waiting 3 seconds for TTL expiration...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    println!("Fetching again after expiration (should re-fetch)...");
    let now = Instant::now();
    short_ttl_cache.get_transaction(txid).await.unwrap();
    println!("Re-fetched: {} ms", now.elapsed().as_millis());

    Ok(())
}
