use alloy_sol_types::SolValue;
use anyhow::Context;
use methods::VERIFY_ELF;
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, VerifierContext};
use serde::{Deserialize, Serialize};
use verity_verifier::verify_proof;

/// Containing the details needed for verification of a proof
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZkInputParam {
    /// session header.
    pub proof: String,

    /// meta_data
    pub meta_data: AssetMetadata,
}

/// The Includes substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetMetadata {
    /// NFT image url
    pub image_url: String,

    /// Near account to be minted to
    pub owner_account_id: String,

    /// tweet Id
    pub token_id: String,
}

/// The tweet structure gotten from the API
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize)]
pub struct Tweet {
    /// data
    pub data: Vec<Data>,
    /// users info
    pub includes: Includes,
}

/// The data substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    /// date created
    pub created_at: String,
    /// id
    pub id: String,
    /// public_metrics
    pub public_metrics: PublicMetrics,
    /// edit_history_tweet_ids
    pub edit_history_tweet_ids: Vec<String>,
    /// author_id of tweet
    pub author_id: String,
    /// tweet text
    pub text: String,
}

/// The PublicMetrics substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize)]
pub struct PublicMetrics {
    /// pub retweet_count
    pub retweet_count: u32,
    /// pub reply_count
    pub reply_count: u32,
    /// pub like_count
    pub like_count: u32,
    /// pub quote_count
    pub quote_count: u32,
    /// pub bookmark_count
    pub bookmark_count: u32,
    /// pub impression_count
    pub impression_count: u32,
}

/// The Includes substructure of a metadata NFT
///
/// Containing the details about a metadata NFT
#[derive(Debug, Deserialize, Serialize)]
pub struct Includes {
    /// users
    pub users: Vec<User>,
}

/// The User substructure of a tweet
///
/// Containing the details about a tweet
#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    /// pub id: String,
    pub id: String,
    /// pub username: Option<String>
    pub username: Option<String>, // Optional because it's not present in every user object
    /// pub name: String,
    pub name: String,
    /// pub created_at: String,
    pub created_at: String,
}

pub fn generate_groth16_proof(zk_inputs: ZkInputParam) -> (Vec<u8>, Vec<u8>) {
    // serialize the inputs to bytes to pass to the remote prover
    let input = serde_json::to_string(&zk_inputs).unwrap();
    let input: &[u8] = input.as_bytes();

    // begin the proving process
    let env = ExecutorEnv::builder().write_slice(&input).build().unwrap();
    let receipt = default_prover()
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
        .context("decoding journal data")
        .unwrap();

    (seal, journal_output)
}


impl ZkInputParam {
    /// Returns the proof string associated with this ZkInputParam.
    #[must_use]
    pub fn get_proof(&self) -> (String,String) {
        verify_proof(&self.proof).unwrap()
    }
}
