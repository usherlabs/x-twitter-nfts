use near_jsonrpc_client::methods::query::RpcQueryRequest;
use near_jsonrpc_client::methods::tx::RpcTransactionResponse;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::TransactionInfo;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::BlockReference;
use near_primitives::views::{QueryRequest, TxExecutionStatus};
use serde_json::json;
use verity_verify_remote::{
    config::Config,
    ic::Verifier,
};


use crate::helper::proof::get_verity_client;
use crate::helper::SETTINGS;

pub async fn verify_near_proof_v2(
    tweet_id: String,
    image_url: String,
    nft_owner: String,
) -> Result<(RpcTransactionResponse, String), Box<dyn std::error::Error>> {

    // grab your pre‑loaded config
    let cfg = &*SETTINGS;

    println!("Proving a GET request using VerityClient...");

    let client = get_verity_client();

    let result = client
        .get(
            format!("https://api.x.com/2/tweets?ids={}&tweet.fields=created_at,public_metrics&expansions=author_id&user.fields=created_at", tweet_id)
        )
        .header(
            "Authorization",
            format!("Bearer {}", cfg.tweet_bearer.clone())
        ).redact("req:header:authorization".to_string())
        .send().await;

    let response = result.expect("successful response");

    let json: serde_json::Value = response.subject.json().await.unwrap();
    println!("json: {:#?}", json);
    println!("response.proof.len(): {:#?}", response.proof.len());

    // Get the Notary information from the Prover
    let notaryinfo = client.get_notary_info().await;

    let notary_pub_key = notaryinfo.expect("success").public_key;



    let rv_identity_path = &cfg.rv_identity_file;

    println!("notary_pub_key: {:#?}\n", notary_pub_key);

    let rv_config = Config::new(
        cfg.verity_ic_gateway.clone(),
        rv_identity_path.to_string(),
        cfg.verity_ic_id.to_string()
    );

    let remote_verifier = Verifier::from_config(&rv_config).await.unwrap();


    let proof_value: serde_json::Value = serde_json::from_str(&response.proof).unwrap();
    let session = proof_value.to_string();

    let verified_by_remote = remote_verifier
        .verify_proof(
            // You can verify multiple proofs at once
            vec![session],
            notary_pub_key,
        )
        .await
        .unwrap();


    
    let signer = near_crypto::InMemorySigner::from_secret_key(cfg.signer_account_id.clone(), cfg.signer_secret_key.clone());
    
    
    let client = JsonRpcClient::connect(cfg.near_rpc_url.clone());
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
    
    
    
    let transaction = Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce: current_nonce + 1,
        receiver_id: cfg.contract_account_id.clone(),
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

     println!("proof:{:?}", &response);
    
    Ok((response.unwrap(), tx_hash.to_string()))
}
