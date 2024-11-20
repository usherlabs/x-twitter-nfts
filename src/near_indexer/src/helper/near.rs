use crate::entity::near_transaction;

use crate::helper::aurora::TxSender;
use crate::helper::nft::extract_metadata_from_request;
use crate::methods::VERIFY_ELF;
use alloy_sol_types::SolValue;
use ethers::utils::hex;
use near_client::client::NearClient;
use near_client::prelude::{AccountId, Finality};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_jsonrpc_client::methods::query::RpcQueryRequest;
use near_jsonrpc_client::methods::tx::RpcTransactionResponse;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::{RpcTransactionError, TransactionInfo};
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::BlockReference;
use near_primitives::views::{QueryRequest, TxExecutionStatus};
use regex::Regex;
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, ReceiptKind, VerifierContext};
use sea_orm::ActiveValue::Set;
use sea_orm::{DbConn, DbErr, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, Error as SJError};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::marker::{Send, Sync};
use std::str::FromStr;
use std::time::Duration;
use tokio::task::spawn_blocking;
use tokio::time;
use tokio::time::sleep;
use tracing::{debug, info};

pub async fn verify_near_proof(
    journal_output: Vec<u8>,
    token_metadata: TokenMetadata,
) -> Result<RpcTransactionResponse, Box<dyn std::error::Error>> {
    let rpc_url = env::var("NEAR_RPC_URL").expect("RPC_URL_NOT_PRESENT");
    let account_id = env::var("NEAR_SIGNER_ACCOUNT_ID").expect("ACCOUNT_ID_NOT_PRESENT");
    let secret_key = env::var("NEAR_ACCOUNT_SECRET_KEY").expect("SECRET_KEY_NOT_PRESENT");
    let contract_account_id =
        env::var("NEAR_VERIFIER_CONTRACT_ACCOUNT_ID").expect("CONTRACT_ACCOUNT_ID_NOT_PRESENT");

    let signer_account_id: near_primitives::types::AccountId = account_id.parse()?;
    let signer_secret_key: near_crypto::SecretKey = secret_key.parse()?;

    let signer = near_crypto::InMemorySigner::from_secret_key(signer_account_id, signer_secret_key);

    let client = JsonRpcClient::connect(rpc_url);
    let access_key_query_response = client
        .call(RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        })
        .await?;

    let current_nonce = match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ => Err("failed to extract current nonce")?,
    };

    let transaction = Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce: current_nonce + 1,
        receiver_id: contract_account_id.parse()?,
        block_hash: access_key_query_response.block_hash,
        actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
            method_name: "verify_proof".to_string(),
            args: json!({
                "journal": journal_output,
                "token_metadata": token_metadata
            })
            .to_string()
            .into_bytes(),
            gas: 100_000_000_000_000, // 100 TeraGas
            deposit: 0,
        }))],
    };

    let request = methods::broadcast_tx_async::RpcBroadcastTxAsyncRequest {
        signed_transaction: transaction.sign(&signer),
    };

    let sent_at = time::Instant::now();
    let tx_hash = client.call(request).await?;
    let res: Option<RpcTransactionResponse>;
    loop {
        let response = client
            .call(methods::tx::RpcTransactionStatusRequest {
                transaction_info: TransactionInfo::TransactionId {
                    tx_hash,
                    sender_account_id: signer.account_id.clone(),
                },
                wait_until: TxExecutionStatus::Executed,
            })
            .await;
        let received_at = time::Instant::now();
        let delta = (received_at - sent_at).as_secs();

        if delta > 60 {
            Err("time limit exceeded for the transaction to be recognized")?;
        }

        match response {
            Err(err) => match err.handler_error() {
                Some(
                    RpcTransactionError::TimeoutError
                    | RpcTransactionError::UnknownTransaction { .. },
                ) => {
                    time::sleep(time::Duration::from_secs(2)).await;
                    continue;
                }
                _ => Err(err)?,
            },
            Ok(response) => {
                debug!("response gotten after: {}s", delta);
                res = Some(response);
                break;
            }
        }
    }

    Ok(res.unwrap())
}

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
            build_id: (if account_id.ends_with(".testnet") {
                "0kmxltnOS1UsrfVrMd5fP"
            } else {
                "bE43kUihJPVfWqBYXxGBQ"
            })
            .to_owned(),
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
        &mut self,
    ) -> Result<Vec<JSONTransaction>, Box<dyn Error + Send + Sync>> {
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
    pub async fn next_page(
        &mut self,
    ) -> Result<Vec<JSONTransaction>, Box<dyn Error + Send + Sync>> {
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
        reset: bool,
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
                    base, self.build_id, self.account_id
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
    txns: Vec<JSONTransaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JSONTransaction {
    id: String,
    signer_account_id: String,
    receiver_account_id: String,
    transaction_hash: String,
    included_in_block_hash: String,
    block_timestamp: String,
    receipt_conversion_tokens_burnt: String,
    block: Block,
    actions: Vec<JSONAction>,
    outcomes: Outcomes,
}

#[derive(Serialize, Deserialize, Debug)]
struct Block {
    block_height: u128,
}

#[derive(Serialize, Deserialize, Debug)]
struct JSONAction {
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
    tweet_id: String,
    image_url: String,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct IpfsData {
    IpfsHash: String,
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

pub async fn get_proof(
    tweet_id: String,
) -> Result<(String, TweetResponse), Box<dyn std::error::Error>> {
    let verity_client = get_verity_client();
    let _temp = verity_client
        .get(
            format!("https://api.x.com/2/tweets?ids={}&tweet.fields=created_at,public_metrics&expansions=author_id&user.fields=created_at", tweet_id)
        )
        .header(
            "Authorization",
            format!("Bearer {}", env::var("TWEET_BEARER").unwrap_or(String::from("")))
        ).redact("authorization".to_string())
        .send().await;

    if _temp.is_err() {
        info!("Error: {:?} ", _temp.err());
        return Err(format!("Error found ID").into());
    }

    let verify_response = _temp.unwrap();
    let res = verify_response.subject.json::<TweetResponse>().await;
    if res.is_err() {
        info!("invalid Tweet at {},{:?}", tweet_id, res.err());
        return Err(format!("invalid Tweet ID").into());
    }
    let res = res.expect("response must exist");
    if res.data.is_some() {
        Ok((verify_response.proof, res))
    } else {
        info!("Error found at {},{:?}", tweet_id, res);
        Err(format!("Error found ID").into())
    }
}

pub async fn process_near_transaction(
    db: &DbConn,
    transaction: &JSONTransaction,
    client: &NearClient,
    notifier: &OathTweeterHandler,
) -> Result<bool, DbErr> {
    let nft_contract_id = env::var("NFT_CONTRACT_ID").unwrap_or("test-usher.testnet".to_owned());
    let nft_contract_id = AccountId::from_str(&nft_contract_id).unwrap();

    let pk = transaction.id.parse::<i32>().unwrap();

    // Find by primary key
    let _near_transaction: Option<near_transaction::Model> =
        near_transaction::Entity::find_by_id(pk).one(db).await?;
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
                debug!(
                    "Ignored Transaction: {} No method Found",
                    transaction.transaction_hash
                );
                return Ok(false);
            }
            let action = match transaction.actions.get(0) {
                Some(action) => action,
                None => {
                    &(JSONAction {
                        action: "".to_string(),
                        method: None,
                        args: None,
                    })
                }
            };

            if let Some(method) = &action.method {
                // LIST OPERATIONS
                if method == "mint_tweet_request" {
                    let mint_data: Result<MintRequestData, SJError> = serde_json::from_str(
                        action.args.clone().unwrap_or("".to_string()).as_str(),
                    );
                    if mint_data.is_err() {
                        debug!(
                            "mint_tweet_request: Could not parse data :{}",
                            transaction.transaction_hash
                        );
                        return Ok(false);
                    }
                    let mint_data = mint_data.unwrap();

                    let fetched_nft = client
                        .view::<Option<NftData>>(
                            &nft_contract_id,
                            Finality::Final,
                            "nft_token",
                            Some(json!({
                                    "token_id": mint_data.tweet_id.clone(),
                            })),
                        )
                        .await;
                    if fetched_nft.is_err() {
                        info!(
                            "Failed to fetched nft data at :{}",
                            transaction.transaction_hash
                        );
                        return Ok(false);
                    }

                    let fetched_nft = fetched_nft.unwrap().data();

                    if fetched_nft.is_some() {
                        debug!("NFT already minted :{}", transaction.transaction_hash);
                        return Ok(false);
                    }
                    debug!("fetched_nft: {:?}\nmint_data:{:?}", fetched_nft, &mint_data);

                    // Generate Proof
                    let proof = get_proof(mint_data.tweet_id.clone()).await;

                    if proof.is_err() {
                        info!("Invalid Tweet ID{}\n", &mint_data.tweet_id);
                        return Ok(false);
                    }
                    let (proof, tweet_res_data) = proof.unwrap();

                    let meta_data = AssetMetadata {
                        image_url: mint_data.image_url.to_string(),
                        owner_account_id: transaction.signer_account_id.clone(),
                        token_id: mint_data.tweet_id.clone(),
                    };

                    let zk_input = ZkInputParam {
                        proof,
                        meta_data: meta_data.clone(),
                    };

                    let (seal, journal_output) =
                        spawn_blocking(|| generate_groth16_proof(zk_input))
                            .await
                            .unwrap();

                    info!(
                        "{:?} was committed to the journal",
                        hex::encode(&journal_output)
                    );
                    info!("{:?} was the provided seal", hex::encode(&seal));
                    let aurora_client = TxSender::default();
                    let aurora_tx_future = aurora_client
                        .verify_proof_on_aurora(journal_output.clone(), seal)
                        .await;
                    let aurora_tx_response = aurora_tx_future.unwrap();
                    info!(
                        "Aurora transation has been verified with response: {:?}\n",
                        aurora_tx_response
                    );

                    let chain_transaction = aurora_tx_response.block_hash.unwrap();
                    debug!(
                        "ON Chain Transaction Hash {} for mint {}",
                        chain_transaction.to_string(),
                        transaction.transaction_hash
                    );

                    // perform verification near
                    // mint NFT if near verification is successfull
                    let near_tx_response = verify_near_proof(
                        journal_output,
                        extract_metadata_from_request(tweet_res_data, meta_data),
                    )
                    .await;
                    if near_tx_response.is_err() {
                        info!(
                            "Failed to mint {}\n {:?}\n",
                            &mint_data.tweet_id,
                            near_tx_response.err()
                        );
                        return Ok(false);
                    }
                    let near_tx_response = near_tx_response.expect("successful mint response");
                    debug!(
                        "Near transaction has been verified with response: {:?}\n",
                        near_tx_response
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
                        tweet_id: Set(mint_data.tweet_id.clone().to_string()),
                        image_url: Set(mint_data.image_url.clone()),
                        user_to_notify: Set(Some(mint_data.notify.clone())),
                        mint_transaction_hash: Set(Some(chain_transaction.to_string())),
                        ..Default::default() // all other attributes are `NotSet`
                    };
                    near_transaction::Entity::insert(new_transaction)
                        .exec(db)
                        .await?;

                    if !mint_data.notify.is_empty() {
                        let _ = notifier
                            .notifier(&mint_data.tweet_id.clone(), &mint_data.notify)
                            .await;
                    }
                    return Ok(false);
                }
            } else {
                debug!(
                    "Ignored Transaction: {} Method Called: {:?}",
                    transaction.transaction_hash, action.method
                );
                return Ok(false);
            }
            Ok(false)
        }
    }
}

use k256::SecretKey;
use rand::rngs::OsRng;
use verity_client::client::{AnalysisConfig, VerityClient, VerityClientConfig};

use super::nft::{TweetData, TweetResponse};
use super::twitter::OathTweeterHandler;

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

#[cfg(test)]
mod tests {
    use crate::helper::aurora::TxSender;

    use super::*;
    use dotenv::dotenv;
    use ethers::utils::hex;
    use tokio::{self, task::spawn_blocking};

    #[tokio::test]
    async fn test_get_verify() {
        dotenv().expect("Error occurred when loading .env");
        let tweet_id: u64 = 1858184885493485672;
        let proof = get_proof(tweet_id.to_string()).await;
        if proof.is_err() {
            debug!("{:?}", proof.err());
        } else {
            let (proof, _) = proof.unwrap();

            assert!(proof.len() > 100, "proof is invalid");

            let meta_data= AssetMetadata{
            image_url: "https://386f4b0d6749763bc7ab0a648c3e650f.ipfscdn.io/ipfs/QmXPD7KqFyFWwMTQyEo9HuTJjkKLxergS1YTt1wjJNAAHV".to_string(),
            owner_account_id:"xlassixx.testnet".to_string(),
            token_id: tweet_id.to_string(),
        };

            let zk_input = ZkInputParam {
                proof: proof,
                meta_data,
            };

            let (seal, journal_output) = spawn_blocking(|| generate_groth16_proof(zk_input))
                .await
                .unwrap();

            debug!(
                "{:?} was committed to the journal",
                hex::encode(&journal_output)
            );
            debug!("{:?} was the provided seal", hex::encode(&seal));
            let aurora_client = TxSender::default();
            let aurora_tx_future = aurora_client
                .verify_proof_on_aurora(journal_output.clone(), seal)
                .await;
            let aurora_tx_response = aurora_tx_future.unwrap();
            debug!(
                "Aurora transation has been verified with resriscponse: {:?}\n",
                aurora_tx_response
            );
            assert!(hex::check_raw(format!(
                "{:x}",
                aurora_tx_response.block_hash.unwrap()
            )));
        }
    }
}

pub fn generate_groth16_proof(zk_inputs: ZkInputParam) -> (Vec<u8>, Vec<u8>) {
    // serialize the inputs to bytes to pass to the remote prover
    let input = serde_json::to_string(&zk_inputs).unwrap();
    let input: &[u8] = input.as_bytes();


    // begin the proving process
    let env = ExecutorEnv::builder().write_slice(&input).build().unwrap();
    let receipt = default_prover()
        .prove_with_ctx(
            env,
            &VerifierContext::default(),
            VERIFY_ELF,
            &ProverOpts::groth16(),
        )
        .unwrap()
        .receipt;


    // // Encode the seal with the selector.
    let seal = groth16::encode(receipt.inner.groth16().unwrap().seal.clone()).unwrap();

    // // Extract the journal from the receipt.
    let journal = receipt.journal.bytes.clone();

    let journal_output = <Vec<u8>>::abi_decode(&journal, true)
        // .context("decoding journal data")
        .unwrap();

    (seal, journal_output)
}
