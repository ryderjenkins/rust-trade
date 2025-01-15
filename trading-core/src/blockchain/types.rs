use serde::{Deserialize, Serialize};
use sp_core::H256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub free_balance: u128,
    pub reserved_balance: u128,
}

#[derive(Debug, Clone)]
pub struct TransactionStatus {
    pub hash: H256,
    pub is_finalized: bool,
    pub block_hash: Option<H256>,
}

#[derive(Debug, Clone)]
pub enum ExtrinsicStatus {
    Future,
    Ready,
    Broadcast(Vec<String>),
    InBlock(H256),
    Finalized(H256),
    Error(String),
}