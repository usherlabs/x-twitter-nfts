// ! Entry point for host executing ZK Proof Generation

use anyhow::Result;
use apps::{ generate_tweet_nft_payload, get_nft_by_id };
use dotenv;
use indexer::helper::{
    aurora::TxSender,
    near::verify_near_proof,
    proof::generate_groth16_proof,
    ZkInputParam,
};
use sha256::digest;
use std::thread;
use std::time::Duration;
use verity_verifier::verify_proof;

fn main() -> Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    println!("Proof generation process started");
    // read in the input parameter fom the processed json which contains the substrings
    let proof_params = std::fs::read_to_string("fixtures/zk_params.json").unwrap();
    let proof_params: ZkInputParam = serde_json::from_str(proof_params.as_str()).unwrap();

    // // TODO call image generation service here
    // generate the NFT payload
    let (response, _request) = verify_proof(&proof_params.proof.clone()).unwrap();
    println!("response:\t{}\n\n _request:\t{}\n\n", response, _request);
    let (nft_payload, stringified_nft_payload) = generate_tweet_nft_payload(
        response,
        proof_params.meta_data.clone()
    );

    // generate the proof and journal output
    let (seal, journal_output) = generate_groth16_proof(proof_params);
    let hex_encoded_journal_output = hex::encode(&journal_output);

    println!("{:?} was committed to the journal", hex::encode(&journal_output));
    println!("{:?} was the provided seal", hex::encode(&seal));
    println!("{:?} was the payload generated", nft_payload);

    // verify the journal output is representative of the NFT metadata
    let metadata_hash = digest(stringified_nft_payload.clone());
    assert_eq!(metadata_hash, hex_encoded_journal_output, "invalid payload");

    println!("hash {:?} ", digest(stringified_nft_payload));

    // perform initial verification on aurora
    let runtime = tokio::runtime::Runtime::new()?;
    let aurora_client = TxSender::default();
    let aurora_tx_future = aurora_client.verify_proof_on_aurora(journal_output.clone(), seal);
    let aurora_tx_response = runtime.block_on(aurora_tx_future).unwrap();
    println!("Aurora transation has been verified with response: {:?}\n", aurora_tx_response);

    // perform verification near
    // mint NFT if near verification is successfull
    let near_tx_future = verify_near_proof(journal_output, nft_payload.clone());
    let near_tx_response = runtime.block_on(near_tx_future).unwrap();
    println!("Near transaction has been verified with response: {:?}\n", near_tx_response);

    // get the NFT Minted
    let token_id = nft_payload.title.clone().unwrap();
    println!("Querying for token with id: {}", token_id);
    // Sleep for 3 seconds
    thread::sleep(Duration::from_secs(5));
    let nft_future = get_nft_by_id(token_id.clone());
    let nft_details = runtime.block_on(nft_future).unwrap();
    println!("NFT:{} Succesfully minted", token_id);
    println!("{:?}", nft_details);
    Ok(())
}
