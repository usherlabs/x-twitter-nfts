use alloy::{
    primitives::{ aliases::U256, utils::parse_ether, Address },
    signers::local::PrivateKeySigner,
};
use anyhow::ensure;
use alloy_sol_types::SolValue;
use anyhow::Context;
use boundless_market::{
    contracts::{ Input, Offer, Predicate, ProofRequest, Requirements },
    client::ClientBuilder,
    storage::StorageProviderConfig,
};
use clap::Parser;
use indexer::helper::ZkInputParam;
use methods::{ VERIFY_ELF, VERIFY_ID };
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{
    default_executor,
    default_prover,
    sha::Digestible,
    ExecutorEnv,
    ProverOpts,
    VerifierContext,
};
use url::Url;
use std::time::Duration;


pub fn generate_groth16_proof(zk_inputs: ZkInputParam) -> (Vec<u8>, Vec<u8>) {
    // serialize the inputs to bytes to pass to the remote prover
    let input = serde_json::to_string(&zk_inputs).unwrap();
    let input: &[u8] = input.as_bytes();

    // begin the proving process
    let env = ExecutorEnv::builder().write_slice(&input).build().unwrap();
    let receipt = default_prover()
        .prove_with_ctx(env, &VerifierContext::default(), VERIFY_ELF, &ProverOpts::groth16())
        .unwrap().receipt;

    // Encode the seal with the selector.
    let seal = groth16::encode(receipt.inner.groth16().unwrap().seal.clone()).unwrap();

    // // Extract the journal from the receipt.
    let journal = receipt.journal.bytes.clone();
    let journal_output = <Vec<u8>>
        ::abi_decode(&journal, true)
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

    /// Storage provider to use
    #[clap(flatten)]
    storage_config: Option<StorageProviderConfig>,

    /// Submit the request offchain via the provided order stream service url.
    #[clap(short, long, requires = "order_stream_url")]
    offchain: bool,
    /// Offchain order stream service URL to submit offchain requests to.
    #[clap(long, env)]
    order_stream_url: Option<Url>,
}

pub async fn generate_boundless_proof(
    zk_inputs: ZkInputParam
) -> Result<(Vec<u8>, Vec<u8>), anyhow::Error> {
    let args = Args::parse();


    // Create a Boundless client from the provided parameters.
    let boundless_client = ClientBuilder::default()
      .with_rpc_url(args.rpc_url)
      .with_boundless_market_address(args.proof_market_address)
      .with_set_verifier_address(args.set_verifier_address)
      .with_storage_provider_config(args.storage_config)
      .with_order_stream_url(args.offchain.then_some(args.order_stream_url).flatten())
      .with_private_key(args.wallet_private_key)
      .build()
      .await?;

    ensure!(
        boundless_client.storage_provider.is_some(),
        "a storage provider is required to upload the zkVM guest ELF"
    );

    // set IMAGE_URL
    // TODO: change this a local IPFS hash reference of the Image URL
    let image_url = "https://dweb.link/ipfs/Qmb7VomWXebHNzey3KGr6bh5McHB6LURkKGz2VKZFsQ3TR";

    let string_input = String::from(serde_json::to_string(&zk_inputs).unwrap());
    let string_input = string_input.as_bytes();
    let input_url = boundless_client.upload_input(string_input).await?;
    println!("Uploaded input to {}", input_url);

    let env = ExecutorEnv::builder().write_slice(string_input).build()?;
    let session_info = default_executor().execute(env, VERIFY_ELF)?;
    let mcycles_count = session_info.segments
        .iter()
        .map(|segment| 1 << segment.po2)
        .sum::<u64>()
        .div_ceil(1_000_000);
    let journal = session_info.journal;

    // begin the proving process
    let request = ProofRequest::default()
        .with_image_url(&image_url)
        .with_input(Input::url(input_url))
        .with_requirements(Requirements::new(VERIFY_ID, Predicate::digest_match(journal.digest())))
        .with_offer(
            Offer::default()
                .with_min_price_per_mcycle(parse_ether("0.001")?, mcycles_count)
                // NOTE: If your offer is not being accepted, try increasing the max price.
                .with_max_price_per_mcycle(parse_ether("0.002")?, mcycles_count)
                // slashed.
                .with_timeout(100000)
        );

    // Send the request and wait for it to be completed.
    let (request_id,expires_at) = boundless_client.submit_request(&request).await?;
    println!("Request {} submitted", request_id);

    // Wait for the request to be fulfilled by the market, returning the journal and seal.
    println!("Waiting for request {} to be fulfilled", request_id);
    let (_journal, seal) = boundless_client.wait_for_request_fulfillment(
        request_id,
        Duration::from_secs(5),
        expires_at+1000
    ).await?;
    println!("Request {} fulfilled", request_id);

    Ok((seal.to_vec(), _journal.to_vec()))
}