use alloy::{
    primitives::{aliases::U96, utils::parse_ether, Address },
    signers::local::PrivateKeySigner,
};
use alloy_sol_types::SolValue;
use anyhow::Context;
use boundless_market::{
    contracts::{Input, Offer, Predicate, ProvingRequest, Requirements},
    sdk::client::Client,
};
use clap::Parser;
use methods::{VERIFY_ELF, VERIFY_ID};
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{
    default_executor, default_prover, sha::Digestible, ExecutorEnv, ProverOpts, VerifierContext,
};
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
    // set IMAGE_URL
    let image_url = "https://dweb.link/ipfs/Qmd94YSBa2rFamq2vjx4p8GboiW7KjdAhRM6pboJDakRTC";


    let string_input = String::from(serde_json::to_string(&zk_inputs).unwrap());
    let string_input = string_input.as_bytes();
    let input_url = boundless_client.upload_input(string_input).await?;
    println!("Uploaded input to {}", input_url);

    let env = ExecutorEnv::builder().write_slice(string_input).build()?;
    let session_info = default_executor().execute(env, VERIFY_ELF)?;
    let mcycles_count = session_info
        .segments
        .iter()
        .map(|segment| 1 << segment.po2)
        .sum::<u64>()
        .div_ceil(1_000_000);
    let journal = session_info.journal;

    // begin the proving process
    let request = ProvingRequest::default()
        .with_image_url(&image_url)
        .with_input(Input::url(input_url))
        .with_requirements(Requirements::new(
            VERIFY_ID,
            Predicate::digest_match(journal.digest()),
        ))
        .with_offer(
            Offer::default()
            .with_min_price_per_mcycle(
                U96::from::<u128>(parse_ether("0.0001")?.try_into()?),
                mcycles_count,
            )
            // NOTE: If your offer is not being accepted, try increasing the max price.
            .with_max_price_per_mcycle(
                U96::from::<u128>(parse_ether("0.00015")?.try_into()?),
                mcycles_count,
            )
                // .with_lockin_stake(U96::from::<u128>(parse_ether("0.0366")?.try_into()?))
                // The market uses a reverse Dutch auction mechanism to match requests with provers.
                // Each request has a price range that a prover can bid on. One way to set the price
                // is to choose a desired (min and max) price per million cycles and multiply it
                // by the number of cycles. Alternatively, you can use the `with_min_price` and
                // `with_max_price` methods to set the price directly.
                // .with_min_price(U96::from::<u128>(parse_ether("0.056")?.try_into()?))
                // // NOTE: If your offer is not being accepted, try increasing the max price.
                // .with_max_price(U96::from::<u128>(parse_ether("0.122")?.try_into()?))
                // The timeout is the maximum number of blocks the request can stay
                // unfulfilled in the market before it expires. If a prover locks in
                // the request and does not fulfill it before the timeout, the prover can be
                // slashed.
                .with_timeout(100000),
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
