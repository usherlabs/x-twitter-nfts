use crate::entity::near_transaction;

use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::time::Duration;
use std::error::Error;
use std::marker::{Send,Sync};
use near_client::client::{NearClient, Signer};
use near_client::crypto::Key;
use near_client::prelude::{AccountId, Ed25519PublicKey, Ed25519SecretKey, Finality};
use regex::Regex;
use reqwest::Url;
use sea_orm::{DbConn, DbErr, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, Error as SJError};
use tokio::time::sleep;
use sea_orm::ActiveValue::{Set};
use tracing::{debug, error};


pub struct NearExplorerIndexer<'a> {
    pub page_no: u8,
    pub cursor: Option<String>,
    pub account_id: &'a str,
    pub build_id: String,
}

impl<'a> NearExplorerIndexer<'a> {
    pub fn new(account_id: &'a str,) -> Result<Self, Box<dyn Error>> {
        if !(account_id.ends_with(".testnet")||account_id.ends_with(".near")){
            return Err("Invalid account_id".into());
        }
        
        Ok(NearExplorerIndexer { 
            cursor: None,
            page_no:0,
            build_id:(if account_id.ends_with(".testnet") {"0kmxltnOS1UsrfVrMd5fP"}else{"bE43kUihJPVfWqBYXxGBQ"}).to_owned(),
            account_id: &account_id
        })
    }

    /// Retrieves the Latest Batch of transactions.
    ///
    /// This method continues fetching transactions from latest
    ///
    /// # Returns
    /// A vector of transactions representing the next page.
    pub async fn get_transactions(&mut self) -> Result<Vec<Transaction>, Box<dyn Error+Send+Sync>> {
        let data = self.fetch(true).await?;

        if data.cursor.is_some(){
            self.page_no=1;
            self.cursor=data.cursor;
        }

        Ok(data.txns)
    }

    /// Retrieves the next page of transactions.
    ///
    /// This method continues fetching transactions from where the previous call left off.
    ///
    /// # Returns
    /// A vector of transactions representing the next page.
    pub async fn next_page(&mut self) -> Result<Vec<Transaction>, Box<dyn Error+Send+Sync>> {
        let data = self.fetch(false).await?;
        if let Some(cursor)=data.cursor{
            self.cursor=Some(cursor);
        }else{
            self.cursor=None;
        }
        self.page_no=self.page_no+1;

        Ok(data.txns)
    }

