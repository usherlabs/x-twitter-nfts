pub mod entity;
mod helper;

use async_std::task::sleep;
use dotenv::dotenv;
use helper::process_near_transaction;
use near_client::{client::NearClient, prelude::Ed25519SecretKey};
use reqwest::Url;
use sea_orm::Database;
use std::{env, str::FromStr, time::Duration};
use tracing::{debug, error};
#[async_std::main]
async fn main() {
    // Load .env
    dotenv().expect("Error occurred when loading .env");

    //Load Essential for env Variables
    let nft_contract_id = env::var("NFT_CONTRACT_ID").unwrap_or("test-usher.testnet".to_owned());
    let db = Database::connect(env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .unwrap();

    let near_rpc = env::var("NEAR_RPC").unwrap_or("https://rpc.testnet.near.org/".to_owned());
    let sk = env::var("SIGNER_SK").expect("SIGNER_SK Must be set");
    let ed25519_secret_key = Ed25519SecretKey::from_expanded(&sk).unwrap();

    // Init Near Client
    let client = NearClient::new(Url::from_str(&near_rpc).unwrap()).unwrap();

    let indexer = helper::NearExplorerIndexer::new(&nft_contract_id);
    if indexer.is_err() {
        error!("indexer-init-error: {:?}", indexer.err());
        return;
    }

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
                let exists =
                    process_near_transaction(&db, &transaction, &client, &ed25519_secret_key)
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
