use near_contract_tools::event;
use near_sdk::{AccountId, Balance};

/// Mint Request Events

/// `TweetMintRequest` is emitted when a mint request occurs.
///
/// Properties:
///
/// * `tweet_id`: The ID of the tweet associated with the mint request.
/// * `account`: The account requesting the mint operation.
#[event(standard = "custom", version = "1.0.0")]
pub struct TweetMintRequest {
    pub tweet_id: String,
    pub account: AccountId,
    pub deposit: Balance,
    pub image_url: String,
    pub notify: String,
}

#[event(standard = "custom", version = "1.0.0")]
pub struct CancelMintRequest {
    pub tweet_id: String,
    pub account: AccountId,
    pub withdraw: Balance,
}
