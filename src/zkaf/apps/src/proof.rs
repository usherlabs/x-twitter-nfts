use tlsn_substrings_verifier::ZkInputParam;
use methods::VERIFY_ELF;
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, VerifierContext};
use alloy_sol_types::SolValue;
use anyhow::Context;

pub fn generate_groth16_proof(zk_inputs: ZkInputParam) -> (Vec<u8>,Vec<u8>){
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
        ).unwrap()
        .receipt;

    
    // // Encode the seal with the selector.
    let seal = groth16::encode(receipt.inner.groth16().unwrap().seal.clone()).unwrap();

    // // Extract the journal from the receipt.
    let journal = receipt.journal.bytes.clone();
    let journal_output = <Vec<u8>>::abi_decode(&journal, true).context("decoding journal data").unwrap();

    (seal, journal_output)
}