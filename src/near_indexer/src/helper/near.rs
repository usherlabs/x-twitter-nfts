use super::{AssetMetadata, TweetResponse, User};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_jsonrpc_client::methods::query::RpcQueryRequest;
use near_jsonrpc_client::methods::tx::RpcTransactionResponse;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::{RpcTransactionError, TransactionInfo};
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::BlockReference;
use near_primitives::views::{QueryRequest, TxExecutionStatus};
use serde_json::json;
use std::env;
use tokio::time;
use tracing::debug;

pub async fn verify_near_proof(
    journal_output: Vec<u8>,
    token_metadata: TokenMetadata,
) -> Result<RpcTransactionResponse, Box<dyn std::error::Error>> {
    let rpc_url = env::var("NEAR_RPC_URL").expect("RPC_URL_NOT_PRESENT");
    let account_id = env::var("NEAR_SIGNER_ACCOUNT_ID").expect("ACCOUNT_ID_NOT_PRESENT");
    let secret_key = env::var("NEAR_ACCOUNT_SECRET_KEY").expect("SECRET_KEY_NOT_PRESENT");
    let contract_account_id =
        env::var("NEAR_VERIFIER_CONTRACT_ACCOUNT_ID").expect("CONTRACT_ACCOUNT_ID_NOT_PRESENT");

    let signer_account_id: near_primitives::types::AccountId = account_id.parse()?;
    let signer_secret_key: near_crypto::SecretKey = secret_key.parse()?;

    let signer = near_crypto::InMemorySigner::from_secret_key(signer_account_id, signer_secret_key);

    let client = JsonRpcClient::connect(rpc_url);
    let access_key_query_response = client
        .call(RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        })
        .await?;

    let current_nonce = match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ => Err("failed to extract current nonce")?,
    };

    let transaction = Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce: current_nonce + 1,
        receiver_id: contract_account_id.parse()?,
        block_hash: access_key_query_response.block_hash,
        actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
            method_name: "verify_proof".to_string(),
            args: json!({
                "journal": journal_output,
                "token_metadata": token_metadata
            })
            .to_string()
            .into_bytes(),
            gas: 100_000_000_000_000, // 100 TeraGas
            deposit: 0,
        }))],
    };

    let request = methods::broadcast_tx_async::RpcBroadcastTxAsyncRequest {
        signed_transaction: transaction.sign(&signer),
    };

    let sent_at = time::Instant::now();
    let tx_hash = client.call(request).await?;
    let res: Option<RpcTransactionResponse>;
    loop {
        let response = client
            .call(methods::tx::RpcTransactionStatusRequest {
                transaction_info: TransactionInfo::TransactionId {
                    tx_hash,
                    sender_account_id: signer.account_id.clone(),
                },
                wait_until: TxExecutionStatus::Executed,
            })
            .await;
        let received_at = time::Instant::now();
        let delta = (received_at - sent_at).as_secs();

        if delta > 60 {
            Err("time limit exceeded for the transaction to be recognized")?;
        }

        match response {
            Err(err) => match err.handler_error() {
                Some(
                    RpcTransactionError::TimeoutError
                    | RpcTransactionError::UnknownTransaction { .. },
                ) => {
                    time::sleep(time::Duration::from_secs(2)).await;
                    continue;
                }
                _ => Err(err)?,
            },
            Ok(response) => {
                debug!("response gotten after: {}s", delta);
                res = Some(response);
                break;
            }
        }
    }

    Ok(res.unwrap())
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

    token_metadata
}

#[cfg(test)]
mod tests {
    use crate::helper::{
        aurora::TxSender,
        proof::{generate_groth16_proof, get_proof},
        ZkInputParam,
    };

    use super::*;
    use dotenv::dotenv;
    use ethers::utils::hex;
    use tokio::{self, task::spawn_blocking};

    #[tokio::test]
    async fn test_get_verify() {
        dotenv().expect("Error occurred when loading .env");
        let tweet_id: u64 = 1858184885493485672;
        let proof = get_proof(tweet_id.to_string()).await;
        if proof.is_err() {
            debug!("{:?}", proof.err());
        } else {
            let (proof, _) = proof.unwrap();

            assert!(proof.len() > 100, "proof is invalid");

            let meta_data= AssetMetadata{
            image_url: "https://386f4b0d6749763bc7ab0a648c3e650f.ipfscdn.io/ipfs/QmXPD7KqFyFWwMTQyEo9HuTJjkKLxergS1YTt1wjJNAAHV".to_string(),
            owner_account_id:"xlassixx.testnet".to_string(),
            token_id: tweet_id.to_string(),
        };

            let zk_input = ZkInputParam {
                proof: proof,
                meta_data,
            };

            let (seal, journal_output) = spawn_blocking(|| generate_groth16_proof(zk_input))
                .await
                .unwrap();

            debug!(
                "{:?} was committed to the journal",
                hex::encode(&journal_output)
            );
            debug!("{:?} was the provided seal", hex::encode(&seal));
            let aurora_client = TxSender::default();
            let aurora_tx_future = aurora_client
                .verify_proof_on_aurora(journal_output.clone(), seal)
                .await;
            let aurora_tx_response = aurora_tx_future.unwrap();
            debug!(
                "Aurora transation has been verified with resriscponse: {:?}\n",
                aurora_tx_response
            );
            assert!(hex::check_raw(format!(
                "{:x}",
                aurora_tx_response.block_hash.unwrap()
            )));
        }
    }
}
