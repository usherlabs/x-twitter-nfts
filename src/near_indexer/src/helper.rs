use crate::entity::near_transaction;

use near_client::client::{ NearClient, Signer };
use near_client::crypto::Key;
use near_client::prelude::{ AccountId, Ed25519PublicKey, Ed25519SecretKey, Finality };
use regex::Regex;
use reqwest::multipart::{ Form, Part };
use reqwest::Client;
use sea_orm::ActiveValue::Set;
use sea_orm::{ DbConn, DbErr, EntityTrait };
use serde::{ Deserialize, Serialize };
use serde_json::{ json, Error as SJError, Value };
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::marker::{ Send, Sync };
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{ debug, info };

pub struct NearExplorerIndexer<'a> {
    pub page_no: u8,
    pub cursor: Option<String>,
    pub account_id: &'a str,
    pub build_id: String,
}

impl<'a> NearExplorerIndexer<'a> {
    pub fn new(account_id: &'a str) -> Result<Self, Box<dyn Error>> {
        if !(account_id.ends_with(".testnet") || account_id.ends_with(".near")) {
            return Err("Invalid account_id".into());
        }

        Ok(NearExplorerIndexer {
            cursor: None,
            page_no: 0,
            build_id: (
                if account_id.ends_with(".testnet") {
                    "0kmxltnOS1UsrfVrMd5fP"
                } else {
                    "bE43kUihJPVfWqBYXxGBQ"
                }
            ).to_owned(),
            account_id: &account_id,
        })
    }

    /// Retrieves the Latest Batch of transactions.
    ///
    /// This method continues fetching transactions from latest
    ///
    /// # Returns
    /// A vector of transactions representing the next page.
    pub async fn get_transactions(
        &mut self
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.page_no = 0;
        self.cursor = None;
        let data = self.fetch(true).await?;

        if data.cursor.is_some() {
            self.page_no = 1;
            self.cursor = data.cursor;
        }

        Ok(data.txns)
    }

    /// Retrieves the next page of transactions.
    ///
    /// This method continues fetching transactions from where the previous call left off.
    ///
    /// # Returns
    /// A vector of transactions representing the next page.
    pub async fn next_page(&mut self) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let data = self.fetch(false).await?;
        if let Some(cursor) = data.cursor {
            self.cursor = Some(cursor);
        } else {
            self.cursor = None;
        }
        self.page_no = self.page_no + 1;

        Ok(data.txns)
    }

    pub fn has_next_page(&self) -> bool {
        self.cursor.is_some()
    }
    async fn fetch(
        &mut self,
        reset: bool
    ) -> Result<TransactionData, Box<dyn Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let base = if self.account_id.ends_with(".testnet") {
            "testnet.nearblocks.io"
        } else {
            "nearblocks.io"
        };

        let max_retries = 5;
        let mut retries = 0;
        loop {
            let url = if self.page_no == 0 || reset {
                format!(
                    "https://{}/_next/data/{}/en/address/{}.json",
                    base,
                    self.build_id,
                    self.account_id
                )
            } else {
                format!(
                    "https://{}/_next/data/{}/en/address/{}.json?cursor={}&p={}",
                    base,
                    self.build_id,
                    self.account_id,
                    self.cursor.clone().unwrap_or("".to_owned()),
                    self.page_no
                )
            };
            let response = client.get(&url).send().await?;

            if (&response).status() == 404 {
                let html_text = response.text().await.unwrap();
                let re = Regex::new(r"_next/static/([a-zA-Z0-9\-_]+)/_buildManifest").unwrap();

                // Attempt to find captures in the text
                if let Some(captures) = re.captures(&html_text) {
                    // Extract the first capture group, which is our build number
                    if let Some(build_number) = captures.get(1) {
                        self.build_id = build_number.as_str().to_owned();
                    }
                }
            } else {
                match response.json::<NearIndexerData>().await {
                    Ok(output) => {
                        return Ok(output.pageProps.data);
                    }
                    Err(e) => {
                        if retries >= max_retries {
                            return Err(format!("FETCH_ERROR: {}, {}", e, &url).into());
                        }
                    }
                }
            }
            retries += 1;
            let delay = Duration::from_millis(5000);
            sleep(delay).await;
        }
    }
}

