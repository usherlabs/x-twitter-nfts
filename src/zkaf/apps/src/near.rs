use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_jsonrpc_client::methods::tx::RpcTransactionResponse;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::{RpcTransactionError, TransactionInfo};
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::{ BlockReference, Finality, FunctionArgs};
use near_primitives::views::{QueryRequest, TxExecutionStatus};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;

use serde_json::json;
use tokio::time;
use serde_json::from_slice;
use std::env;

// TODO
// ----- write a function to fetch/query the NFT contract 
// make the contract call fail when it actually fails
// fix twitter verifier
// conjoin all the code in one repo
// clean up all code

// monday potentially: close out tickets
pub async fn get_nft_by_id(token_id: TokenId) -> Result<Token, Box<dyn std::error::Error>> {
    let rpc_url = env::var("NEAR_RPC_URL").expect("RPC_URL_NOT_PRESENT");
    let contract_account_id = env::var("NEAR_NFT_CONTRACT_ACCOUNT_ID").expect("CONTRACT_ACCOUNT_ID_NOT_PRESENT");

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
    }else {
        Err("INVALID RESPONSE".into())
    }
}

pub async fn verify_near_proof(journal_output: Vec<u8>, token_metadata: TokenMetadata) -> Result<RpcTransactionResponse, Box<dyn std::error::Error>> {
    let rpc_url = env::var("NEAR_RPC_URL").expect("RPC_URL_NOT_PRESENT");
    let account_id = env::var("NEAR_SIGNER_ACCOUNT_ID").expect("ACCOUNT_ID_NOT_PRESENT");
    let secret_key = env::var("NEAR_ACCOUNT_SECRET_KEY").expect("SECRET_KEY_NOT_PRESENT");
    let contract_account_id = env::var("NEAR_VERIFIER_CONTRACT_ACCOUNT_ID").expect("CONTRACT_ACCOUNT_ID_NOT_PRESENT");
    
    let signer_account_id: near_primitives::types::AccountId = account_id.parse()?;
    let signer_secret_key: near_crypto::SecretKey = secret_key.parse()?;
    
    let signer = near_crypto::InMemorySigner::from_secret_key(signer_account_id, signer_secret_key);
    
    let client = JsonRpcClient::connect(rpc_url);
    let access_key_query_response = client
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
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
                println!("response gotten after: {}s", delta);
                res = Some(response);
                break;
            }
        }
    }

    Ok(res.unwrap())
}
