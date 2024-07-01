// ! Entry point for host executing ZK Proof Generation

use std::{env, str::FromStr};
use std::thread;
use std::time::Duration;
use alloy_primitives::U256;
use alloy_sol_types::{sol, SolInterface, SolValue};
use anyhow::{Context, Result};
use apps::{aurora::TxSender, near::{get_nft_by_id, verify_near_proof}, proof::generate_groth16_proof};
use clap::Parser;
use ethers::prelude::*;
use methods::VERIFY_ELF;
use near_contract_standards::non_fungible_token::TokenId;
use near_primitives::types::AccountId;
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, VerifierContext};
use tlsn_substrings_verifier::{nft::{generate_tweet_nft_payload, get_http_payload}, ZkInputParam};
use sha256::digest;


fn main() -> Result<()> {
    env_logger::init();

    println!("Proof generation process started");
    // // read in the input parameter fom the processed json which contains the substrings
    let proof_params = std::fs::read_to_string("inputs/zk_params.json").unwrap();
    let proof_params: ZkInputParam = serde_json::from_str(proof_params.as_str()).unwrap();

    // // TODO call image generation service here
    // // // generate the NFT payload
    let (_request, response) = get_http_payload(proof_params.clone());
    let (nft_payload, stringified_nft_payload) = generate_tweet_nft_payload(response);
    
    // // // generate the proof and journal output
    let (seal, journal_output) = generate_groth16_proof(proof_params);
    // let seal = hex::decode("310fe598136858b44eca32b144a1403b051abebe4743e7327fdef1a9d8d049b162c29c0414a791d8c46e3c184d29be950d798135751592b6c641d6830ca89c705f90f0f2209cb6214d9afc83878fd01861802c9b5ef0183c70e9227e30c41f66ae7eeac62dfbb9f2e60912438795d967ba0c508fc02cbebb2f12224fc0e1f7865260ea5828ae4194a9536e0b012281dc5e391e210be497652942f0d70a87927b22c304e30ffc0680c919b2f0e1121ae79a021997862ff2782da9362b1db0c99bac61c827301a23de27f994bd7afdb7c1771de3b6d2a06b713af547f2ba6e8b9bab36895504a56b185df4673d3143a329bbd13f3c6deb686810dbc1ec2f98fc4452b872fb").unwrap();
    // let journal_output = hex::decode("87532ef9e4e9d2ae58ce81ed14f5aa9b50babb2dda3a9266af790876c4bc02bd").unwrap();
    let hex_encoded_journal_output = hex::encode(&journal_output);

    println!("{:?} was committed to the journal", hex::encode(&journal_output));
    println!("{:?} was the provided seal", hex::encode(&seal));

    // // verify the journal output is representative of the NFT metadata
    let metadata_hash = digest(stringified_nft_payload);
    assert_eq!(metadata_hash, hex_encoded_journal_output, "invalid payload");

    // // perform initial verification on aurora
    let runtime = tokio::runtime::Runtime::new()?;
    let aurora_client = TxSender::default();
    let aurora_tx_future = aurora_client.verify_proof_on_aurora(
        journal_output.clone(),
        seal
    );
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
    let nfts_owned_tx_future = get_nft_by_id(token_id.clone());
    let nfts_owned = runtime.block_on(nfts_owned_tx_future).unwrap();
    println!("NFT:{} Succesfully minted", token_id);
    println!("{:?}", nfts_owned);
    Ok(())
}
