use bitcoin::{OutPoint, Transaction};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use async_trait::async_trait;

use crate::blockchain::{BlockchainDataSource, BlockchainError, Result};

pub struct CachedValue {}

pub struct CachingDataSource<C: BlockchainDataSource> {
    inner: C,
    cache: Arc<RwLock<HashMap<String, CachedValue>>>,
    ttl: Duration,
}

#[async_trait]
impl<C: BlockchainDataSource + std::marker::Sync> BlockchainDataSource for CachingDataSource<C> {
    async fn get_transaction(&self, _txid: bitcoin::Txid) -> Result<Transaction> {
        todo!();
    }
    async fn get_spending_transaction(
        &self,
        outpoint: bitcoin::OutPoint,
    ) -> Result<Option<bitcoin::Transaction>> {
        todo!()
    }
    async fn get_address_transactions(
        &self,
        address: bitcoin::Address,
    ) -> Result<Vec<bitcoin::Transaction>> {
        todo!()
    }
    async fn get_transactions_batch(
        &self,
        txids: &[bitcoin::Txid],
    ) -> Result<Vec<Option<bitcoin::Transaction>>> {
        todo!()
    }
    async fn get_spending_transactions_batch(
        &self,
        outpoints: &[bitcoin::OutPoint],
    ) -> Result<Vec<Option<bitcoin::Transaction>>> {
        todo!()
    }
}
