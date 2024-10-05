use std::error::Error;
use std::marker::{Send,Sync};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize,};
use tracing::debug;
use reqwest;
use std::collections::HashMap;
use serde_json::json;
use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page;

pub async fn create_twitter_post_image(url:String)->Result<Vec<u8>, Box<dyn Error+Send+Sync>>  {


    let prefix = ["https://x.com","https://twitter.com"];

    assert!(url.to_lowercase().starts_with(&prefix[0])||url.to_lowercase().starts_with(&prefix[1]), "Hostname must be twitter.com or x.com");
    

    let browser = Browser::default()?;

    browser.get_process_id();
    let tab = browser.new_tab()?;
    tab.set_default_timeout(std::time::Duration::from_secs(60));

    // Set screen Dimension (in this case mobile)
    let tab = tab.set_bounds(
        headless_chrome::types::Bounds::Normal { 
            left: Some(0), 
            top: Some(0), 
            width: Some(720.0), 
            height: Some(4500.0) 
        })?;

    // Navigate to wikipedia
    tab.navigate_to(&url)?;
    

    tab.wait_until_navigated()?;

    let page_error=tab.find_element_by_xpath("/html/body/div/div/div/div[2]/main/div/div/div/div/div/div[3]/div/span");
    match page_error {
        Ok(element) => {
           let data= element.get_inner_text();
           match data {
               Ok(data)=>{
                   return Err(format!("PageError Found: {}", data).into());
               },
               Err(_)=>{},
           }
        },
        Err(_) => {
            // Proceed with execution for successful case
            println!("Page didn't error out");
        }
    }

    if let Ok(element) = tab.wait_for_elements_by_xpath("//*[@id=\"react-root\"]/div/div/div[2]/main/div/div/div/div[1]/div/section/div/div/div[1]"){

       let view_port=element[0].get_box_model().unwrap().content_viewport();
    
        // Take a screenshot a cropped view of the browser window
        let jpeg_data = tab.capture_screenshot(
            Page::CaptureScreenshotFormatOption::Png,
            Some(100),
            Some(Page::Viewport { x: view_port.x, y: view_port.y, width:view_port.width, height: view_port.height-109.0, scale: 2.0 }),
            true)?;
            
            Ok(jpeg_data)
    }else {
        // Take a screenshot a cropped view of the browser window
        let jpeg_data = tab.capture_screenshot(
            Page::CaptureScreenshotFormatOption::Png,
            Some(100),
            Some(Page::Viewport { x: 10.0, y: 0.0, width:750.0, height: 1250.0, scale: 2.0 }),
            true)?;
            
            Ok(jpeg_data)
    }
}

pub async fn create_twitter_post_image_from_id(tweet_id :u64)->Result<Vec<u8>, Box<dyn Error+Sync+Send>>  {
    return create_twitter_post_image(format!("https://x.com/x/status/{}", tweet_id)).await;
}


#[cfg(test)]
mod tests {    
    use super::*;
    use tokio;
    use headless_chrome;

