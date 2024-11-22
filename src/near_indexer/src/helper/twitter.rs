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

impl OathTweeterHandler {
    /// Creates a default `OathTweeterHandler`.
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

    /// Creates a new `OathTweeterHandler`.
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
            "text": format!("Hey {},  Your X-NFT with ID:{} has been successfully minted! 🎉\nCheck your wallet to confirm receipt of your new digital collectible. Enjoy owning a unique piece of blockchain art! #NFT #Web3",notify,tweet_id)
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
        assert!(res["data"].take()["id"].take().to_string().len()> 3);
    }
}
