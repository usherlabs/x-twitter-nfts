use alloy_primitives::{Bytes, FixedBytes, U256};
use alloy_sol_types::{sol, sol_data::Bool, SolInterface, SolType, SolValue};
use anyhow::{Context, Result};
use apps::{near::verify_near_proof, BonsaiProver, TxSender};
use clap::Parser;
use methods::VERIFY_ELF;

use serde::{Deserialize, Serialize};
use tlsn_substrings_verifier::{self, proof::{SessionHeader, SubstringsProof}};
#[derive(Serialize, Deserialize, Debug)]
struct ZkParam {
    header: SessionHeader,
    substrings: SubstringsProof,
}

// `IVerifier` interface automatically generated via the alloy `sol!` macro.
sol! {
    interface IVerifier {
        function verify_proof(bytes memory journal_output, bytes32 post_state_digest, bytes calldata seal);
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("")
}

/// Arguments of the publisher CLI.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Ethereum chain ID
    #[clap(long)]
    chain_id: u64,

    /// Ethereum Node endpoint.
    #[clap(long, env)]
    eth_wallet_private_key: String,

    /// Ethereum Node endpoint.
    #[clap(long)]
    rpc_url: String,

    /// Application's contract address on Ethereum
    #[clap(long)]
    contract: String,

    /// The input to provide to the guest binary
    #[clap(short, long)]
    input: U256,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // // Create a new `TxSender`.
    // let tx_sender = TxSender::new(
    //     1313161555,
    //     "https://testnet.aurora.dev",
    //     "AURORA_PK",
    //     "0x3dd915C0A23656f8F060187A3721193c43f0E289",
    // )?;

    // ABI encode the input for the guest binary, to match what the `is_even` guest
    // code expects.
    let proof_params = std::fs::read_to_string("inputs/zk_params.json").unwrap();
    let proof_params: ZkParam = serde_json::from_str(proof_params.as_str()).unwrap();

    let input = serde_json::to_string(&proof_params).unwrap();
    let input: &[u8] = input.as_bytes();

    // Send an off-chain proof request to the Bonsai proving service.
    let (journal, post_state_digest, seal) = BonsaiProver::prove(VERIFY_ELF, input)?;

    // Decode the journal. Must match what was written in the guest with
    let req_res_hash = <Vec<u8>>::abi_decode(&journal, true).context("decoding journal data")?;
    let hex_string = hex::encode(&req_res_hash);

    println!("{hex_string} was committed to the journal");

    let tx = verify_near_proof(req_res_hash, post_state_digest.to_vec(), seal);

    // tests to get it working with minimal inputs and outputs
    // let calldata = IVerifier::IVerifierCalls::verify_proof(IVerifier::verify_proofCall {
    //     journal_output: req_res_hash,
    //     post_state_digest,
    //     seal,
    // })
    // .abi_encode();

    // call the proof directly from the aurura contract
    // calling the aurora method directly from the near contract works
    // let post_state_digest:[u8;32] = hex::decode("0622b43513c8148548a5aa01d3842e669df2ed7cae1b498e98ae1a8b3aae1a14").unwrap().try_into().unwrap();
    // let calldata = IVerifier::IVerifierCalls::verify_proof(IVerifier::verify_proofCall {
    //     journal_output: hex::decode("4f8ad5ce1d0fc577d04d618880b8c77c9aced63740ca43b708f32425a95b11b7").unwrap(),
    //     post_state_digest: FixedBytes(post_state_digest),
    //     seal: hex::decode("1a61e905022e6727f77f55f5632d1b418040e236961222301b1b0df883d8124c225b443b81443529841cbd6845e65efc7a36547ff66003d5e94ac36f91ca83f42c81d508177c6b03d647ccc662e83ba94f0814f941dbef13dc374eea5897bd5a19578107ca5766c68e0a49b4576c0e3d30d4c82f5bdd34ffff81673d2141b6d6163b8ac07ddd56827824da2196046baade5cfd64dd3d0046a4faa1c0b2602f762a2d547b5bf7aaadca7c1f69f5a97eebf7f9c810fe420ab846081f7a8a16e1172735b48780dc334aaa3a8c4806c02a396746e6500cee439f8bf9b6de1c43a0e81345aac7a3fce0e8eed676b4307611b7b8e567f1e6f14333e29e032843a0fe1b").unwrap(),
    // })
    // .abi_encode();

    // // Send the calldata to Ethereum.
    // let runtime = tokio::runtime::Runtime::new()?;
    // let tx_response = runtime.block_on(tx_sender.send(calldata)).unwrap();

    // println!("{:?}", tx_response);
    // tests to get it working with minimal inputs and outputs

    Ok(())
}
