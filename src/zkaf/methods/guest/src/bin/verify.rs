use sha256::digest;
use std::io::Read;

use alloy_sol_types::SolValue;
use risc0_zkvm::guest::env;

use serde::{Deserialize, Serialize};

use apps::{near::generate_tweet_nft_payload, proof::ZkInputParam};

fn main() {
    // Read the input data for this application.
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();

    let proof_params: String = String::from_utf8(input_bytes).unwrap();
    let proof_params: ZkInputParam = serde_json::from_str(proof_params.as_str()).unwrap();

    let (response, _request) = proof_params.get_proof();
    let (_, string_metadata) = generate_tweet_nft_payload(response, proof_params.meta_data);

    env::log(&format!("Derived metadata: {}", string_metadata));

    let metadata_hash = digest(string_metadata);
    let encoded_metadata_hash = hex::decode(metadata_hash).unwrap();

    env::log("committing results to journal");
    // Commit the journal that will be received by the application contract.
    // Journal is encoded using Solidity ABI for easy decoding in the app contract.
    env::commit_slice(encoded_metadata_hash.abi_encode().as_slice());
}
