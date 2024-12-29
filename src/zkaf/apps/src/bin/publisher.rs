// ! Entry point for host executing ZK Proof Generation

use anyhow::Result;
use apps::{boundless::generate_boundless_proof, generate_tweet_nft_payload};
use dotenv;
use indexer::helper::ZkInputParam;
use verity_verify_tls::verify_proof;

fn main() -> Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    println!("Proof generation process started");

    let proof_params = std::fs::read_to_string("fixtures/zk_params.json").unwrap();
    let proof_params: ZkInputParam = serde_json::from_str(proof_params.as_str()).unwrap();

    // // TODO call image generation service here
    // generate the NFT payload
    let (response, _request) = verify_proof(&proof_params.proof.clone()).unwrap();
    println!("response:\t{}\n\n _request:\t{}\n\n", response, _request);
    let (nft_payload, _) = generate_tweet_nft_payload(response, proof_params.meta_data.clone());

    let runtime = tokio::runtime::Runtime::new()?;
    // generate the boundless proof and journal output
    let bound_future = generate_boundless_proof(proof_params.clone());
    let (seal, journal_output) = runtime.block_on(bound_future).unwrap();

    // let hex_encoded_journal_output = hex::encode(&journal_output);
    println!(
        "boundless:\t{:?} was committed to the journal",
        hex::encode(&journal_output)
    );

    println!("boundless:\t{:?} was the provided seal", hex::encode(&seal));
    println!("boundless:\t{:?} was the payload generated", nft_payload);

    Ok(())
}
