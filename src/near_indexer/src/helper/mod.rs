pub mod aurora;
pub mod near;

pub mod twitter;

pub mod cktls;
pub mod indexer;
pub mod proof;

use std::collections::HashMap;

use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use serde::{Deserialize, Serialize};
use std::path::Path;

use near_crypto::SecretKey;
use near_primitives::types::AccountId;
use once_cell::sync::Lazy;
use std::{env, fs};
use verity_verify_remote::ic::{DEFAULT_IC_GATEWAY_LOCAL, DEFAULT_IC_GATEWAY_MAINNET};

#[derive(Debug, Clone)]
pub struct Settings {
    pub near_rpc_url: String,
    pub signer_account_id: AccountId,
    pub signer_secret_key: SecretKey,
    pub contract_account_id: AccountId,
    pub tweet_bearer: String,
    pub rv_identity_file: String,
    pub is_production: bool,
    pub verity_ic_id: &'static str,
    pub verity_ic_gateway: String,
    pub nft_contract: String,
}

// Read ALL env-vars exactly once at startup
impl Settings {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let near_rpc_url = env::var("NEAR_RPC_URL")?;
        let signer_account_id: AccountId = env::var("NEAR_SIGNER_ACCOUNT_ID")?.parse()?;
        let signer_secret_key: SecretKey = env::var("NEAR_ACCOUNT_SECRET_KEY")?.parse()?;
        let contract_account_id: AccountId =
            env::var("NEAR_VERIFIER_CONTRACT_ACCOUNT_ID")?.parse()?;

        let tweet_bearer = env::var("TWEET_BEARER")?;

        // Load RV identity file path from env and verify existence
        let rv_identity_file = env::var("RV_IDENTITY_FILE")?;
        let path = Path::new(&rv_identity_file);
        if !path.exists() {
            return Err(format!("RV identity file not found at: {}", rv_identity_file).into());
        }
        // Ensure the file starts with the expected EC PRIVATE KEY header
        let content = fs::read_to_string(&rv_identity_file)?;
        if !content
            .trim_start()
            .starts_with("-----BEGIN EC PRIVATE KEY-----")
        {
            return Err(format!(
                "RV identity file at {} does not begin with the required EC PRIVATE KEY header",
                rv_identity_file
            )
            .into());
        }

        let nft_contract = env::var("NEAR_NFT_CONTRACT_ACCOUNT_ID").unwrap_or_default();
        let is_production = true; //nft_contract.ends_with(".near");

        // choose your Verity canister ID and gateway once
        let (verity_ic_id, verity_ic_gateway) = if is_production {
            (
                "yf57k-fyaaa-aaaaj-azw2a-cai",
                DEFAULT_IC_GATEWAY_MAINNET.to_string(),
            )
        } else {
            (
                "bkyz2-fmaaa-aaaaa-qaaaq-cai",
                DEFAULT_IC_GATEWAY_LOCAL.to_string(),
            )
        };

        Ok(Settings {
            near_rpc_url,
            signer_account_id,
            signer_secret_key,
            contract_account_id,
            tweet_bearer,
            rv_identity_file,
            is_production,
            verity_ic_id,
            verity_ic_gateway,
            nft_contract,
        })
    }
}

// 3. Create a single static instance
pub static SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::from_env().unwrap());

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
