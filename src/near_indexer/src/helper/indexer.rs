use regex::Regex;
use std::error::Error;
use std::marker::{Send, Sync};
use std::time::Duration;
use tokio::time::sleep;

use super::{JSONTransaction, NearIndexerData, TransactionData};

pub struct NearExplorerIndexer<'a> {
    pub page_no: u8,
    pub cursor: Option<String>,
    pub account_id: &'a str,
    pub build_id: String,
}

impl<'a> NearExplorerIndexer<'a> {
    pub fn new(account_id: &'a str) -> Result<Self, Box<dyn Error>> {
        if !(account_id.ends_with(".testnet") || account_id.ends_with(".near")) {
            return Err("Invalid account_id".into());
        }

        Ok(NearExplorerIndexer {
            cursor: None,
            page_no: 0,
            build_id: (if account_id.ends_with(".testnet") {
                "0kmxltnOS1UsrfVrMd5fP"
            } else {
                "bE43kUihJPVfWqBYXxGBQ"
            })
            .to_owned(),
            account_id: &account_id,
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
        self.page_no = 0;
        self.cursor = None;
        let data = self.fetch(true).await?;

        if data.cursor.is_some() {
            self.page_no = 1;
            self.cursor = data.cursor;
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
        let data = self.fetch(false).await?;
        if let Some(cursor) = data.cursor {
            self.cursor = Some(cursor);
        } else {
            self.cursor = None;
        }
        self.page_no = self.page_no + 1;

        Ok(data.txns)
    }

    pub fn has_next_page(&self) -> bool {
        self.cursor.is_some()
    }
    async fn fetch(
        &mut self,
        reset: bool,
    ) -> Result<TransactionData, Box<dyn Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let base = if self.account_id.ends_with(".testnet") {
            "testnet.nearblocks.io"
        } else {
            "nearblocks.io"
        };

        let max_retries = 5;
        let mut retries = 0;
        loop {
            let url = if self.page_no == 0 || reset {
                format!(
                    "https://{}/_next/data/{}/en/address/{}.json",
                    base, self.build_id, self.account_id
                )
            } else {
                format!(
                    "https://{}/_next/data/{}/en/address/{}.json?cursor={}&p={}",
                    base,
                    self.build_id,
                    self.account_id,
                    self.cursor.clone().unwrap_or("".to_owned()),
                    self.page_no
                )
            };
            let response = client.get(&url).send().await?;

            if (&response).status() == 404 {
                let html_text = response.text().await.unwrap();
                let re = Regex::new(r"_next/static/([a-zA-Z0-9\-_]+)/_buildManifest").unwrap();

                // Attempt to find captures in the text
                if let Some(captures) = re.captures(&html_text) {
                    // Extract the first capture group, which is our build number
                    if let Some(build_number) = captures.get(1) {
                        self.build_id = build_number.as_str().to_owned();
                    }
                }
            } else {
                match response.json::<NearIndexerData>().await {
                    Ok(output) => {
                        return Ok(output.pageProps.data);
                    }
                    Err(e) => {
                        if retries >= max_retries {
                            return Err(format!("FETCH_ERROR: {}, {}", e, &url).into());
                        }
                    }
                }
            }
            retries += 1;
            let delay = Duration::from_millis(5000);
            sleep(delay).await;
        }
    }
}

#[cfg(test)]
mod test_near_explorer_indexer {
    use super::*;

    #[tokio::test]
    async fn test_near_explorer_indexer() {
        let mut indexer = NearExplorerIndexer::new("priceoracle.near").unwrap();
        assert_eq!(indexer.get_transactions().await.unwrap().len(), 25);
        assert_eq!(indexer.next_page().await.unwrap().len(), 25);
        assert_eq!(indexer.has_next_page(), true);
    }

    #[tokio::test]
    async fn test_near_explorer_indexer_testnet() {
        let mut indexer = NearExplorerIndexer::new("ush_test.testnet").unwrap();
        let data_size = indexer.get_transactions().await.unwrap().len();
        assert!(data_size < 25);
        assert_eq!(indexer.has_next_page(), false);
    }
}