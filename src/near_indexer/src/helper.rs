use std::{collections::HashMap, time::Duration};
use std::error::Error;
use std::marker::{Send,Sync};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;


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
            self.page_no=self.page_no+1;
            self.cursor=Some(cursor);
        }

        Ok(data.txns)
    }

    pub fn is_next_page(&self) ->bool {
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
        sleep(delay);
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
        assert_eq!(indexer.is_next_page(),true);

    }

    #[tokio::test]
    async fn test_near_explorer_indexer_testnet() {
        let mut indexer = NearExplorerIndexer::new("ush_test.testnet").unwrap();
        let data_size =indexer.get_transactions().await.unwrap().len();
        assert!(data_size<25);
        assert_eq!(indexer.is_next_page(),false);
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