    pub fn has_next_page(&self) ->bool {
        self.cursor.is_some()
    }
    async fn fetch(&mut self,reset: bool) -> Result<TransactionData, Box<dyn Error+Send+Sync>> {
        let client = reqwest::Client::new();
        let base= if self.account_id.ends_with(".testnet") {"testnet.nearblocks.io"}else{"nearblocks.io"};
        
        let max_retries=5;
        let mut retries = 0;
        loop {
            let url = if self.page_no==0||reset {format!("https://{}/_next/data/{}/en/address/{}.json",base,self.build_id,self.account_id)}else{format!("https://{}/_next/data/{}/en/address/{}.json?cursor={}&p={}",base,self.build_id,self.account_id,self.cursor.clone().unwrap_or("".to_owned()),self.page_no)};
            let response = client.get(&url).send().await?;

           if (&response).status()==404 {
            let html_text =response.text().await.unwrap();
            let re = Regex::new(r"_next/static/([a-zA-Z0-9\-_]+)/_buildManifest").unwrap();
    
            // Attempt to find captures in the text
            if let Some(captures) = re.captures(&html_text) {
                // Extract the first capture group, which is our build number
                if let Some(build_number) = captures.get(1) {
                  self.build_id= build_number.as_str().to_owned();
                }
            } 
           }else{
               match response.json::<NearIndexerData>().await {
                   Ok(output) => return  Ok(output.pageProps.data),
                   Err(e) => {
            
                    if retries >= max_retries {
                        return Err(format!("FETCH_ERROR: {}, {}",e,&url).into());
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
        assert_eq!(indexer.get_transactions().await.unwrap().len(),25);
        assert_eq!(indexer.next_page().await.unwrap().len(),25);
        assert_eq!(indexer.has_next_page(),true);

    }

    #[tokio::test]
    async fn test_near_explorer_indexer_testnet() {
        let mut indexer = NearExplorerIndexer::new("ush_test.testnet").unwrap();
        let data_size =indexer.get_transactions().await.unwrap().len();
        assert!(data_size<25);
        assert_eq!(indexer.has_next_page(),false);
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct NearIndexerData {
    pageProps: PageProps,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize,Debug)]
pub struct PageProps {
    statsDetails: StatsDetails,
    // accountDetails: AccountDetails,
    data: TransactionData,
    dataCount: DataCount,
}

#[derive(Serialize, Deserialize,Debug)]
struct StatsDetails {
    stats: Vec<Stat>,
}

#[derive(Serialize, Deserialize,Debug)]
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

#[derive(Serialize, Deserialize,Debug)]
struct AccountDetails {
    account: Vec<Account>,
}

#[derive(Serialize, Deserialize,Debug)]
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

#[derive(Serialize, Deserialize,Debug)]
struct Created {
    transaction_hash: Option<String>,
    block_timestamp: Option<u64>,
}

#[derive(Serialize, Deserialize,Debug)]
struct Deleted {
    transaction_hash: Option<String>,
    block_timestamp: Option<u64>,
}

#[derive(Serialize, Deserialize,Debug)]
struct ContractData {
    deployments: Vec<Deployment>,
}

#[derive(Serialize, Deserialize,Debug)]
struct Deployment {
    transaction_hash: String,
    block_timestamp: u64,
    receipt_predecessor_account_id: String,
}

#[derive(Serialize, Deserialize,Debug)]
struct TokenDetails {}

#[derive(Serialize, Deserialize,Debug)]
struct NftTokenDetails {}

#[derive(Serialize, Deserialize,Debug)]
struct ParseDetails {
    contract: Vec<Contract>,
}

#[derive(Serialize, Deserialize,Debug)]
struct Contract {
    contract: ContractInfo,
}

#[derive(Serialize, Deserialize,Debug)]
struct ContractInfo {
    method_names: Vec<String>,
    probable_interfaces: Vec<String>,
    by_method: HashMap<String, Vec<String>>,
    schema: Option<String>,
}

#[derive(Serialize, Deserialize,Debug)]
struct InventoryDetails {
    inventory: Inventory,
}

#[derive(Serialize, Deserialize,Debug)]
struct Inventory {
    fts: Vec<String>,
    nfts: Vec<String>,
}

#[derive(Serialize, Deserialize,Debug)]
pub struct TransactionData {
    cursor: Option<String>,
    txns: Vec<Transaction>,
}




#[derive(Serialize, Deserialize,Debug)]
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
    outcomes: Outcomes
}


#[derive(Serialize, Deserialize,Debug)]
struct Block {
    block_height: u128,
}

#[derive(Serialize, Deserialize,Debug)]
struct Action {
    action: String,
    method: Option<String>,
    args: Option<String>,
}

#[derive(Serialize, Deserialize,Debug)]
struct ActionsAgg {
    deposit: u128,
}

#[derive(Serialize, Deserialize,Debug)]
struct Outcomes {
    status: bool,
}

#[derive(Serialize, Deserialize,Debug)]
struct DataCount {
    txns: Vec<DataCountTxn>,
}

#[derive(Serialize, Deserialize,Debug)]
struct DataCountTxn {
    count: String,
}

#[derive(Serialize, Deserialize,Debug)]
struct LatestBlocks {
    blocks: Vec<Block>,
}

#[derive(Serialize, Deserialize,Debug)]
struct Tab {
    tab: String,
}

#[derive(Debug, Deserialize)]
struct MintRequestData {
    notify: String,
    tweet_id: u64,
    image_url:String,
}


pub async fn process_near_transaction(db: &DbConn,transaction: &Transaction) -> Result<bool, DbErr> {
    
    // let near_rpc = env::var("NEAR_RPC").unwrap_or("https://rpc.testnet.near.org".to_owned());
    // let nft_contract_id=env::var("NFT_CONTRACT_ID").unwrap_or("test-usher.testnet".to_owned());
    // let sk=env::var("SIGNER_SK").unwrap_or("testnet".to_owned());
    // let ed25519SecretKey=Ed25519SecretKey::from_string(&sk).unwrap();
    // let nft_contract_id=AccountId::from_str(&nft_contract_id).unwrap();

    
    let pk =transaction.id.parse::<i32>().unwrap();

    // Find by primary key
    let _near_transaction: Option<near_transaction::Model> = near_transaction::Entity::find_by_id(pk).one(db).await?;
    match _near_transaction {
        Some(_)=>{
            Ok(true)
        },
        None=>{
            if !transaction.outcomes.status{
                debug!("Failed BlockChain Transaction Ignored, {}",transaction.transaction_hash);
                return  Ok(false);
            }
            if transaction.actions[0].method.is_none(){
                debug!("Ignored Transaction: {} No method Found",transaction.transaction_hash);
                return  Ok(false);
            }
            let action = match transaction.actions.get(0) {
                Some(action) => action,
                None => &Action { action: "".to_string(), method: None, args: None },
            };
            
            if let Some(method) = &action.method {
                // LIST OPERATIONS 
                if method == "mint_tweet_request" {
                    let mint_data:Result<MintRequestData,SJError> = serde_json::from_str(action.args.clone().unwrap_or("".to_string()).as_str());
                    if mint_data.is_err(){
                        error!("mint_tweet_request: Could not parse data :{}",transaction.transaction_hash);
                        return  Ok(false);
                    }
                    let mint_data=mint_data.unwrap();
 

                    
                    // // mint call
                    // let client = NearClient::new(Url::from_str(&near_rpc).unwrap()).unwrap();
                    // let ed25519_public_key = Ed25519PublicKey::from_string(&sk).unwrap();
                    // let nonce = client
                    // .view_access_key(&nft_contract_id, &ed25519_public_key, Finality::Final)
                    //     .await;
                    // if nonce.is_err(){
                    //     error!("Failed to fetch nonce at :{}",transaction.transaction_hash);
                    //     return Ok(false);
                    // }
                    // let nonce =nonce.unwrap().nonce;
                    // let signer = Signer::from_secret(ed25519SecretKey,nft_contract_id.clone(), nonce);
                    // let chain_transaction = client.function_call(&signer, &nft_contract_id, "nft_mint").args(json!(
                    //     {
                    //         "token_id": mint_data.tweet_id,
                    //         "receiver_id": transaction.signer_account_id,
                    //         "metadata": {
                    //             "media": mint_data.image_url,
                    //             // Add proof and more description
                    //         }
                    //     }
                    // ))
                    // .deposit(5870000000000000000000)
                    // .retry(near_client::client::Retry::TWICE)
                    // .commit_async(Finality::Final).await;

                    // if chain_transaction.is_err(){
                    //     error!("On Chain Transaction failed :{}\n Details:{:?}",transaction.transaction_hash,chain_transaction.err());
                    //     return Ok(false);
                    // }

                    // debug!("ON Chain Transaction Hash {} for mint {}",chain_transaction.unwrap(), transaction.transaction_hash);
                                

                    let new_transaction = near_transaction::ActiveModel {
                        id: Set(pk),
                        transaction_hash:Set(transaction.transaction_hash.clone()),
                        signer_account_id: Set(transaction.signer_account_id.clone()),
                        receiver_account_id: Set(transaction.receiver_account_id.clone()),
                        block_timestamp: Set(transaction.transaction_hash.clone()),
                        block_height: Set(transaction.block.block_height.try_into().unwrap()),
                        action: Set(action.action.clone()),
                        method: Set(method.clone()),
                        outcomes_status: Set(transaction.outcomes.status),
                        tweet_id: Set(mint_data.tweet_id.to_string()),
                        image_url: Set(mint_data.image_url.clone()),
                        user_to_notify:Set(Some(mint_data.notify.clone())),
                        ..Default::default() // all other attributes are `NotSet`
                    };
                    near_transaction::Entity::insert(new_transaction).exec(db).await?;
                    println!("{:?}", transaction);
                    return  Ok(false);
                }
            }else{
                debug!("Ignored Transaction: {} Method Called: {:?}",transaction.transaction_hash,action.method);
                return  Ok(false);
            }
            Ok(false)
        }
    }
}
