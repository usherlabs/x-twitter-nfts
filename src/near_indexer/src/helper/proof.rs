use std::env;

use crate::generated::methods::VERIFY_ELF;
use super::{TweetResponse, ZkInputParam};
use alloy_sol_types::SolValue;
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{ ExecutorEnv, ProverOpts, VerifierContext,BonsaiProver};
use std::error::Error;
use tracing::debug;
use verity_client::client::{VerityClient, VerityClientConfig};
use risc0_zkvm::Prover;

pub fn generate_groth16_proof(zk_inputs: ZkInputParam) -> (Vec<u8>, Vec<u8>) {
    // serialize the inputs to bytes to pass to the remote prover
    let input = serde_json::to_string(&zk_inputs).unwrap();
    let input: &[u8] = input.as_bytes();

    // begin the proving process
    let env = ExecutorEnv::builder().write_slice(&input).build().unwrap();
    let receipt = BonsaiProver::new("groth16")
        .prove_with_ctx(
            env,
            &VerifierContext::default(),
            VERIFY_ELF,
            &ProverOpts::groth16(),
        )
        .unwrap()
        .receipt;

    // // Encode the seal with the selector.
    let seal = groth16::encode(receipt.inner.groth16().unwrap().seal.clone()).unwrap();

    // // Extract the journal from the receipt.
    let journal = receipt.journal.bytes.clone();

    let journal_output = <Vec<u8>>::abi_decode(&journal, true)
        // .context("decoding journal data")
        .unwrap();

    (seal, journal_output)
}

pub async fn get_proof(tweet_id: String) -> Result<(String, TweetResponse), Box<dyn Error>> {
    let verity_client = get_verity_client();
    let _temp = verity_client
        .get(
            format!("https://api.x.com/2/tweets?ids={}&tweet.fields=created_at,public_metrics&expansions=author_id&user.fields=created_at", tweet_id)
        )
        .header(
            "Authorization",
            format!("Bearer {}", env::var("TWEET_BEARER").unwrap_or(String::from("")))
        ).redact("req:header:authorization".to_string())
        .send().await;

    if _temp.is_err() {
        debug!("Error: {:?} ", _temp.err());
        return Err(format!("Error found ID").into());
    }

    let verify_response = _temp.unwrap();
    let res = verify_response.subject.json::<TweetResponse>().await;
    if res.is_err() {
        debug!("invalid Tweet at {},{:?}", tweet_id, res.err());
        return Err(format!("invalid Tweet ID").into());
    }
    let res = res.expect("response must exist");
    if res.data.is_some() {
        Ok((verify_response.proof, res))
    } else {
        debug!("Error found at {},{:?}", tweet_id, res);
        Err(format!("Error found ID").into())
    }
}

pub fn get_verity_client() -> VerityClient {
    let verity_config = VerityClientConfig {
        prover_url: env::var("VERITY_PROVER_URL").unwrap_or(String::from("http://127.0.0.1:8080")),
    };

    VerityClient::new(verity_config)
}
