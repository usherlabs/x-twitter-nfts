pub mod aurora;
pub mod near;

pub mod twitter;

pub mod cktls;
pub mod indexer;
pub mod proof;

use std::collections::HashMap;

use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use serde::{Deserialize, Serialize};

/// Containing the details needed for verification of a proof
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZkInputParam {
    /// session header.
    pub proof: String,

    /// meta_data
    pub meta_data: AssetMetadata,
}

/// The Includes substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetMetadata {
    /// NFT image url
    pub image_url: String,

    /// Near account to be minted to
    pub owner_account_id: String,

    /// tweet Id
    pub token_id: String,
}

/// The tweet structure gotten from the API
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TweetResponse {
    /// data
    pub data: Option<Vec<TweetData>>,
    /// users info
    pub includes: Includes,

    pub errors: Option<Vec<ErrorObject>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorObject {
    value: String,
    detail: String,
    title: String,
    resource_type: String,
    parameter: String,
}
/// The data substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TweetData {
    /// date created
    pub created_at: String,
    /// id
    pub id: String,
    /// public_metrics
    pub public_metrics: PublicMetrics,
    /// edit_history_tweet_ids
    pub edit_history_tweet_ids: Option<Vec<String>>,
    /// author_id of tweet
    pub author_id: String,
    /// tweet text
    pub text: String,
}

/// The PublicMetrics substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PublicMetrics {
    /// pub retweet_count
    pub retweet_count: u32,
    /// pub reply_count
    pub reply_count: u32,
    /// pub like_count
    pub like_count: u32,
    /// pub quote_count
    pub quote_count: u32,
    /// pub bookmark_count
    pub bookmark_count: u32,
    /// pub impression_count
    pub impression_count: u32,
}

/// The Includes substructure of a metadata NFT
///
/// Containing the details about a metadata NFT
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Includes {
    /// users
    pub users: Vec<User>,
}

/// The User substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    /// pub id: String,
    pub id: String,
    /// pub username: Option<String>
    pub username: Option<String>, // Optional because it's not present in every user object
    /// pub name: String,
    pub name: String,
    /// pub created_at: String,
    pub created_at: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct NearIndexerData {
    pageProps: PageProps,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct PageProps {
    // statsDetails: StatsDetails,
    // accountDetails: AccountDetails,
    data: TransactionData,
    dataCount: DataCount,
}

#[derive(Serialize, Deserialize, Debug)]
struct StatsDetails {
    stats: Vec<Stat>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Stat {
    id: u64,
    total_supply: Option<String>,
    circulating_supply: Option<String>,
    avg_block_time: String,
    gas_price: String,
    nodes_online: u32,
    near_price: Option<String>,
    near_btc_price: Option<String>,
    market_cap: Option<String>,
    volume: Option<String>,
    high_24h: Option<String>,
    high_all: Option<String>,
    low_24h: Option<String>,
    low_all: Option<String>,
    change_24: Option<String>,
    total_txns: String,
    tps: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct AccountDetails {
    account: Vec<Account>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Account {
    amount: String,
    block_hash: String,
    block_height: u64,
    code_hash: String,
    locked: String,
    storage_paid_at: u64,
    storage_usage: u64,
    account_id: String,
    created: Created,
    deleted: Deleted,
}

#[derive(Serialize, Deserialize, Debug)]
struct Created {
    transaction_hash: Option<String>,
    block_timestamp: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Deleted {
    transaction_hash: Option<String>,
    block_timestamp: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ContractData {
    deployments: Vec<Deployment>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Deployment {
    transaction_hash: String,
    block_timestamp: u64,
    receipt_predecessor_account_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenDetails {}

#[derive(Serialize, Deserialize, Debug)]
struct NftTokenDetails {}

#[derive(Serialize, Deserialize, Debug)]
struct ParseDetails {
    contract: Vec<Contract>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Contract {
    contract: ContractInfo,
}

#[derive(Serialize, Deserialize, Debug)]
struct ContractInfo {
    method_names: Vec<String>,
    probable_interfaces: Vec<String>,
    by_method: HashMap<String, Vec<String>>,
    schema: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InventoryDetails {
    inventory: Inventory,
}

#[derive(Serialize, Deserialize, Debug)]
struct Inventory {
    fts: Vec<String>,
    nfts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionData {
    #[serde(default)]
    cursor: Option<String>,
    txns: Vec<JSONTransaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JSONTransaction {
    pub id: String,
    pub signer_account_id: String,
    pub receiver_account_id: String,
    pub transaction_hash: String,
    pub included_in_block_hash: String,
    pub block_timestamp: String,
    pub receipt_conversion_tokens_burnt: String,
    pub block: Block,
    pub actions: Option<Vec<JSONAction>>,
    pub outcomes: Outcomes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub block_height: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JSONAction {
    pub action: String,
    pub method: Option<String>,
    pub args: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionsAgg {
    pub deposit: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Outcomes {
    #[serde(default)]
    pub status: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataCount {
    pub txns: Vec<DataCountTxn>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataCountTxn {
    pub count: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LatestBlocks {
    pub blocks: Vec<Block>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tab {
    pub tab: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MintRequestData {
    pub notify: String,
    pub tweet_id: String,
    pub image_url: String,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone)]
pub struct IpfsData {
    pub IpfsHash: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NftData {
    #[allow(dead_code)]
    token_id: String,
    #[allow(dead_code)]
    owner_id: String,
    #[allow(dead_code)]
    metadata: TokenMetadata,
}
