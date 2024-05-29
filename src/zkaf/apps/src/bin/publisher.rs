// ! Entry point for host executing ZK Proof Generation

use alloy_sol_types::SolValue;
use anyhow::{Context, Result};
use apps::{aurora::TxSender, near::verify_near_proof, BonsaiProver};
use methods::VERIFY_ELF;
use tlsn_substrings_verifier::{self, ZkInputParam};


fn main() -> Result<()> {
    env_logger::init();

    // read in the input parameter fom the processed json which contains the substrings
    let proof_params = std::fs::read_to_string("inputs/zk_params.json").unwrap();
    let proof_params: ZkInputParam = serde_json::from_str(proof_params.as_str()).unwrap();

    // serialize the inputs to bytes to pass to the remote prover 
    let input = serde_json::to_string(&proof_params).unwrap();
    let input: &[u8] = input.as_bytes();

    // Send an off-chain proof request to the Bonsai proving service.
    let (journal, post_state_digest, seal) = BonsaiProver::prove(VERIFY_ELF, input)?;

    // Decode the journal. Must match what was written in the guest
    let journal_output = <Vec<u8>>::abi_decode(&journal, true).context("decoding journal data")?;
    let hex_string = hex::encode(&journal_output);
    println!("{hex_string} was committed to the journal");

    // perform initial verification on aurora
    let runtime = tokio::runtime::Runtime::new()?;
    let aurora_client = TxSender::default();
    let aurora_tx_future = aurora_client.verify_proof_on_aurora(
        journal_output.clone(),
        post_state_digest,
        seal
    );
    let aurora_tx_response = runtime.block_on(aurora_tx_future).unwrap();
    println!("Aurora transation has been verified with response: {:?}", aurora_tx_response);


    let near_tx_future = verify_near_proof(journal_output);
    let near_tx_response = runtime.block_on(near_tx_future).unwrap();
    println!("\nNear transaction has been verified with response: {:?}", near_tx_response);

    println!("\nSuccessfully verified proof: {:?}", near_tx_response);
    Ok(())
}
