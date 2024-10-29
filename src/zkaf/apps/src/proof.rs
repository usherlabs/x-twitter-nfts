use alloy::{
    primitives::{aliases::U96, utils::parse_ether, Address, Bytes},
    signers::local::PrivateKeySigner,
};
use alloy_sol_types::SolValue;
use anyhow::Context;
use boundless_market::{
    contracts::{Input, Offer, Predicate, ProvingRequest, Requirements},
    sdk::client::Client,
};
use clap::Parser;
use methods::VERIFY_ELF;
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, VerifierContext};
use std::time::Duration;
use tlsn_substrings_verifier::ZkInputParam;
use url::Url;

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

    // Encode the seal with the selector.
    let seal = groth16::encode(receipt.inner.groth16().unwrap().seal.clone()).unwrap();

    // // Extract the journal from the receipt.
    let journal = receipt.journal.bytes.clone();
    let journal_output = <Vec<u8>>::abi_decode(&journal, true)
        .context("decoding journal data")
        .unwrap();

    (seal, journal_output)
}

fn convert_to_big_endian(value: usize) -> String {
    let len = format!("{:x}", value);
    let x = (32 - len.len() * 4) as u8;
    format!("{:x}", value << x)
}

fn get_prefix_string(prefix_length: usize, input_string: &str) -> Vec<u8> {
    let prefix = &input_string[..prefix_length];
    prefix.as_bytes().to_vec()
}

fn encoded_string(string_byte: Vec<u8>) -> Vec<u8> {
    let temp_length = string_byte.len();
    let hex_string = vec![
        convert_to_big_endian(temp_length),
        string_byte
            .iter()
            .map(|&b| {
                let b32: usize = b.into();
                format!("{}", convert_to_big_endian(b32))
            })
            .collect::<Vec<_>>()
            .join(""),
    ]
    .join("");

    return hex::decode(hex_string).unwrap();
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// URL of the Ethereum RPC endpoint.
    #[clap(short, long, env)]
    rpc_url: Url,
    /// Private key used to interact with the EvenNumber contract.
    #[clap(short, long, env)]
    wallet_private_key: PrivateKeySigner,

    /// Address of the SetVerifier contract.
    #[clap(short, long, env)]
    set_verifier_address: Address,
    /// Address of the ProofMarket contract.
    #[clap(short, long, env)]
    proof_market_address: Address,
}

pub async fn generate_boundless_proof(
    zk_inputs: ZkInputParam,
) -> Result<(Vec<u8>, Vec<u8>), anyhow::Error> {
    let args = Args::parse();
    // Create a Boundless client from the provided parameters.
    let boundless_client = Client::from_parts(
        args.wallet_private_key,
        args.rpc_url,
        args.proof_market_address,
        args.set_verifier_address,
    )
    .await?;
    // serialize the inputs to bytes to pass to the remote prover
    let input = serde_json::to_string(&zk_inputs).unwrap();
    let prefix = get_prefix_string(4, &input);
    let input: Vec<u8> = input.as_bytes().to_vec();
    let image_url = "https://dweb.link/ipfs/QmTx3vDKicYG5RxzMxrZEiCQJqhpgYNrSFABdVz9ri2m5P";

    let _image_id = "257569e11f856439ec3c1e0fe6486fb9af90b1da7324d577f65dd0d45ec12c7d";

    // Parse the hexadecimal string
    let _image_id = hex::decode(_image_id).unwrap();

    // Convert the byte slice to a fixed-size array of 32 bytes
    let mut image_id: [u8; 32] = [0; 32];
    image_id.copy_from_slice(&_image_id);

    // begin the proving process
    let request = ProvingRequest::default()
        .with_image_url(&image_url)
        .with_input(Input::inline(input))
        .with_requirements(Requirements::new(image_id, Predicate::prefix_match(prefix)))
        .with_offer(
            Offer::default()
                // The market uses a reverse Dutch auction mechanism to match requests with provers.
                // Each request has a price range that a prover can bid on. One way to set the price
                // is to choose a desired (min and max) price per million cycles and multiply it
                // by the number of cycles. Alternatively, you can use the `with_min_price` and
                // `with_max_price` methods to set the price directly.
                .with_min_price_per_mcycle(U96::from::<u128>(parse_ether("0.001")?.try_into()?), 1)
                // NOTE: If your offer is not being accepted, try increasing the max price.
                .with_max_price_per_mcycle(U96::from::<u128>(parse_ether("0.002")?.try_into()?), 1)
                // The timeout is the maximum number of blocks the request can stay
                // unfulfilled in the market before it expires. If a prover locks in
                // the request and does not fulfill it before the timeout, the prover can be
                // slashed.
                .with_timeout(1000),
        );

    // Send the request and wait for it to be completed.
    let request_id = boundless_client.submit_request(&request).await?;
    println!("Request {} submitted", request_id);

    // Wait for the request to be fulfilled by the market, returning the journal and seal.
    println!("Waiting for request {} to be fulfilled", request_id);
    let (_journal, seal) = boundless_client
        .wait_for_request_fulfillment(request_id, Duration::from_secs(5), None)
        .await?;
    println!("Request {} fulfilled", request_id);

    Ok((seal.to_vec(), _journal.to_vec()))
}
