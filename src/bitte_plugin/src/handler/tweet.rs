
use std::env;

use reqwest::{ multipart::{Form, Part}, Client};
use rocket::serde::json::{json, Json, Value};
use tracing::debug;

use crate::{handler::IpfsData, models::response::NetworkResponse};


#[get("/tweet?<tweet_id>")]
pub async fn mint_tweet_request(
    tweet_id: Option<String>
) -> NetworkResponse{
    if tweet_id.is_none(){
        return NetworkResponse::BadRequest(json!({
            "error": "tweet_id is required"
        }));
    }

    let tweet_id= tweet_id.unwrap();
    let thirdweb_client_id=env::var("THIRDWEB_CLIENT_ID").expect("MY_VAR must be set");

    let _tweet_id=tweet_id.parse::<u64>();

    if _tweet_id.is_err(){
        return NetworkResponse::BadRequest(json!({
            "error": "invalid Tweet Id"
        }));
    }

    let description =get_tweet_content(&tweet_id).await;

    if description.is_err(){
        return NetworkResponse::BadRequest(json!({
            "error": format!("failed to retrieve tweet Description : {}",description.err().expect("Failed to get post image"))
        }));
    }
    let description=description.unwrap();

    debug!("description: {}",&description);
    let image=helper::create_twitter_post_image_from_id(_tweet_id.unwrap()).await;

    if image.is_err(){
        debug!("{}",format!("{:#?}",image.err()));
       return NetworkResponse::StatusOk(json!({
            "description": description,
            "imageURL": ""
        }));
    }

    let image= image.unwrap();



    let client = Client::new();

    let form = Form::new()
    .part("file", Part::bytes(image).file_name("image.png"))
    .part("pinataOptions", Part::text("{\"wrapWithDirectory\":false}"))
    .part("pinataOptions", Part::text("{\"wrapWithDirectory\":false}"))
    .part("pinataMetadata", Part::text("{\"name\":\"Storage SDK\",\"keyvalues\":{}}"));

    // Return a JSON response
    let url = "https://storage.thirdweb.com/ipfs/upload";
    let response = client.post(url)
        .header("X-Client-Id",&thirdweb_client_id )
        .header("Content-Type", format!("multipart/form-data; boundary={}", form.boundary()))
        .multipart(form)
        .send()
        .await;

    
    if response.is_err(){
        return NetworkResponse::BadRequest(json!({
            "error": format!("IPFS_ERROR: {}",response.err().expect("IPFS Upload Failed"))
        }));
    }

    
    let response=  response.unwrap().json::<IpfsData>()
        .await;
    
    if response.is_err(){
        return NetworkResponse::BadRequest(json!({
            "error": format!("IPFS_ERROR: {}",response.err().expect("IPFS Upload Failed"))
        }));
    }

    let image_url=format!(
        "https://{}.ipfscdn.io/ipfs/{}",
        thirdweb_client_id,
        response.unwrap().IpfsHash
    );

    debug!("image_url: {}",&image_url);

    NetworkResponse::StatusOk(json!({
        "description": description,
        "imageURL": image_url
    }))
}


async fn get_tweet_content(tweet_id: &str) -> Result<String, Box<dyn std::error::Error+Sync+Send>> {
    let client = Client::new();

    let url = format!(
        "https://api.x.com/2/tweets?ids={}",
        tweet_id
    );

    let response = client
        .get(&url)
        .header
            ("Authorization", format!("Bearer {}", env::var("TWEET_BEARER").unwrap()))
        .header("Host", "api.x.com")
        .header   ("Accept", "*/*")
        .header ("Cache-Control", "no-cache")
        .send()
        .await?;

    let json: Value = response.json().await?;

    Ok(json["data"][0]["text"].as_str().unwrap_or("").to_string())
}
fn cleanup_image_link(image_url: &str) -> String {
    if image_url.contains("ipfs") {
        let ipfs_hash = image_url.split('/').filter(|elem| elem.len() >= 46).next().unwrap_or_default();
        format!("ipfs://{}", ipfs_hash)
    } else  if image_url.contains("arweave") {
        let image_hash = image_url.split('/').filter(|elem| elem.len() >= 40).next().unwrap_or_default();
        format!("ar://{}", image_hash)
    }else {
        image_url.to_string()
    }
}


#[get("/tweet-contract-call?<tweet_id>&<image_url>&<notify>")]
pub async fn tweet_contract_call(
    tweet_id: String,
    image_url: String,
    notify: Option<String>
) -> Json<Value> {
    let contract_id= env::var("NEAR_CONTRACT_ADDRESS").unwrap_or(String::from("xlassixx.near"));
    let notify= notify.unwrap_or(String::from(""));
    Json(json!(
        [
            {
            "receiverId":  contract_id,
            "functionCalls": [
                {
                "methodName": "mint_tweet_request",
                "args": {
                    "tweet_id": tweet_id,
                    "image_url": cleanup_image_link(&image_url),
                    "notify": notify,
                },
                "gas": "20000000000000000",
                "amount": "5870000000000000000000",
                },
                ],
            },
        ]
    ))
}