//! Caching layer for Blockchain data sources
//!
//! Provides generic `CachingDataSource<C>` decorator that wraps any `BlockchainDataSource`
//! implementation with TTL-based in memory caching.
//!
//! Critical for performance when handling large traces where paths converge.

use crate::blockchain::{BlockchainDataSource, Result};
use async_trait::async_trait;
use bitcoin::{Address, OutPoint, Transaction, Txid};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::time::Instant;

/// Cache key type distinguishing between transaction lookups and spending lookups
///
/// # Fields
///
/// * `Transaction(Txid)` - Direct Transaction lookup with
/// * `Spending(OutPoint)` - Spending Tx lookup by outpoint (which tx spent this output?)
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum CacheKey {
    Transaction(Txid),
    Spending(OutPoint),
}

/// A cached transaction entry with insertion timestamp for TTL checking.
///
/// # Fields
/// * `transaction` - a cached bitcoin::Transaction
/// * `inserted_at` - timestamp for TTL cechking
#[derive(Debug, Clone)]
pub struct CachedEntry {
    transaction: Transaction,
    inserted_at: Instant,
}

/// Decorator that adds TTL-based caching to any `BlockchainDataSource`.
///
/// Uses `Arc<RwLock<HashMap>>` for thread-safe concurrent access:
/// - Read locks for cache lookups (allows concurrent reads)
/// - Write locks for cache inserts (exclusive)
///
/// # Example
/// ```ignore
/// let esplora = EsploraClient::new("https://mempool.space/api".to_string());
/// let cached = CachingDataSource::new(esplora, Duration::from_secs(300));
/// ```
pub struct CachingDataSource<C> {
    /// Inner data source (Esplora, Bitcoin Core RPC, etc.)
    inner: C,
    /// Thread-safe cache with TTL eviction
    cache: Arc<RwLock<HashMap<CacheKey, CachedEntry>>>,
    /// Time to live for cache entries
    ttl: Duration,
}

impl<C> CachingDataSource<C> {
    /// Creates a new caching wrapper around the given data source.
    ///
    /// # Arguments
    /// * `inner` - The underlying blockchain data source
    /// * `ttl` - How long cached entries remain valid
    pub fn new(inner: C, ttl: Duration) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }
}

#[async_trait]
impl<C: BlockchainDataSource + std::marker::Sync> BlockchainDataSource for CachingDataSource<C> {
    /// Fetches a transaction by txid, checking cache first.
    ///
    /// Cache strategy:
    /// 1. Check cache with read lock
    /// 2. If hit and not expired, return cached tx
    /// 3. If miss or expired, fetch from inner source
    /// 4. Store result in cache with write lock
    async fn get_transaction(&self, txid: Txid) -> Result<Transaction> {
        let key = CacheKey::Transaction(txid);

        // Check the cache (read lock)
        {
            let cache = self.cache.read().unwrap();
            if let Some(entry) = cache.get(&key)
                && entry.inserted_at.elapsed() < self.ttl
            {
                return Ok(entry.transaction.clone());
            }
            // Entry expired, fetch it
        }

        // cache miss or expired, fetch Transaction from source
        let tx = self.inner.get_transaction(txid).await?;

        // Store the fetched Tx into cache (write lock)
        {
            let mut cache = self.cache.write().unwrap();
            let entry = CachedEntry {
                transaction: tx.clone(),
                inserted_at: Instant::now(),
            };
            cache.insert(key, entry);
        }

        Ok(tx)
    }

    /// Fetches the transaction that spent the given outpoint, checking cache first.
    ///
    /// Returns `None` if the output is unspent. Unspent outputs are NOT cached
    /// NOTE:(they may be spent between checks).
    async fn get_spending_transaction(&self, outpoint: OutPoint) -> Result<Option<Transaction>> {
        let key = CacheKey::Spending(outpoint);

        // check the cache (read lock)
        {
            let cache = self.cache.read().unwrap();
            if let Some(entry) = cache.get(&key)
                && entry.inserted_at.elapsed() < self.ttl
            {
                return Ok(Some(entry.transaction.clone()));
            }
            // Entry expired, fetch it
        }

        // cache miss or expired, fetch Transaction from source
        let tx = self.inner.get_spending_transaction(outpoint).await?;

        // Update cache only if we got a transaction (write lock)
        // Note: None (unspent) is not cached to avoid stale data
        {
            if let Some(ref transaction) = tx {
                let mut cache = self.cache.write().unwrap();
                cache.insert(
                    key,
                    CachedEntry {
                        transaction: transaction.clone(),
                        inserted_at: Instant::now(),
                    },
                );
            } // Write lock released (or skipped if None)
        }
        Ok(tx)
    }
    async fn get_address_transactions(&self, _address: Address) -> Result<Vec<Transaction>> {
        todo!()
    }
    async fn get_transactions_batch(&self, _txids: &[Txid]) -> Result<Vec<Option<Transaction>>> {
        todo!()
    }
    async fn get_spending_transactions_batch(
        &self,
        _outpoints: &[OutPoint],
    ) -> Result<Vec<Option<Transaction>>> {
        todo!()
    }
}