    #[tokio::test]
    async fn test_url_starts_with_twitter_com() {

        println!("{:?}", headless_chrome::browser::default_executable());
        let tweet_id:u64 = 1834071245224308850;
        match create_twitter_post_image_from_id(tweet_id).await {
            Ok(bytes) => {
                assert!(bytes.len() > 0, "Image bytes should not be empty");
                    // Save the screenshot to disc
                    let _ = std::fs::write("./screenshot.png", bytes);
            }
            Err(e) => panic!("Expected Ok, but got Err: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_url_starts_with_other_prefix() {
        let url = "https://example.com/some/path".to_string();
        // This should fail as the URL does not start with "https://x.com" or "https://twitter.com".
        let result = std::panic::catch_unwind(|| {
            let _ = futures::executor::block_on(create_twitter_post_image(url));
        });
        assert!(result.is_err(), "The URL was expected to be invalid");
    }

    #[tokio::test]
    async fn test_url_starts_with_other_prefix_case_insensitive() {
        let url = "https://EXAMPLE.COM/some/path".to_string();
        // This should fail as the URL does not start with "https://x.com" or "https://twitter.com".
        let result = std::panic::catch_unwind(|| {
            let _ =  futures::executor::block_on(create_twitter_post_image(url));
        });
        assert!(result.is_err(), "The URL was expected to be invalid");
    }

    #[tokio::test]
    async fn test_empty_url() {
        let url = "".to_string();
        // This should fail as the URL is empty and does not start with either prefix.
        let result = std::panic::catch_unwind(|| {
            let _ =   futures::executor::block_on(create_twitter_post_image(url));
        });
        assert!(result.is_err(), "The URL was expected to be invalid");
    }
}

const API_URL_BASE: &str = "https://wallet.bitte.ai/";
#[derive(Serialize, Deserialize, Debug)]
struct _Data {
    name: String,
    description: Option<String>,
    metadata: Metadata,
}

#[derive(Serialize, Deserialize, Debug , Clone)]
struct Metadata {
    url: Option<String>,
    hash: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: Option<String>,
    data: Option<_Data>,
    id: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    messages: Vec<Message>,
}
pub struct BitteImageGenerator<'a> {
    pub session_key: String,
    pub creator: &'a str
}

impl<'a> BitteImageGenerator<'a> {
    pub async fn new(creator: &'a str,) -> Result<Self, Box<dyn Error>> {
        
        // Serialize the payload to JSON
        let payload = json!({
            "creator": &creator,
            "message": "I need an NFT minted for a tweet",
        });

        let client = reqwest::Client::new();
        let response = client.post(format!("{}api/smart-action/create", API_URL_BASE))
            .json(&payload)
            .send()
            .await.unwrap().json::<HashMap<String, String>>().await.unwrap();
        
            println!("{:?}", response);
            let session_key = response.get("id").ok_or("Missing 'id' in response")?;
            let session_key = session_key.clone();
            debug!("{:?}", response);


            Ok(BitteImageGenerator { 
                session_key: session_key.clone(),
                creator: creator
            })
    }
    pub async fn add_conversation(&mut self, conversation: &str) -> Result<String, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let payload =json!(
            {
                "accountData": {
                // devicePublicKey and  accountId should be changed to company values
                  "devicePublicKey": "ed25519:8rdDSTRtfa651Vigcp9HVJ4MJzpiVvK2Mv6ziZnif5bb",
                  "accountId": self.creator,
                  "isCreated": true
                },
                "walletConfig": {
                  "network": "mainnet",
                  "networkConfig": {
                    "networkId": "mainnet",
                    "viewAccountId": "mainnet",
                    "nodeUrl": "https://free.rpc.fastnear.com/",
                    "walletUrl": "https://wallet.mainnet.near.org",
                    "helperUrl": "https://helper.mainnet.near.org"
                  },
                  "relayer": {
                    "accountId": "mintbase.near"
                  }
                },
                "kvId": self.session_key,
                "config": {
                  "mode": "default"
                },
                "threadId": null,
                "message": conversation,
              }
        );
        let response = client.post(format!("{}api/ai-router/v1/assistants",API_URL_BASE ))
            .json(&payload)
            .send()
            .await?;


        let response = response.text().await?;
        if response.contains("proceed") || response.contains("suggested-prompts"){
        let payload =json!(
            {
                "accountData": {
                // devicePublicKey and  accountId should be changed to company values or creator values
                  "devicePublicKey": "ed25519:8rdDSTRtfa651Vigcp9HVJ4MJzpiVvK2Mv6ziZnif5bb",
                  "accountId": self.creator,
                  "isCreated": true
                },
                "walletConfig": {
                  "network": "mainnet",
                  "networkConfig": {
                    "networkId": "mainnet",
                    "viewAccountId": "mainnet",
                    "nodeUrl": "https://free.rpc.fastnear.com/",
                    "walletUrl": "https://wallet.mainnet.near.org",
                    "helperUrl": "https://helper.mainnet.near.org"
                  },
                  "relayer": {
                    "accountId": "mintbase.near"
                  }
                },
                "kvId": self.session_key,
                "config": {
                  "mode": "default"
                },
                "threadId": null,
                "message": "yes, mint for free",
              }
        );
        client.post(format!("{}api/ai-router/v1/assistants",API_URL_BASE ))
            .json(&payload)
            .send()
            .await?;
        }
        let response = client.get(format!("{}api/smart-action/create/{}",API_URL_BASE,self.session_key ))
        .send()
        .await.unwrap().json::<Response>().await.unwrap();
        // let response =from_str(response.as_str());

