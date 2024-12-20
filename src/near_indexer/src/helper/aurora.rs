use alloy_sol_types::{sol, SolInterface};
use ethers::prelude::*;
use tracing::info;
use std::env;
use std::error::Error;
use std::marker::{Send, Sync};


// `IVerifier` interface automatically generated via the alloy `sol!` macro.
sol! {
    interface IVerifier {
        function verify_proof(bytes memory journal_output, bytes calldata seal);
    }
}

pub struct TxSender {
    chain_id: u64,
    client: SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>,
    contract: Address,
}

impl TxSender {
    /// Creates a default `TxSender`.
    pub fn default() -> Self {
        let chain_id: u64 = env::var("EVM_CHAIN_ID")
            .expect("EVM_CHAIN_ID_NOT_PRESENT")
            .parse()
            .unwrap();
        let rpc_url = env::var("AURORA_RPC_URL").expect("AURORA_RPC_URL_NOT_PRESENT");
        let private_key =
            env::var("AURORA_WALLET_PRIVATE_KEY").expect("AURORA_WALLET_PRIVATE_KEY_NOT_PRESENT");
        let contract_address: String =
            env::var("AURORA_VERIFIER_CONTRACT").expect("AURORA_VERIFIER_CONTRACT_NOT_PRESENT");
        println!("contract_address:{}", contract_address);
        return Self::new(chain_id, &rpc_url, &private_key, &contract_address).unwrap();
    }

    /// Creates a new `TxSender`.
    pub fn new(
        chain_id: u64,
        rpc_url: &str,
        private_key: &str,
        contract: &str,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
        let client = SignerMiddleware::new(provider.clone(), wallet.clone());
        let contract = contract.parse::<Address>()?;

        Ok(TxSender {
            chain_id,
            client,
            contract,
        })
    }

    /// Send a transaction with the given calldata.
    pub async fn send(
        &self,
        calldata: Vec<u8>,
    ) -> Result<Option<TransactionReceipt>, Box<dyn Error + Send + Sync>> {
        let tx = TransactionRequest::new()
            .chain_id(self.chain_id)
            .to(self.contract)
            .from(self.client.address())
            .data(calldata);

        info!("Transaction request: {:?}", &tx);

        let tx = self.client.send_transaction(tx, None).await?.await?;

        info!("Transaction receipt: {:?}", &tx);

        Ok(tx)
    }

    /// verify a snark on aurora
    pub async fn verify_proof_on_aurora(
        &self,
        journal_output: Vec<u8>,
        seal: Vec<u8>,
    ) -> Option<TransactionReceipt> {
        let calldata = IVerifier::IVerifierCalls::verify_proof(IVerifier::verify_proofCall {
            journal_output: journal_output,
            seal,
        })
        .abi_encode();

        let tx = self.send(calldata).await.unwrap();
        return tx;
    }
}
