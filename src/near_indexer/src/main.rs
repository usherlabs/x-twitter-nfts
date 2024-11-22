pub mod entity;

pub mod helper;

use async_std::task::sleep;
use aurora::TxSender;
use dotenv::dotenv;
use entity::near_transaction;
use ethers::utils::hex;
use helper::*;
use methods::VERIFY_ELF;
use near::{extract_metadata_from_request, verify_near_proof};
use near_client::client::NearClient;
use near_client::prelude::{AccountId, Finality};
use proof::{generate_groth16_proof, get_proof};
use reqwest::Url;
use sea_orm::ActiveValue::Set;
use sea_orm::Database;
use sea_orm::{DbConn, DbErr, EntityTrait};
use serde_json::{json, Error as SJError};
use sha256::digest;
use std::str::FromStr;
use std::{env, time::Duration};
use tokio::task::spawn_blocking;
use tracing::{debug, error, info};

#[async_std::main]
async fn main() {
    // Load .env
    dotenv().expect("Error occurred when loading .env");

    //Load Essential for env Variables
    env::var("TWEET_BEARER").expect("TWEET_BEARER must be set");

    let nft_contract_id = env::var("NFT_CONTRACT_ID").unwrap_or("local-nft.testnet".to_owned());
    let db = Database::connect(env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .unwrap();

    let near_rpc = env::var("NEAR_RPC").unwrap_or("https://rpc.testnet.near.org/".to_owned());

    // Init Near Client
    let client = NearClient::new(Url::from_str(&near_rpc).unwrap()).unwrap();

    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    info!(
        "\n\nELF SHA\t\t:{}\n\n,",
        digest(format!("{}", hex::encode(VERIFY_ELF)))
    );

    let indexer = indexer::NearExplorerIndexer::new(&nft_contract_id);
    if indexer.is_err() {
        error!("indexer-init-error: {:?}", indexer.err());
        return;
    }
    let twitter_client = twitter::OathTweeterHandler::default();

    let mut indexer = indexer.unwrap();
    loop {
        let mut data = indexer.get_transactions().await;

        if data.is_err() {
            error!("init-fetch-error: {:?}", data.err());
            return;
        }

        loop {
            let transactions = data.unwrap();
            debug!("Found {} Transactions", transactions.len());
            for transaction in transactions {
                println!("{transaction:?}");
                let exists = process_near_transaction(&db, &transaction, &client, &twitter_client)
                    .await
                    .unwrap();
                if exists {
                    break;
                }
            }
            println!("Page Number: {}", indexer.page_no);
            // Walk pages
            if !indexer.has_next_page() {
                println!("All transaction indexed");
                break;
            }
            data = indexer.next_page().await;
        }

        // wait 60 seconds
        println!("wait 60 secs");
        sleep(Duration::from_secs(60)).await;
    }
}

pub async fn process_near_transaction(
    db: &DbConn,
    transaction: &JSONTransaction,
    client: &NearClient,
    notifier: &twitter::OathTweeterHandler,
) -> Result<bool, DbErr> {
    let nft_contract_id = env::var("NFT_CONTRACT_ID").unwrap_or("local-nft.testnet".to_owned());
    let nft_contract_id = AccountId::from_str(&nft_contract_id).unwrap();

    let pk = transaction.id.parse::<i32>().unwrap();

    // Find by primary key
    let _near_transaction: Option<near_transaction::Model> =
        near_transaction::Entity::find_by_id(pk).one(db).await?;
    match _near_transaction {
        Some(_) => Ok(true),
        None => {
            if !transaction.outcomes.status {
                debug!(
                    "Found Failed BlockChain Transaction Ignored, {}",
                    transaction.transaction_hash
                );
                return Ok(false);
            }
            if transaction.actions[0].method.is_none() {
                debug!(
                    "Ignored Transaction: {} No method Found",
                    transaction.transaction_hash
                );
                return Ok(false);
            }
            let action = match transaction.actions.get(0) {
                Some(action) => action,
                None => {
                    &(JSONAction {
                        action: "".to_string(),
                        method: None,
                        args: None,
                    })
                }
            };

            if let Some(method) = &action.method {
                // LIST OPERATIONS
                if method == "mint_tweet_request" {
                    let mint_data: Result<MintRequestData, SJError> = serde_json::from_str(
                        action.args.clone().unwrap_or("".to_string()).as_str(),
                    );
                    if mint_data.is_err() {
                        debug!(
                            "mint_tweet_request: Could not parse data :{}",
                            transaction.transaction_hash
                        );
                        return Ok(false);
                    }
                    let mint_data = mint_data.unwrap();

                    let fetched_nft = client
                        .view::<Option<NftData>>(
                            &nft_contract_id,
                            Finality::Final,
                            "nft_token",
                            Some(json!({
                                    "token_id": mint_data.tweet_id.clone(),
                            })),
                        )
                        .await;
                    if fetched_nft.is_err() {
                        info!(
                            "Failed to fetched nft data at :{}",
                            transaction.transaction_hash
                        );
                        return Ok(false);
                    }

                    let fetched_nft = fetched_nft.unwrap().data();

                    if fetched_nft.is_some() {
                        debug!("NFT already minted :{}", transaction.transaction_hash);
                        return Ok(false);
                    }
                    debug!("fetched_nft: {:?}\nmint_data:{:?}", fetched_nft, &mint_data);

                    // Generate Proof
                    let proof = get_proof(mint_data.tweet_id.clone()).await;

                    if proof.is_err() {
                        info!("Invalid Tweet ID{}\n", &mint_data.tweet_id);
                        return Ok(false);
                    }
                    let (proof, tweet_res_data) = proof.unwrap();

                    let meta_data = AssetMetadata {
                        image_url: mint_data.image_url.to_string(),
                        owner_account_id: transaction.signer_account_id.clone(),
                        token_id: mint_data.tweet_id.clone(),
                    };

                    let zk_input = ZkInputParam {
                        proof,
                        meta_data: meta_data.clone(),
                    };
                    debug!("{:?}", &zk_input);

                    let (seal, journal_output) =
                        spawn_blocking(|| generate_groth16_proof(zk_input))
                            .await
                            .unwrap();

                    info!(
                        "{:?} was committed to the journal",
                        hex::encode(&journal_output)
                    );
                    info!("{:?} was the provided seal", hex::encode(&seal));
                    let aurora_client = TxSender::default();
                    let aurora_tx_future = aurora_client
                        .verify_proof_on_aurora(journal_output.clone(), seal)
                        .await;
                    let aurora_tx_response = aurora_tx_future.unwrap();
                    info!(
                        "Aurora transation has been verified with response: {:?}\n",
                        aurora_tx_response
                    );

                    let chain_transaction = aurora_tx_response.block_hash.unwrap();
                    debug!(
                        "ON Chain Transaction Hash {} for mint {}",
                        chain_transaction.to_string(),
                        transaction.transaction_hash
                    );

                    // perform verification near
                    // mint NFT if near verification is successfull
                    let near_tx_response = verify_near_proof(
                        journal_output,
                        extract_metadata_from_request(tweet_res_data, meta_data),
                    )
                    .await;
                    if near_tx_response.is_err() {
                        info!(
                            "Failed to mint {}\n {:?}\n",
                            &mint_data.tweet_id,
                            near_tx_response.err()
                        );
                        return Ok(false);
                    }
                    let near_tx_response = near_tx_response.expect("successful mint response");
                    debug!(
                        "Near transaction has been verified with response: {:?}\n",
                        near_tx_response
                    );

                    let new_transaction = near_transaction::ActiveModel {
                        id: Set(pk),
                        transaction_hash: Set(transaction.transaction_hash.clone()),
                        signer_account_id: Set(transaction.signer_account_id.clone()),
                        receiver_account_id: Set(transaction.receiver_account_id.clone()),
                        block_timestamp: Set(transaction.transaction_hash.clone()),
                        block_height: Set(transaction.block.block_height.try_into().unwrap()),
                        action: Set(action.action.clone()),
                        method: Set(method.clone()),
                        outcomes_status: Set(transaction.outcomes.status),
                        tweet_id: Set(mint_data.tweet_id.clone().to_string()),
                        image_url: Set(mint_data.image_url.clone()),
                        user_to_notify: Set(Some(mint_data.notify.clone())),
                        mint_transaction_hash: Set(Some(chain_transaction.to_string())),
                        ..Default::default() // all other attributes are `NotSet`
                    };
                    near_transaction::Entity::insert(new_transaction)
                        .exec(db)
                        .await?;

                    if !mint_data.notify.is_empty() {
                        let _ = notifier
                            .notifier(&mint_data.tweet_id.clone(), &mint_data.notify)
                            .await;
                    }
                    return Ok(false);
                }
            } else {
                debug!(
                    "Ignored Transaction: {} Method Called: {:?}",
                    transaction.transaction_hash, action.method
                );
                return Ok(false);
            }
            Ok(false)
        }
    }
}
