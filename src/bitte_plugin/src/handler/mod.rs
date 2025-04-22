use serde::{Deserialize, Serialize};

pub mod catcher_handler;
pub mod open_api_handler;
pub mod tweet;
mod utils;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct PluginInfo {
    // pluginId: String,
    url: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct IpfsData {
    Hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct PublicMetric {
    bookmark_count: u128,
    impression_count: u128,
    like_count: u128,
    quote_count: u128,
    reply_count: u128,
    retweet_count: u128,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TweetResponse {
    /// data
    pub data: Option<Vec<TweetData>>,
    /// users info
    pub includes: Includes,

    pub errors: Option<Vec<ErrorObject>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
#[derive(Debug, Deserialize, Serialize, Clone)]
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
#[derive(Debug, Deserialize, Serialize, Clone)]
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
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Includes {
    /// users
    pub users: Vec<User>,
}

/// The User substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize, Clone)]
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