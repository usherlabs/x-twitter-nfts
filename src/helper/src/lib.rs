use std::error::Error;
use std::marker::{Send,Sync};
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
