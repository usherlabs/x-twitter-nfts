use std::{env, ops::Index, str::FromStr};

use indexer::helper::TweetResponse;
use near_client::{
    client::NearClient,
    prelude::{AccountId, Finality},
};
use reqwest::{
    multipart::{Form, Part},
    Client,
};
use rocket::serde::json::{json, Json, Value};
use tracing::debug;
use url::Url;

use crate::{handler::IpfsData, helper, models::response::NetworkResponse};

/// Handles the request to mint a new tweet.
///
/// # Parameters
///
/// - `tweet_id`: Optional string parameter representing the ID of the tweet.
///
/// # Returns
///
/// A `NetworkResponse` indicating the result of the operation.

#[get("/tweet?<tweet_id>")]
pub async fn mint_tweet_request(tweet_id: Option<String>) -> NetworkResponse {
    if tweet_id.is_none() {
        return NetworkResponse::BadRequest(json!({
            "error": "tweet_id is required"
        }));
    }

    let tweet_id = tweet_id.unwrap();
    let thirdweb_client_id = env::var("THIRDWEB_CLIENT_ID").expect("MY_VAR must be set");

    let _tweet_id = tweet_id.parse::<u64>();

    if _tweet_id.is_err() {
        return NetworkResponse::BadRequest(json!({
            "error": "invalid Tweet Id"
        }));
    }

    let result = get_tweet_content(&tweet_id).await;

    let near_client = NearClient::new(
        Url::from_str(
            &env::var("NEAR_RPC").unwrap_or(String::from("https://rpc.testnet.near.org")),
        )
        .unwrap(),
    )
    .unwrap();

    if result.is_err() {
        return NetworkResponse::BadRequest(json!({
            "error": format!("failed to retrieve tweet Description : {}",result.err().expect("Failed to get post image"))
        }));
    }

    let (description, public_metric) = result.unwrap();

    let computed_cost = near_client
        .view::<u128>(
            &AccountId::from_str(
                &env::var("NEAR_CONTRACT_ADDRESS")
                    .unwrap_or("x-bitte-nft.testnet".to_string())
                    .to_owned(),
            )
            .unwrap(),
            Finality::Final,
            "compute_cost",
            Some(json!({
                "public_metrics": public_metric
            })),
        )
        .await
        .unwrap()
        .data();

    debug!("description: {}", &description);
    let image = helper::create_twitter_post_image_from_id(_tweet_id.unwrap()).await;

    if image.is_err() {
        debug!("{}", format!("{:#?}", image.err()));
        return NetworkResponse::StatusOk(json!({
            "description": description,
            "imageURL": ""
        }));
    }

    let image = image.unwrap();

    let client = Client::new();

    let form = Form::new()
        .part("file", Part::bytes(image).file_name("image.png"))
        .part("pinataOptions", Part::text("{\"wrapWithDirectory\":false}"))
        .part("pinataOptions", Part::text("{\"wrapWithDirectory\":false}"))
        .part(
            "pinataMetadata",
            Part::text("{\"name\":\"Storage SDK\",\"keyvalues\":{}}"),
        );

    // Return a JSON response
    let url = "https://storage.thirdweb.com/ipfs/upload";
    let response = client
        .post(url)
        .header("X-Client-Id", &thirdweb_client_id)
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", form.boundary()),
        )
        .multipart(form)
        .send()
        .await;

    if response.is_err() {
        return NetworkResponse::BadRequest(json!({
            "error": format!("IPFS_ERROR: {}",response.err().expect("IPFS Upload Failed"))
        }));
    }

    let response = response.unwrap().json::<IpfsData>().await;

    if response.is_err() {
        return NetworkResponse::BadRequest(json!({
            "error": format!("IPFS_ERROR: {}",response.err().expect("IPFS Upload Failed"))
        }));
    }

    let image_url = format!(
        "https://{}.ipfscdn.io/ipfs/{}",
        thirdweb_client_id,
        response.unwrap().IpfsHash
    );

    debug!(
        "image_url: {} \ncomputed_cost:{}",
        &image_url,
        &(computed_cost * 12 / 10).to_string()
    );

    NetworkResponse::StatusOk(json!({
        "description": description,
        "imageURL": image_url,
        "computed_cost": (computed_cost*12/10).to_string(),
    }))
}

