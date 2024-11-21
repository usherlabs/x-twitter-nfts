pub mod entity;

pub mod helper;
pub mod methods;

use crate::helper::near::{process_near_transaction, NearExplorerIndexer};
use crate::helper::twitter::OathTweeterHandler;
use async_std::task::sleep;
use dotenv::dotenv;
use ethers::utils::hex;
use methods::VERIFY_ELF;
use near_client::client::NearClient;
use reqwest::Url;
use sea_orm::Database;
use sha256::digest;

use std::{env, str::FromStr, time::Duration};
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

    let indexer = NearExplorerIndexer::new(&nft_contract_id);
    if indexer.is_err() {
        error!("indexer-init-error: {:?}", indexer.err());
        return;
    }
    let twitter_client = OathTweeterHandler::default();

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
                // if exists {
                //     break;
                // }
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
