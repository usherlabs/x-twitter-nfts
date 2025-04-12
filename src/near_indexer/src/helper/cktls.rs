use near_jsonrpc_client::methods::query::RpcQueryRequest;
use near_jsonrpc_client::methods::tx::RpcTransactionResponse;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::TransactionInfo;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::BlockReference;
use near_primitives::views::{QueryRequest, TxExecutionStatus};
use serde_json::json;
use std::env;
use verity_verify_remote::{
    config::Config,
    ic::{Verifier, DEFAULT_IC_GATEWAY_LOCAL},
};

use verity_verify_tls::verify_proof;

use crate::helper::proof::get_verity_client;

pub const DEFAULT_VERITY_VERIFIER_ID: &str = "bkyz2-fmaaa-aaaaa-qaaaq-cai";

pub async fn verify_near_proof_v2(
    tweet_id: String,
    image_url: String,
    nft_owner: String,
) -> Result<(RpcTransactionResponse, String), Box<dyn std::error::Error>> {
    println!("Proving a GET request using VerityClient...");

    let client = get_verity_client();

    let result = client
        .get(
            format!("https://api.x.com/2/tweets?ids={}&tweet.fields=created_at,public_metrics&expansions=author_id&user.fields=created_at", tweet_id)
        )
        .header(
            "Authorization",
            format!("Bearer {}", env::var("TWEET_BEARER").unwrap_or(String::from("")))
        ).redact("req:header:authorization".to_string())
        .send().await;

    let response = result.expect("successful response");

    let json: serde_json::Value = response.subject.json().await.unwrap();
    println!("json: {:#?}", json);
    println!("response.proof.len(): {:#?}", response.proof.len());

    // Get the Notary information from the Prover
    let notaryinfo = client.get_notary_info().await;
    println!("notaryinfo: {:#?}", notaryinfo);

    let notary_pub_key = notaryinfo.expect("success").public_key;

    let verified_by_host: (String, String) =
        verify_proof(&response.proof, &notary_pub_key).unwrap();

    println!("verified_by_host: {:#?}", verified_by_host);
    // Perform the partial remote verification against decentralised compute

    // 1. Create a config file by specifying the params
    // ? To optain this identity.pem, use `dfx identity export` - https://internetcomputer.org/docs/current/developer-docs/developer-tools/cli-tools/cli-reference/dfx-parent

    // TODO: This should eventually be abstracted away from the user...
    let rv_identity_path = "./src/fixtures/identity.pem";
    let rv_id = DEFAULT_VERITY_VERIFIER_ID.to_string();
    let rv_config = Config::new(
        DEFAULT_IC_GATEWAY_LOCAL.to_string(),
        rv_identity_path.to_string(),
        rv_id,
    );

    // 2. Create verifier from a config file
    let remote_verifier = Verifier::from_config(&rv_config).await.unwrap();

    // 3. Extract our the public/private sub-proofs
    let proof_value: serde_json::Value = serde_json::from_str(&response.proof).unwrap();
    let session = proof_value.to_string();

    // 4. Verify a proof and get the response
    let verified_by_remote = remote_verifier
        .verify_proof(
            // You can verify multiple proofs at once
            vec![session],
            notary_pub_key,
        )
        .await
        .unwrap();

    let rpc_url = env::var("NEAR_RPC_URL").expect("RPC_URL_NOT_PRESENT");
    let account_id = env::var("NEAR_SIGNER_ACCOUNT_ID").expect("ACCOUNT_ID_NOT_PRESENT");
    let secret_key = env::var("NEAR_ACCOUNT_SECRET_KEY").expect("SECRET_KEY_NOT_PRESENT");
    let contract_account_id = env::var("NEAR_VERIFIER_CONTRACT_ACCOUNT_ID")
        .expect("NEAR_VERIFIER_CONTRACT_ACCOUNT_ID_NOT_PRESENT");

    let signer_account_id: near_primitives::types::AccountId =
        account_id.parse().expect("signer_account_id");
    let signer_secret_key: near_crypto::SecretKey = secret_key.parse().expect("signer_secret_key");

    let signer = near_crypto::InMemorySigner::from_secret_key(signer_account_id, signer_secret_key);

    println!(
        "signer:{:?} public_key:{:?}",
        signer.account_id.clone(),
        signer.public_key.clone()
    );

    let client = JsonRpcClient::connect(rpc_url);
    let access_key_query_response = client
        .call(RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        })
        .await
        .expect("access_key_query_response error");

    let current_nonce = match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ => 0,
    };

    println!("proof:{:?}", &verified_by_remote.results[0]);

    let transaction = Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce: current_nonce + 1,
        receiver_id: contract_account_id.parse().expect("contract_account_id"),
        block_hash: access_key_query_response.block_hash,
        actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
            method_name: "verify_proof_v2".to_string(),
            args: json!({
                "proof": verified_by_remote.results[0].get_content(),
                "signature": verified_by_remote.signature,
                "image_url": image_url,
                "owner_address":nft_owner
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
    let tx_hash = client.call(request).await?;

    let response = client
        .call(methods::tx::RpcTransactionStatusRequest {
            transaction_info: TransactionInfo::TransactionId {
                tx_hash: tx_hash,
                sender_account_id: signer.account_id.clone(),
            },
            wait_until: TxExecutionStatus::Executed,
        })
        .await;

    // loop {
    //     let response = client
    //         .call(methods::tx::RpcTransactionStatusRequest {
    //             transaction_info: TransactionInfo::TransactionId {
    //                 tx_hash,
    //                 sender_account_id: signer.account_id.clone(),
    //             },
    //             wait_until: TxExecutionStatus::Executed,
    //         })
    //         .await;
    //     let received_at = time::Instant::now();
    //     let delta = (received_at - sent_at).as_secs();

    //     if delta > 60 {
    //         Err("time limit exceeded for the transaction to be recognized")?;
    //     }

    //     match response {
    //         Err(err) => match err.handler_error() {
    //             Some(
    //                 RpcTransactionError::TimeoutError
    //                 | RpcTransactionError::UnknownTransaction { .. },
    //             ) => {
    //                 time::sleep(time::Duration::from_secs(2)).await;
    //                 continue;
    //             }
    //             _ => Err(err)?,
    //         },
    //         Ok(response) => {
    //             debug!("response gotten after: {}s", delta);
    //             res = Some(response);
    //             break;
    //         }
    //     }
    // }

    Ok((response.unwrap(), tx_hash.to_string()))
}