#[cfg(test)]
mod test_near_explorer_indexer {
    use super::*;

    #[tokio::test]
    async fn test_near_explorer_indexer() {
        let mut indexer = NearExplorerIndexer::new("priceoracle.near").unwrap();
        assert_eq!(indexer.get_transactions().await.unwrap().len(), 25);
        assert_eq!(indexer.next_page().await.unwrap().len(), 25);
        assert_eq!(indexer.has_next_page(), true);
    }

    #[tokio::test]
    async fn test_near_explorer_indexer_testnet() {
        let mut indexer = NearExplorerIndexer::new("ush_test.testnet").unwrap();
        let data_size = indexer.get_transactions().await.unwrap().len();
        assert!(data_size < 25);
        assert_eq!(indexer.has_next_page(), false);
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct NearIndexerData {
    pageProps: PageProps,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct PageProps {
    statsDetails: StatsDetails,
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
    cursor: Option<String>,
    txns: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    id: String,
    signer_account_id: String,
    receiver_account_id: String,
    transaction_hash: String,
    included_in_block_hash: String,
    block_timestamp: String,
    receipt_conversion_tokens_burnt: String,
    block: Block,
    actions: Vec<Action>,
    outcomes: Outcomes,
}

#[derive(Serialize, Deserialize, Debug)]
struct Block {
    block_height: u128,
}

#[derive(Serialize, Deserialize, Debug)]
struct Action {
    action: String,
    method: Option<String>,
    args: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionsAgg {
    deposit: u128,
}

#[derive(Serialize, Deserialize, Debug)]
struct Outcomes {
    status: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct DataCount {
    txns: Vec<DataCountTxn>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DataCountTxn {
    count: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LatestBlocks {
    blocks: Vec<Block>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tab {
    tab: String,
}

#[derive(Debug, Deserialize)]
struct MintRequestData {
    notify: String,
    tweet_id: u64,
    image_url: String,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct IpfsData {
    IpfsHash: String,
}

#[derive(Debug, Deserialize)]
struct TokenMetadata {
    #[allow(dead_code)]
    title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    #[allow(dead_code)]
    description: Option<String>, // free-form description
    #[allow(dead_code)]
    media: Option<String>,
    #[allow(dead_code)]
    // URL to associated media, preferably to decentralized, content-addressed storage
    copies: Option<u64>,
    #[allow(dead_code)]
    // number of copies of this set of metadata in existence when token was minted.
    issued_at: Option<String>,
    #[allow(dead_code)]
    // ISO 8601 datetime when token was issued or minted
    expires_at: Option<String>,
    #[allow(dead_code)]
    // ISO 8601 datetime when token expires
    starts_at: Option<String>,
    #[allow(dead_code)]
    // ISO 8601 datetime when token starts being valid
    updated_at: Option<String>,
    #[allow(dead_code)]
    // ISO 8601 datetime when token was last updated
    extra: Option<String>,
    #[allow(dead_code)]
    // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    reference: Option<String>, // URL to an off-chain JSON file with more info.
}

#[derive(Debug, Deserialize)]
struct NftData {
    #[allow(dead_code)]
    token_id: String,
    #[allow(dead_code)]
    owner_id: String,
    #[allow(dead_code)]
    metadata: TokenMetadata,
}

pub async fn get_proof(tweet_id: u64) -> String {
    let verity_client = get_verity_client();
    let thirdweb_client_id = env::var("THIRDWEB_CLIENT_ID").expect("MY_VAR must be set");
    let _temp = verity_client
        .get(
            format!("https://api.x.com/2/tweets?ids={}&tweet.fields=created_at,public_metrics&expansions=author_id&user.fields=created_at", tweet_id)
        )
        .header(
            "Authorization",
            format!("Bearer {}", env::var("TWEET_BEARER").unwrap_or(String::from("")))
        )
        .send().await;

    if _temp.is_err() {
        print!("Error: {:?} ", _temp.err());
        return String::from("");
    }
    let _temp = _temp.unwrap();
    let full_data =
        json!({"proof":_temp.proof,"notary_pub_key":_temp.notary_pub_key,"x":_temp.subject.json::<Value>().await.unwrap()});

    let client = Client::new();

    let form = Form::new()
        .part("file", Part::text(full_data.to_string()).file_name("proof.json"))
        .part("pinataOptions", Part::text("{\"wrapWithDirectory\":false}"))
        .part("pinataOptions", Part::text("{\"wrapWithDirectory\":false}"))
        .part("pinataMetadata", Part::text("{\"name\":\"Storage SDK\",\"keyvalues\":{}}"));

    // Return a JSON response
    let url = "https://storage.thirdweb.com/ipfs/upload";
    let response = client
        .post(url)
        .header("X-Client-Id", &thirdweb_client_id)
        .header("Content-Type", format!("multipart/form-data; boundary={}", form.boundary()))
        .multipart(form)
        .send().await;

    if response.is_err() {
        print!("Error: {:?} ", response.err());
        return String::from("");
    }

    let response = response.unwrap().json::<IpfsData>().await;

    if response.is_err() {
        print!("Error: {:?} ", response.err());
        return String::from("");
    }
    format!("ipfs://{}", response.unwrap().IpfsHash)
}

pub async fn process_near_transaction(
    db: &DbConn,
    transaction: &Transaction,
    client: &NearClient,
    sk: &Ed25519SecretKey
) -> Result<bool, DbErr> {
    let nft_contract_id = env::var("NFT_CONTRACT_ID").unwrap_or("test-usher.testnet".to_owned());
    let nft_contract_id = AccountId::from_str(&nft_contract_id).unwrap();

    let pk = transaction.id.parse::<i32>().unwrap();

    // Find by primary key
    let _near_transaction: Option<near_transaction::Model> = near_transaction::Entity
        ::find_by_id(pk)
        .one(db).await?;
    match _near_transaction {
        Some(_) => Ok(true),
        None => {
            if !transaction.outcomes.status {
                debug!(
                    "Found Failed BlockChain Transaction Ignored, {}",
                    transaction.transaction_hash
                );
                return Ok(false);
            }
            if transaction.actions[0].method.is_none() {
                debug!("Ignored Transaction: {} No method Found", transaction.transaction_hash);
                return Ok(false);
            }
            let action = match transaction.actions.get(0) {
                Some(action) => action,
                None =>
                    &(Action {
                        action: "".to_string(),
                        method: None,
                        args: None,
                    }),
            };

            if let Some(method) = &action.method {
                // LIST OPERATIONS
                if method == "mint_tweet_request" {
                    let mint_data: Result<MintRequestData, SJError> = serde_json::from_str(
                        action.args.clone().unwrap_or("".to_string()).as_str()
                    );
                    if mint_data.is_err() {
                        debug!(
                            "mint_tweet_request: Could not parse data :{}",
                            transaction.transaction_hash
                        );
                        return Ok(false);
                    }
                    let mint_data = mint_data.unwrap();

                    // mint call
                    let ed25519_public_key = Ed25519PublicKey::from(sk);
                    let nonce = client.view_access_key(
                        &nft_contract_id,
                        &ed25519_public_key,
                        Finality::Final
                    ).await;
                    if nonce.is_err() {
                        debug!("Failed to fetch nonce at :{}", transaction.transaction_hash);
                        return Ok(false);
                    }

                    // Generate Proof
                    let proof_url = get_proof(mint_data.tweet_id).await;

                    debug!("PROOF_URL: {}", &proof_url);

                    let fetched_nft = client.view::<Option<NftData>>(
                        &nft_contract_id,
                        Finality::Final,
                        "nft_token",
                        Some(
                            json!({
                                    "token_id": mint_data.tweet_id.to_string(),
                            })
                        )
                    ).await;
                    if fetched_nft.is_err() {
                        info!("Failed to fetched nft data at :{}", transaction.transaction_hash);
                        return Ok(false);
                    }

                    let fetched_nft = fetched_nft.unwrap().data();

                    if fetched_nft.is_some() {
                        debug!("NFT already minted :{}", transaction.transaction_hash);
                        return Ok(false);
                    }
                    debug!("fetched_nft: {:?}\nmint_data:{:?}", fetched_nft, &mint_data);

                    //Type issue had to improvise: signer needs sk
                    let derived_sk = Ed25519SecretKey::try_from_bytes(sk.as_bytes());
                    if derived_sk.is_err() {
                        return Ok(false);
                    }
                    let nonce = nonce.unwrap().nonce;
                    let signer = Signer::from_secret(
                        derived_sk.unwrap(),
                        nft_contract_id.clone(),
                        nonce
                    );

                    // Perform the Mint Fulfillemnt Transaction based on the mint request data
                    let chain_transaction = client
                        .function_call(&signer, &nft_contract_id, "nft_mint")
                        .args(
                            json!(
                            {
                                "token_id": mint_data.tweet_id.to_string(),
                                "receiver_id": transaction.signer_account_id,
                                "token_metadata": {
                                    "media": mint_data.image_url,
                                    // Add proof and more description
                                    "extra":&proof_url,
                                    "reference": proof_url,
                                }
                            }
                        )
                        )
                        .deposit(7340000000000000000000)
                        .gas(10000000000000)
                        .retry(near_client::client::Retry::ONCE)
                        .commit(Finality::Final).await;

                    if chain_transaction.is_err() {
                        debug!(
                            "On Chain Transaction failed :{}\n Details:{:?}",
                            transaction.transaction_hash,
                            chain_transaction.err()
                        );
                        return Ok(false);
                    }

                    let chain_transaction = chain_transaction.unwrap();
                    debug!(
                        "ON Chain Transaction Hash {:?} for mint {}",
                        chain_transaction.id().to_string(),
                        transaction.transaction_hash
                    );

                    let new_transaction = near_transaction::ActiveModel {
                        id: Set(pk),
                        transaction_hash: Set(transaction.transaction_hash.clone()),
                        signer_account_id: Set(transaction.signer_account_id.clone()),
                        receiver_account_id: Set(transaction.receiver_account_id.clone()),
                        block_timestamp: Set(transaction.transaction_hash.clone()),
                        block_height: Set(transaction.block.block_height.try_into().unwrap()),
                        action: Set(action.action.clone()),
                        method: Set(method.clone()),
                        outcomes_status: Set(transaction.outcomes.status),
                        tweet_id: Set(mint_data.tweet_id.to_string()),
                        image_url: Set(mint_data.image_url.clone()),
                        user_to_notify: Set(Some(mint_data.notify.clone())),
                        mint_transaction_hash: Set(Some(chain_transaction.id().to_string())),
                        ..Default::default() // all other attributes are `NotSet`
                    };
                    near_transaction::Entity::insert(new_transaction).exec(db).await?;
                    return Ok(false);
                }
            } else {
                debug!(
                    "Ignored Transaction: {} Method Called: {:?}",
                    transaction.transaction_hash,
                    action.method
                );
                return Ok(false);
            }
            Ok(false)
        }
    }
}

use k256::SecretKey;
use rand::rngs::OsRng;
use verity_client::client::{ AnalysisConfig, VerityClient, VerityClientConfig };

pub fn get_verity_client() -> VerityClient {
    let secret_key = SecretKey::random(&mut OsRng);

    let verity_config = VerityClientConfig {
        prover_url: String::from("http://127.0.0.1:8080"),
        prover_zmq: String::from("tcp://127.0.0.1:5556"),
        analysis: Some(AnalysisConfig {
            analysis_url: String::from("https://analysis.verity.usher.so"),
            secret_key,
        }),
    };

    VerityClient::new(verity_config)
}
