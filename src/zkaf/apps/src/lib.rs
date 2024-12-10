use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::QueryRequest;

use serde_json::from_slice;
use serde_json::json;
use std::env;

use indexer::helper::{AssetMetadata, TweetResponse, User};

pub async fn get_nft_by_id(token_id: TokenId) -> Result<Token, Box<dyn std::error::Error>> {
    let rpc_url = env::var("NEAR_RPC_URL").expect("RPC_URL_NOT_PRESENT");
    let contract_account_id =
        env::var("NEAR_NFT_CONTRACT_ACCOUNT_ID").expect("CONTRACT_ACCOUNT_ID_NOT_PRESENT");

    let client = JsonRpcClient::connect(rpc_url);

    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: contract_account_id.parse()?,
            method_name: "nft_token".to_string(),
            args: FunctionArgs::from(
                json!({
                    "token_id": token_id,
                })
                .to_string()
                .into_bytes(),
            ),
        },
    };

    let response = client.call(request).await.unwrap();

    if let QueryResponseKind::CallResult(result) = response.kind {
        let token = from_slice::<Option<Token>>(&result.result).unwrap();
        return Ok(token.unwrap());
    } else {
        Err("INVALID RESPONSE".into())
    }
}

/// generate the nft payload
pub fn generate_tweet_nft_payload(
    response_http_string: String,
    meta_data: AssetMetadata,
) -> (TokenMetadata, String) {
    let lines: Vec<&str> = response_http_string.split("\n").collect();

    // the json string is the last line in the http payload
    let json_tweet = lines.last().unwrap().to_owned();

    print!("data:{}", json_tweet);

    // get the tweet and the public metric to be stringified
    let tweet: TweetResponse = serde_json::from_str(json_tweet).unwrap();
    let _tweet_data = tweet.data.expect("REASON");
    let tweet_data = _tweet_data.get(0).unwrap();
    let public_metric = &tweet_data.public_metrics;

    // generate a token metadata
    let token_metadata = TokenMetadata {
        title: Some(tweet_data.id.clone()), // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
        description: Some(tweet_data.text.clone()), // free-form description
        extra: Some(
            json!(
            {
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
