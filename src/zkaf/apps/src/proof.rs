use alloy_sol_types::SolValue;
use anyhow::Context;
use methods::VERIFY_ELF;
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, VerifierContext};
use tlsn_substrings_verifier::ZkInputParam;

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

pub async fn generate_boundless_proof(zk_inputs: ZkInputParam) -> (Vec<u8>, Vec<u8>) {
    // Create a Boundless client from the provided parameters.
    let boundless_client = Client::from_parts(
        args.wallet_private_key,
        args.rpc_url,
        args.proof_market_address,
        args.set_verifier_address,
    )
    .await;
    // serialize the inputs to bytes to pass to the remote prover
    let input = serde_json::to_string(&zk_inputs).unwrap();
    let prefix = get_prefix_string(4, &input);
    let input: Vec<u8> = input.as_bytes().to_vec();
    let image_url = "https://dweb.link/ipfs/QmTx3vDKicYG5RxzMxrZEiCQJqhpgYNrSFABdVz9ri2m5P";

    // begin the proving process
    let request = ProvingRequest::default()
        .with_image_url(&image_url)
        .with_input(Input::inline(input))
        .with_requirements(Requirements::new(result, Predicate::prefix_match(prefix)))
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
    let request_id = boundless_client.submit_request(&request).await;
    tracing::info!("Request {} submitted", request_id);

    // Wait for the request to be fulfilled by the market, returning the journal and seal.
    tracing::info!("Waiting for request {} to be fulfilled", request_id);
    let (_journal, seal) = boundless_client
        .wait_for_request_fulfillment(request_id, Duration::from_secs(5), None)
        .await;
    tracing::info!("Request {} fulfilled", request_id);

    (seal, _journal)
}
