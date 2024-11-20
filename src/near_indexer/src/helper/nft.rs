use super::near::AssetMetadata;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use serde::{Deserialize, Serialize};
use serde_json::json;
/// The tweet structure gotten from the API
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize)]
pub struct TweetResponse {
    /// data
    pub data: Option<Vec<TweetData>>,
    /// users info
    pub includes: Option<Includes>,

    errors: Option<Vec<ErrorObject>>,
}

#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Debug, Deserialize, Serialize)]
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

/// generate the nft payload
pub fn extract_metadata_from_request(
    tweet: TweetResponse,
    meta_data: AssetMetadata,
) -> TokenMetadata {
    let tweet_data = tweet.data.expect("REASON");
    let tweet_data = tweet_data.get(0).unwrap();
    let public_metric = &tweet_data.public_metrics;

    // generate a token metadata
    let token_metadata = TokenMetadata {
        title: Some(tweet_data.id.clone()), // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
        description: Some(tweet_data.text.clone()), // free-form description
        extra: Some(
            json!({"public_metric": public_metric, "minted_to":meta_data.owner_account_id.clone() })
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

    token_metadata
}
