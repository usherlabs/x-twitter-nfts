use std::error::Error;
use std::marker::{Send, Sync};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

use super::{JSONTransaction, TransactionData};

fn parse_string_to_i64(opt_str: Option<String>) -> Option<i64> {
    opt_str.map(|s| s.parse::<i64>().unwrap_or(0))
}


pub struct NearExplorerIndexer<'a> {
    pub cursor: Option<i64>,
    pub account_id: &'a str,
    pub near_block_key: &'a str,
}

impl<'a> NearExplorerIndexer<'a> {
    pub fn new(account_id: &'a str, near_block_key: &'a str, cursor: Option<i64>) -> Result<Self, Box<dyn Error>> {
        if !(account_id.ends_with(".testnet") || account_id.ends_with(".near")) {
            return Err("Invalid account_id".into());
        }

        Ok(NearExplorerIndexer {
            cursor,
            account_id: &account_id,
            near_block_key,
        })
    }

    /// Retrieves the Latest Batch of transactions.
    ///
    /// This method continues fetching transactions from latest
    ///
    /// # Returns
    /// A vector of transactions representing the next page.
    pub async fn get_transactions(
        &mut self,
    ) -> Result<Vec<JSONTransaction>, Box<dyn Error + Send + Sync>> {
        self.cursor = None;
        let data = self.fetch().await?;

        if data.cursor.is_some() {
            self.cursor = parse_string_to_i64(data.cursor);
        }

        Ok(data.txns)
    }

    /// Retrieves the next page of transactions.
    ///
    /// This method continues fetching transactions from where the previous call left off.
    ///
    /// # Returns
    /// A vector of transactions representing the next page.
    pub async fn next_page(
        &mut self,
    ) -> Result<Vec<JSONTransaction>, Box<dyn Error + Send + Sync>> {
        let data = self.fetch().await?;

        if let Some(cursor) = data.cursor {
            self.cursor = Some(cursor.parse::<i64>().unwrap_or(0));
        } else {
            self.cursor = None;
        }

        Ok(data.txns)
    }

    pub fn has_next_page(&self) -> bool {
        self.cursor.is_some()
    }
    async fn fetch(
        &mut self
    ) -> Result<TransactionData, Box<dyn Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let base = if self.account_id.ends_with(".testnet") {
            "https://api-testnet.nearblocks.io"
        } else {
            "https://api.nearblocks.io"
        };

        let max_retries = 3;
        let mut retries = 0;
        loop {
            let url = format!(
                "{}/v1/account/{}/txns-only?cursor={}&order=asc",
                base, self.account_id, self.cursor.unwrap_or(0)
            );
            let response = client.get(&url).header("Authorization", self.near_block_key).send().await?;

            info!("{}\n{}", (&response).status(),&url);

            if (&response).status() == 200 {
                match response.json::<TransactionData>().await {
                    Ok(_output) => {
                        return Ok(_output);
                    }
                    Err(e) => {
                        if retries >= max_retries {
                            return Err(format!("FETCH_ERROR: {}, {}", e, &url).into());
                        }
                    }
                }
            }
            retries += 1;
            let delay = Duration::from_secs(100);
            sleep(delay).await;
        }
    }
}

#[cfg(test)]
mod test_near_explorer_indexer {
    use std::env;

    use dotenv::dotenv;

    use super::*;

    #[tokio::test]
    async fn test_near_explorer_indexer() {
        dotenv().expect("Error occurred when loading .env");

        let near_block_key = env::var("NEAR_BLOCK_KEY").expect("NEAR_BLOCK_KEY must be set");
        let mut indexer = NearExplorerIndexer::new("priceoracle.near", &near_block_key,Some(0)).unwrap();
        assert_eq!(indexer.get_transactions().await.unwrap().len(), 25);
        assert_eq!(indexer.has_next_page(), true);
    }

    #[tokio::test]
    async fn test_near_explorer_indexer_testnet() {
        dotenv().expect("Error occurred when loading .env");
        let near_block_key = env::var("NEAR_BLOCK_KEY").expect("NEAR_BLOCK_KEY must be set");
        let mut indexer = NearExplorerIndexer::new("ush_test.testnet", &near_block_key,Some(0)).unwrap();
        let data_size = indexer.get_transactions().await.unwrap().len();
        assert!(data_size < 25);
        assert_eq!(indexer.has_next_page(), false);
    }
}
