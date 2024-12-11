use sha256::digest;
use std::io::Read;

use alloy_sol_types::SolValue;
use risc0_zkvm::guest::env;

use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use serde::{Deserialize, Serialize};
use serde_json::json;
use verity_verifier::verify_proof;

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
#[derive(Debug, Deserialize, Serialize)]
pub struct Tweet {
    /// data
    pub data: Vec<Data>,
    /// users info
    pub includes: Includes,
}

/// The data substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    /// date created
    pub created_at: String,
    /// id
    pub id: String,
    /// public_metrics
    pub public_metrics: PublicMetrics,
    /// edit_history_tweet_ids
    pub edit_history_tweet_ids: Vec<String>,
    /// author_id of tweet
    pub author_id: String,
    /// tweet text
    pub text: String,
}

/// The PublicMetrics substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
pub struct Includes {
    /// users
    pub users: Vec<User>,
}

/// The User substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize)]
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

pub fn generate_tweet_nft_payload(
    response_http_string: String,
    meta_data: AssetMetadata,
) -> (TokenMetadata, String) {
    let lines: Vec<&str> = response_http_string.split("\n").collect();

    // the json string is the last line in the http payload
    let json_tweet = lines.last().unwrap().to_owned();

    print!("data:{}", json_tweet);

    // get the tweet and the public metric to be stringified
    let tweet: Tweet = serde_json::from_str(json_tweet).unwrap();
    let tweet_data = tweet.data.get(0).unwrap();
    let public_metric = &tweet_data.public_metrics;

    // generate a token metadata
    let token_metadata = TokenMetadata {
        title: Some(tweet_data.id.clone()), // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
        description: Some(tweet_data.text.clone()), // free-form description
        extra: Some(
            json!({
               "public_metric": public_metric,
               "minted_to":meta_data.owner_account_id.clone(),
               "author_id":tweet_data.author_id.clone(),
               "user": (tweet.includes.users.get(0).unwrap_or(&User{
                    name: "".to_string(),
                    id:"".to_string(),
                    username: Some("".to_string()),
                    created_at:"".to_string()
                })).username
            })
            .to_string(),
        ), // anything extra the NFT wants to store on-chain. Can be stringified JSON.
        media: Some(meta_data.image_url), // URL to associated media, preferably to decentralized, content-addressed storage
        media_hash: None, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
        copies: Some(1), // number of copies of this set of metadata in existence when token was minted.
        issued_at: None, // ISO 8601 datetime when token was issued or minted
        expires_at: None, // ISO 8601 datetime when token expires
        starts_at: None, // ISO 8601 datetime when token starts being valid
        updated_at: None, // ISO 8601 datetime when token was last updated
        reference: None, // URL to an off-chain JSON file with more info.
        reference_hash: None,
    };

    let stringified_token_metadata = serde_json::to_string(&token_metadata).unwrap();

    (token_metadata, stringified_token_metadata)
}

fn main() {
    // Read the input data for this application.
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();

    let proof_params: String = String::from_utf8(input_bytes).unwrap();
    let proof_params: ZkInputParam = serde_json::from_str(proof_params.as_str()).unwrap();

    let (response, _request) = verify_proof(&proof_params.proof).unwrap();
    let (_, string_metadata) = generate_tweet_nft_payload(response, proof_params.meta_data);

    env::log(&format!("Derived metadata: {}", string_metadata));

    let metadata_hash = digest(string_metadata);
    let encoded_metadata_hash = hex::decode(metadata_hash).unwrap();

    env::log("committing results to journal");
    // Commit the journal that will be received by the application contract.
    // Journal is encoded using Solidity ABI for easy decoding in the app contract.
    env::commit_slice(encoded_metadata_hash.abi_encode().as_slice());
}