/// Fetches the content of a tweet given its ID.
///
/// # Arguments
///
/// * `tweet_id`: The ID of the tweet to fetch.
///
/// # Returns
///
/// A `Result` containing the tweet text as a `String`, or an error if the request fails.
async fn get_tweet_content(
    tweet_id: &str,
) -> Result<(String, Value), Box<dyn std::error::Error + Sync + Send>> {
    // Create a new HTTP client
    let client = Client::new();

    // Construct the API URL with the tweet ID
    let url = format!("https://api.x.com/2/tweets?ids={}&&tweet.fields=created_at,public_metrics&expansions=author_id&user.fields=created_at", tweet_id);

    // Send GET request to the API
    let response = client
        .get(&url)
        .header(
            "Authorization",
            format!("Bearer {}", env::var("TWEET_BEARER").unwrap()),
        )
        .header("Host", "api.x.com")
        .header("Accept", "*/*")
        .header("Cache-Control", "no-cache")
        .send()
        .await?;

    // Parse the JSON response
    let json: TweetResponse = response.json().await?;

    if json.clone().errors.is_some() {
        return Err(format!("Error Found while fetching tweet").into());
    }

    let tweet_data = json.data.unwrap();
    // Extract and return the tweet text
    Ok((
        tweet_data.index(0).text.clone(),
        serde_json::to_value(tweet_data.index(0).public_metrics.clone()).unwrap(),
    ))
}

/// Cleans up image links by converting IPFS or Arweave URLs to their hash-only formats
///
/// # Parameters
///
/// - `image_url`: The URL of the image to clean up
///
/// # Returns
///
/// A cleaned-up version of the image URL, or the original URL if it doesn't contain IPFS or Arweave
fn cleanup_image_link(image_url: &str) -> String {
    // Check if the URL contains "ipfs"
    if image_url.contains("ipfs") {
        // Extract the IPFS hash from the URL
        let ipfs_hash = image_url
            .split('/')
            .filter(|elem| elem.len() >= 46)
            .next()
            .unwrap_or_default();

        // Format the IPFS hash as a full IPFS URL
        format!("ipfs://{}", ipfs_hash)
    } else if image_url.contains("arweave") {
        // Check if the URL contains "arweave"
        let image_hash = image_url
            .split('/')
            .filter(|elem| elem.len() >= 40)
            .next()
            .unwrap_or_default();

        // Format the Arweave hash as a full Arweave URL
        format!("ar://{}", image_hash)
    } else {
        // If neither IPFS nor Arweave, return the original URL unchanged
        image_url.to_string()
    }
}

#[get("/tweet-contract-call?<tweet_id>&<image_url>&<notify>&<computed_cost>")]
pub async fn tweet_contract_call(
    tweet_id: String,
    image_url: String,
    computed_cost: String,
    notify: Option<String>,
) -> Json<Value> {
    // Get the NEAR contract address from environment variable
    let contract_id = env::var("NEAR_CONTRACT_ADDRESS")
        .unwrap_or(String::from("xlassixx.near"))
        .to_owned();

    // Default value for notify if not provided
    let notify = notify.unwrap_or(String::from(""));

    // Construct the JSON payload for the smart contract call
    Json(json!(
            {
            "receiverId":  contract_id,
            "action_kind":"FunctionCall",
            "functionCalls": [
                {
                "methodName": "mint_tweet_request",
                "args": {
                    "tweet_id": tweet_id,
                    "image_url": cleanup_image_link(&image_url),
                    "notify": notify,
                },
                "deposit":computed_cost,
                "gas": "100000000000000",
                "amount": computed_cost,
                "defaultGas" : "3000000000000000000",
                },
                ],
            }
    ))
}

#[get("/tweet-cancel-call?<tweet_id>")]
pub async fn tweet_contract_cancel_call(tweet_id: String) -> Json<Value> {
    // Get the NEAR contract address from environment variable
    let contract_id = env::var("NEAR_CONTRACT_ADDRESS")
        .unwrap_or(String::from("xlassixx.near"))
        .to_owned();

    // Construct the JSON payload for the smart contract call
    Json(json!(
        {
            "receiverId":  contract_id,
            "action_kind": "FunctionCall",
            "functionCalls": [
                {
                    "methodName": "cancel_mint_request",
                    "args": {
                        "tweet_id": tweet_id,
                    },
                    "gas": "100000000000000",
                },
            ],
        }
    ))
}
