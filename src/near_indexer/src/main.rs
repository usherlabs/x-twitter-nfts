pub mod entity;
pub mod generated;
pub mod helper;

use async_std::task::sleep;
use dotenv::dotenv;
use entity::near_transaction;
use ethers::utils::hex;
use helper::cktls::verify_near_proof_v2;
use helper::*;
use migration::{Migrator, MigratorTrait};
use near_client::client::NearClient;
use near_client::prelude::{AccountId, Finality};
use reqwest::Url;
use sea_orm::ActiveValue::Set;
use sea_orm::{Database, QueryOrder};
use sea_orm::{DbConn, DbErr, EntityTrait};
use serde_json::{json, Error as SJError};
use sha256::digest;
use std::str::FromStr;
use std::{env, time::Duration};
use tracing::{debug, error, info};

#[async_std::main]
async fn main() {
    // Load .env
    let tweet_bearer = env::var("TWEET_BEARER");

    if tweet_bearer.is_err() {
        dotenv().expect("Error occurred when loading .env");
    }

    //Load Essential for env Variables
    env::var("TWEET_BEARER").expect("TWEET_BEARER must be set");

    let nft_contract_id =
        env::var("NEAR_NFT_CONTRACT_ACCOUNT_ID").expect("NEAR_NFT_CONTRACT_ACCOUNT_ID must be set");
    let db = Database::connect(env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .unwrap();

    Migrator::up(&db, None).await.unwrap();
    let near_rpc = env::var("NEAR_RPC_URL").expect("NEAR_RPC_URL");

    // Init Near Client
    let client = NearClient::new(Url::from_str(&near_rpc).unwrap()).unwrap();
    let near_block_key = env::var("NEAR_BLOCK_KEY").expect("NEAR_BLOCK_KEY must be set");

    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
    
    let twitter_client = twitter::OathTweeterHandler::default();

    loop {
        let query = near_transaction::Entity::find()
            .order_by_desc(near_transaction::Column::Id)
            .one(&db)
            .await;

        if query.is_err() {
            error!("DB-error: {:?}", query.err());
            return;
        }
        let query = query.unwrap();
        let cursor = if query.is_some() {
            Some(query.clone().unwrap().id)
        } else {
            Some(0)
        };

        debug!("Cursor at:{:?}\n\n",&cursor);

        let indexer = indexer::NearExplorerIndexer::new(&nft_contract_id, &near_block_key, cursor);
        if indexer.is_err() {
            error!("indexer-init-error: {:?}", indexer.err());
            return;
        }

        let mut indexer = indexer.unwrap();
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
                let _ = process_near_transaction(&db, &transaction, &client, &twitter_client)
                    .await
                    .unwrap();
            }
            println!("cursor: {:?}", indexer.cursor);
            // Walk pages
            if !indexer.has_next_page() {
                println!("All transaction indexed");
                break;
            }
            data = indexer.next_page().await;
        }

        // wait 60 seconds
        println!("wait 300 secs");
        sleep(Duration::from_secs(300)).await;
    }
}
pub async fn process_near_transaction(
    db: &DbConn,
    transaction: &JSONTransaction,
    client: &NearClient,
    notifier: &twitter::OathTweeterHandler,
) -> Result<bool, DbErr> {
    // Get the NFT contract ID from environment variable or use default value
    let nft_contract_id =
        env::var("NEAR_NFT_CONTRACT_ACCOUNT_ID").unwrap_or("x-bitte-nfts.testnet".to_owned());
    let nft_contract_id = AccountId::from_str(&nft_contract_id).unwrap();

    // Parse the transaction ID as an integer
    let pk = transaction.id.parse::<i64>().unwrap();

    // Find the near_transaction in the database by primary key
    let _near_transaction: Option<near_transaction::Model> =
        near_transaction::Entity::find_by_id(pk).one(db).await?;

    match _near_transaction {
        Some(_) => Ok(true), // Transaction exists, return true
        None => {
            // Check if the transaction status is failed
            if transaction.outcomes.status.is_none()
                || !transaction.outcomes.status.unwrap_or(false)
            {
                debug!(
                    "Found Failed BlockChain Transaction Ignored, {}",
                    transaction.transaction_hash
                );
                return Ok(false);
            }

            if (&transaction.clone().actions).is_none() {
                debug!(
                    "Ignored Transaction: {} Invalid",
                    transaction.transaction_hash
                );
                return Ok(false);
            }
            // Check if there's no action method in the transaction
            let _action = transaction.actions.clone().expect("REASON");
            if _action[0].method.is_none() {
                debug!(
                    "Ignored Transaction: {} No method Found",
                    transaction.transaction_hash
                );
                return Ok(false);
            }

            // Extract the first action from the transaction
            let action = match _action.get(0) {
                Some(action) => action,
                None => {
                    &(JSONAction {
                        action: "".to_string(),
                        method: None,
                        args: None,
                    })
                }
            };

            // Check if the action method is "mint_tweet_request"
            if let Some(method) = &action.method {
                // LIST OPERATIONS
                if method == "mint_tweet_request" {
                    // Parse the mint data from the action arguments
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

                    // Fetch NFT data from the contract
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

                    // Check if the NFT has already been minted
                    if fetched_nft.is_some() {
                        debug!("NFT already minted :{}", transaction.transaction_hash);
                        return Ok(false);
                    }
                    debug!("fetched_nft: {:?}\nmint_data:{:?}", fetched_nft, &mint_data);

                    // send verified journal to near for the mint transaction to be triggered
                    let proof  = verify_near_proof_v2(
                        mint_data.tweet_id.clone(),
                        mint_data.image_url.to_string(),
                        transaction.signer_account_id.clone(),
                    )
                    .await;
                    if proof.is_err() {
                        info!(
                            "Failed to mint {}\n {:?}\n",
                            &mint_data.tweet_id,
                            proof.err()
                        );
                        return Ok(false);
                    }
                   let (near_tx_response, tx_hash) =proof.expect("NEAR_VERIFICATION FAILED");
                    debug!(
                        "Near transaction has been verified with response: {:?}\n",
                        near_tx_response
                    );

                    // Create a new transaction record
                    let new_transaction = near_transaction::ActiveModel {
                        id: Set(pk),
                        transaction_hash: Set(transaction.transaction_hash.clone()),
                        signer_account_id: Set(transaction.signer_account_id.clone()),
                        receiver_account_id: Set(transaction.receiver_account_id.clone()),
                        block_timestamp: Set(transaction.transaction_hash.clone()),
                        block_height: Set(transaction.block.block_height.try_into().unwrap()),
                        action: Set(action.action.clone()),
                        method: Set(method.clone()),
                        outcomes_status: Set(transaction.outcomes.status.unwrap_or(false)),
                        tweet_id: Set(mint_data.tweet_id.clone().to_string()),
                        image_url: Set(mint_data.image_url.clone()),
                        user_to_notify: Set(Some(mint_data.notify.clone())),
                        mint_transaction_hash: Set(Some(tx_hash.to_string())),
                        ..Default::default() // all other attributes are `NotSet`
                    };
                    near_transaction::Entity::insert(new_transaction)
                        .exec(db)
                        .await?;

                    // Notify the user on Twitter if specified
                    if !mint_data.notify.is_empty() {
                        let _ = notifier
                            .notifier(&mint_data.tweet_id.clone(), &mint_data.notify)
                            .await;
                    }
                    return Ok(false);
                }
            } else {
                // Log ignored transactions for other methods
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
