use aurora_sdk_integration_tests::{
    aurora_engine::{AuroraEngine, ContractInput},
    aurora_engine_types::types::Address,
    ethabi,
    utils::ethabi::{ContractConstructor, DeployedContract},
    workspaces,
};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use std::path::Path;

const RES_PATH: &str = "abi";

#[derive(Debug)]
pub struct VerifierConstructor(pub ContractConstructor);

#[derive(Debug)]
pub struct Verifier(pub DeployedContract);

impl VerifierConstructor {
    pub fn load() -> Self {
        let contract_path = Path::new(RES_PATH).join("Verifier.json");
        Self(ContractConstructor::from_extended_json(contract_path))
    }

    /// Creates the bytes that are used as the input to an EVM transaction for deploying
    /// the PositionManager contract. This function does not interact with any EVM itself, it only
    /// produces the bytes needed to pass to an EVM.
    pub fn create_deploy_bytes(&self, groth16_verifier_address: Address) -> Vec<u8> {
        self.0
            .create_deploy_bytes_with_args(&[ethabi::Token::Address(
                groth16_verifier_address.raw(),
            )])
    }

    // bytes memory journalOutput, bytes calldata seal

    pub fn deployed_at(self, address: Address) -> Verifier {
        Verifier(self.0.deployed_at(address))
    }
}

impl Verifier {
    pub fn create_verify_proof_bytes(
        &self,
        journal_output: Vec<u8>,
        seal: Vec<u8>,
    ) -> ContractInput {
        ContractInput(self.0.create_call_method_bytes_with_args(
            "verify_proof",
            &[
                ethabi::Token::Bytes(journal_output),
                ethabi::Token::Bytes(seal),
            ],
        ))
    }

    /// the token metadata generated has to be valid for the used `journal_output`
    /// sha256(tokenmetadata) == journal_output
    /// the verfication would fail if this condition is not met
    pub fn generate_default_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("1800368936443379989".to_string()),
            description: Some("🎧 @ryan_soury touches on how Log Store captures data from Streamr for verifiable integration into Kwil, a credibly neutral database.\n📡Real-time data in, transparent SQL out.\nThis integration creates a secure turnkey platform for advanced DePINs like @PowerPod_People 🚗 https://t.co/gtuEQeWCZK".to_string()),
            media: None,
            media_hash: None,
            copies: None,
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: Some("{\"retweet_count\":2,\"reply_count\":1,\"like_count\":7,\"quote_count\":0,\"bookmark_count\":0,\"impression_count\":357}".to_string()),
            reference: None,
            reference_hash: None,
        }
    }
}

#[derive(Debug)]
pub struct RISC0VerifierConstructor(pub ContractConstructor);

#[derive(Debug)]
pub struct RISC0Verifier(pub DeployedContract);

impl RISC0VerifierConstructor {
    pub fn load() -> Self {
        let contract_path = Path::new(RES_PATH).join("RiscZeroGroth16Verifier.json");
        Self(ContractConstructor::from_extended_json(contract_path))
    }

    /// Creates the bytes that are used as the input to an EVM transaction for deploying
    /// the Factory contract. This function does not interact with any EVM itself, it only
    /// produces the bytes needed to pass to an EVM.
    pub fn create_deploy_bytes(&self) -> Vec<u8> {
        // @dev these parameters can be gotten from the zkaf repo at lib/risc0-ethereum/contracts/src/groth16/ControlID.sol
        let control_root =
            hex::decode("a516a057c9fbf5629106300934d48e0e775d4230e41e503347cad96fcbde7e2e")
                .unwrap();
        let bn254_control_id =
            hex::decode("0eb6febcf06c5df079111be116f79bd8c7e85dc9448776ef9a59aaf2624ab551")
                .unwrap();

        self.0.create_deploy_bytes_with_args(&[
            ethabi::Token::FixedBytes(control_root),
            ethabi::Token::FixedBytes(bn254_control_id),
        ])
    }

    pub fn deployed_at(self, address: Address) -> RISC0Verifier {
        RISC0Verifier(self.0.deployed_at(address))
    }
}

pub struct VerifierTestContext {
    pub proxy_account: workspaces::Account,
    pub verifier: Verifier,
    pub verifier_contract_address: Address,
}

impl VerifierTestContext {
    pub async fn new(aurora: AuroraEngine, proxy_account: workspaces::Account) -> Self {
        // deploy the risc0 verifier contract
        let risc0_contract_manager = RISC0VerifierConstructor::load();
        let risc0_contract_address = aurora
            .deploy_evm_contract(risc0_contract_manager.create_deploy_bytes())
            .await
            .unwrap();

        // deploy the actual verifier contract
        let verifier_contract_manager = VerifierConstructor::load();
        let verifier_contract_address = aurora
            .deploy_evm_contract(
                verifier_contract_manager.create_deploy_bytes(risc0_contract_address),
            )
            .await
            .unwrap();

        Self {
            proxy_account,
            verifier: verifier_contract_manager.deployed_at(verifier_contract_address),
            verifier_contract_address,
        }
    }
}


pub fn remove_quotes(input: &str) -> String {
    input.chars().filter(|&c| c != '\'' && c != '\"').collect()
}
