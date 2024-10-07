mod helper;
use std::env;

use dotenv::dotenv;
use sea_orm::Database;
use tracing::{debug,error};

#[async_std::main]
async fn main() {
    // Load .env
    dotenv().expect("Error occurred when loading .env");


    let nft_contract_id=env::var("NFT_CONTRACT_ID").unwrap_or("test-usher.testnet".to_owned());
    let db = Database::connect(env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .unwrap();

    let indexer=helper::NearExplorerIndexer::new(&nft_contract_id);
    if indexer.is_err(){
        error!("indexer-init-error: {:?}",indexer.err());
        return ;
    }

    let data=indexer.unwrap().get_transactions().await;
    if data.is_err(){
        error!("init-fetch-error: {:?}",data.err());
        return ;
    }
    println!("{db:?}\n{:?}",data);


    

    println!("===== =====\n");
}