    // Filter messages to find the one with `name: "generate-image"`
     if let Some(message) = response.messages.iter().rev().find(|msg| {
            if let Some(data) = &msg.data {
                data.name == "generate-image"
            } else {
                false
            }
        }) {
            match &message.data {
                None=> Err("Not Data".into()),
                Some(data)=>{
                    let _data= data.metadata.clone();
                    if let Some(error) = _data.error {
                        if error != "null" {
                            return Err(format!("Error Generating image: {}", error).into());
                        }
                    }
                    Ok(_data.url.unwrap())
                }
            }
        }else{
            Err("Not Found".into())
        } 
    }

    pub async fn generate(&mut self, tweet_content: &str) -> Result<String, Box<dyn Error>> {
        let prompt =format!("I need an NFT generated for this tweet (NB: the image must contain tweet data)  data:```{}```",tweet_content.to_string());
        self.add_conversation(&prompt).await
    }
}

#[cfg(test)]
mod test_bitte_image_generator {
    use super::*;

    #[tokio::test]
    async fn test_async_image_generator() {
        let mut generator = BitteImageGenerator::new("xlassix.near").await.unwrap();
        assert_eq!(generator.session_key.len(), 21);
        let image_url1=generator.generate("10015.io @10015io Hello world! 👋 Do you know that http://10015.io offers the best online tool for converting tweets into fancy images with lots of customization options? 🐦 ↪️ 🖼️ #tweet #image #converter").await.unwrap();
        let image_url=generator.add_conversation("i need another image in abstract futuristic art style use the detail for NFT 1, Please don't forget the add the description into the image generated").await.unwrap();
        assert_ne!(image_url,image_url1);
    }
}

pub struct NearExplorerIndexer<'a> {
    pub page_no: u8,
    pub cursor: String,
    pub account_id: &'a str
}

impl<'a> NearExplorerIndexer<'a> {
    pub fn new(account_id: &'a str,) -> Result<Self, Box<dyn Error>> {
        if !(account_id.ends_with(".testnet")||account_id.ends_with(".near")){
            return Err("Invalid account_id".into());
        }

        Ok(NearExplorerIndexer { 
            cursor: String::from(""),
            page_no:0,
            account_id: &account_id
        })
    }

    pub async fn get_transactions(&mut self) -> Result<Vec<Transaction>, Box<dyn Error+Send+Sync>> {
        let data = self.fetch().await?;
        if let Some(cursor)=data.cursor{
            self.cursor=cursor;
            self.page_no=self.page_no+1;
        }

        Ok(data.txns)
    }

    async fn fetch(&self) -> Result<TransactionData, Box<dyn Error+Send+Sync>> {
        let client = reqwest::Client::new();
        let build_id= if self.account_id.ends_with(".testnet") {"0kmxltnOS1UsrfVrMd5fP"}else{"bE43kUihJPVfWqBYXxGBQ"};
        let base= if self.account_id.ends_with(".testnet") {"testnet.nearblocks.io"}else{"nearblocks.io"};
        
        // let response=  response.json::<NearIndexerData>().await;
        
        
        // if response.is_err(){
        //     return Err(format!("FETCH_ERROR: {},",response.err().expect("parse Failed")).into());
        // }
        let max_retries=5;
        let mut retries = 0;
        loop {
            let url = if self.page_no==0 {format!("https://{}/_next/data/{}/en/address/{}.json",base,build_id,self.account_id)}else{format!("https://{}/_next/data/{}/en/address/{}.json?id=xlassixx.near&cursor={}&p={}",base,build_id,self.account_id,self.cursor,self.page_no)};
            let response = client.get(url).send().await?;
            match response.json::<NearIndexerData>().await {
                Ok(output) => return  Ok(output.pageProps.data),
                Err(e) => {
                    if retries >= max_retries {
                        return Err(format!("FETCH_ERROR: {},",e).into());
                    }
                    retries += 1;
                    let delay = Duration::from_millis(500) * 2u32.pow(retries as u32 - 1);
                    thread::sleep(delay);
                }
            }
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
    action: Option<String>,
    method: Option<String>,
    deposit: u128,
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
