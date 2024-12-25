use reqwest;
use reqwest_oauth1::OAuthClientProvider;
use serde_json::json;
use serde_json::Value;
use std::env;
use std::error::Error;
use std::marker::{Send, Sync};

pub struct OathTweeterHandler {
    consumer_key: String,
    consumer_secret: String,
    access_token: String,
    token_secret: String,
}

fn add_prefix_if_needed(s: &str) -> String {
    if s.starts_with('@') {
        s.to_string()
    } else {
        format!("@{}", s)
    }
}

impl OathTweeterHandler {
    ///
    /// This method retrieves the necessary credentials from environment variables
    /// and creates a new `OathTweeterHandler` instance.
    ///
    /// # Environment Variables
    ///
    /// - `TWEET_CONSUMER_KEY`: The consumer key for Twitter API authentication.
    /// - `TWEET_CONSUMER_SECRET`: The consumer secret for Twitter API authentication.
    /// - `TWEET_ACCESS_TOKEN`: The access token for Twitter API authentication.
    /// - `TWEET_TOKEN_SECRET`: The token secret for Twitter API authentication.
    pub fn default() -> Self {
        let consumer_key = env::var("TWEET_CONSUMER_KEY").expect("TWEET_CONSUMER_KEY_NOT_PRESENT");
        let consumer_secret =
            env::var("TWEET_CONSUMER_SECRET").expect("TWEET_CONSUMER_SECRET_NOT_PRESENT");
        let access_token = env::var("TWEET_ACCESS_TOKEN").expect("TWEET_ACCESS_TOKEN_NOT_PRESENT");
        let token_secret = env::var("TWEET_TOKEN_SECRET").expect("TWEET_TOKEN_SECRET_NOT_PRESENT");
        return Self::new(
            &consumer_key,
            &consumer_secret,
            &access_token,
            &token_secret,
        )
        .unwrap();
    }

    /// Creates a new `OathTweeterHandler` with the provided credentials.
    ///
    /// This method takes individual components of the Twitter API authentication
    /// as separate parameters and creates a new `OathTweeterHandler` instance.
    ///
    /// # Parameters
    ///
    /// - `consumer_key`: The consumer key for Twitter API authentication.
    /// - `consumer_secret`: The consumer secret for Twitter API authentication.
    /// - `access_token`: The access token for Twitter API authentication.
    /// - `token_secret`: The token secret for Twitter API authentication.
    ///
    /// # Errors
    ///
    /// This method will return an error if any of the provided credentials are invalid.
    pub fn new(
        consumer_key: &str,
        consumer_secret: &str,
        access_token: &str,
        token_secret: &str,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(OathTweeterHandler {
            consumer_key: consumer_key.to_string(),
            consumer_secret: consumer_secret.to_string(),
            access_token: access_token.to_string(),
            token_secret: token_secret.to_string(),
        })
    }

    /// Sends a notification tweet using the Twitter API.
    ///
    /// This method sends a formatted tweet to notify someone about a newly minted NFT.
    ///
    /// # Parameters
    ///
    /// - `tweet_id`: The ID of the tweet being responded to.
    /// - `notify`: The username to notify in the tweet.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Value` representing the API response, or an error if the request fails.
    pub async fn notifier(
        &self,
        tweet_id: &str,
        notify: &str,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let secrets =
            reqwest_oauth1::Secrets::new(self.consumer_key.clone(), self.consumer_secret.clone())
                .token(self.access_token.clone(), self.token_secret.clone());
        let endpoint = "https://api.x.com/2/tweets";

        let payload = json!({
            "text": format!("Hey {},  Your X-NFT with ID:{} has been successfully minted! 🎉\nCheck your wallet to confirm receipt of your new digital collectible. Enjoy owning a unique piece of blockchain art! #NFT #Web3", add_prefix_if_needed(notify), tweet_id)
        }).to_string();

        let client = reqwest::Client::new();
        let resp = client
            // enable OAuth1 request
            .oauth1(secrets)
            .post(endpoint)
            .body(payload)
            .header("content-type", "application/json")
            .send()
            .await?;
        let res: Value = resp.json().await.unwrap();
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_notifier() {
        dotenv().expect("Error occurred when loading .env");

        let tweet_id = "1858184885493485672";
        let user = "@ryan_soury";

        let twitter_client = OathTweeterHandler::default();

        let mut res: Value = twitter_client.notifier(tweet_id, &user).await.unwrap();
        println!("{:?}", res);
        assert!(res["data"].take()["id"].take().to_string().len() > 3);
    }

    #[test]
    fn test_add_prefix_if_needed() {
        let test_strings = vec![
            ("@example", "@example"),
            ("example", "@example"),
            ("no_prefix", "@no_prefix"),
        ];

        for test_variables in test_strings {
            assert_eq!(test_variables.1, add_prefix_if_needed(test_variables.0))
        }
    }
}
