use aurora_sdk::near_sdk::{Gas, PromiseError};
use aurora_sdk::{
    ethabi, near_sdk, Address, CallArgs, FunctionCallArgsV1, SubmitResult, TransactionStatus,
};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::Token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};

pub mod external;
pub use crate::external::*;

/// Selector for `isJournalVerified(bytes)`.
/// The value is computing by taking the first 4 bytes of the keccak hash of the type
/// signature for the function, see https://www.4byte.directory/signatures/?bytes4_signature=0xdb3e2198
const IS_JOURNAL_VERIFIED_SELECTOR: [u8; 4] = [ 181, 76, 30, 108 ];
const DEPOSIT:u128 = 15020000000000000000000;


#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct VerifierProxy {
    aurora: AccountId,
    contract_address: Address,
    nft_account_id: AccountId,
}


#[near_bindgen]
impl VerifierProxy {

    #[init]
    pub fn init(contract_address: String, nft_account_id: AccountId) -> Self {
        Self {
            // This value only needs to be changed if you are running the aurora testnet locally
            aurora:  "aurora".parse().unwrap(),
            contract_address: aurora_sdk::parse_address(&contract_address).unwrap(),
            nft_account_id
        }
    }

    pub fn get_nft_account_id(&self) -> AccountId {
        self.nft_account_id.clone()
    }

    pub fn get_verifier_address(&self) -> Address {
        self.contract_address.clone()
    }

    //make this payable and provide the metadata
    pub fn verify_proof(&self, journal: Vec<u8>, token_metadata: TokenMetadata) -> Promise{
        let journal_output = ethabi::Token::Bytes(journal.clone().into());

        // assert that the journal is equal to the sha256 hash of the stringified token metadata
        let stringified_token_metadata = serde_json::to_string(&token_metadata).unwrap();
        let stringified_token_metadata = stringified_token_metadata.as_bytes();
        
        let hashed_token_metadata = env::sha256(stringified_token_metadata);
        assert_eq!(hex::encode(hashed_token_metadata),hex::encode(journal.clone()), "invalid token_metadata");


        let evm_input = ethabi::encode(&[journal_output]);
        let aurora_call_args = CallArgs::V1(FunctionCallArgsV1 {
            contract: self.contract_address,
            input: [
                IS_JOURNAL_VERIFIED_SELECTOR.as_slice(),
                evm_input.as_slice(),
            ]
            .concat(),
        });

        aurora_sdk::aurora_contract::ext(self.aurora.clone())
            .with_unused_gas_weight(3)
            .call(aurora_call_args)
            .then(Self::ext(env::current_account_id()).parse_verification_response(token_metadata))
    }

    #[private]
    pub fn set_verifier_address(&mut self, new_contract_address: String) {
        self.contract_address = aurora_sdk::parse_address(&new_contract_address).unwrap();
    }

    // Callback used to parse the output from the call to Aurora made in `exact_output_single`.
    // TODO pass in the nft payload to this function and mint the NFT after this callback
    #[private]
    pub fn parse_verification_response(
        &mut self,
        token_metadata: TokenMetadata,
        #[serializer(borsh)]
        #[callback_unwrap]
        result: SubmitResult,
    ) -> Promise {
        match result.status {
            TransactionStatus::Succeed(bytes) => {
                // bytes is a vector of length 32, where the last bit is 1|0 depending on the truthy value
                // Parse only the last bit and use that to determine if this is true or false
                let is_valid = bytes.get(31).unwrap().clone() == 1;
                // if this proof is invalid then throw an error
                if !is_valid {env::panic_str("message")};

                // mint the NFT here after a successfull verification
                let token_id = token_metadata.title.clone().unwrap();
                let receiver_id = env::predecessor_account_id();
                return nft_contract::ext(self.nft_account_id.clone())
                    .with_static_gas(Gas(5_000_000_000_000))
                    .with_attached_deposit(DEPOSIT)
                    .nft_mint(
                        token_id,
                        receiver_id.clone(),
                        token_metadata.clone(),
                    )
                    .then(
                        // Create a callback change_greeting_callback
                        Self::ext(env::current_account_id())
                        .with_static_gas(Gas(5_000_000_000_000))
                        .nft_creation_callback(),
                    );

            }
            TransactionStatus::Revert(bytes) => {
                let error_message =
                    format!("Revert: {}", aurora_sdk::parse_evm_revert_message(&bytes));
                env::panic_str(&error_message)
            }
            other => env::panic_str(&format!("Aurora Error: {other:?}")),
        }
    }

    #[private]
    pub fn nft_creation_callback(
        &mut self,
        #[callback_result] call_result: Result<Token, PromiseError>,
    ) -> bool{
        // Return whether or not the promise succeeded using the method outlined in external.rs
        if call_result.is_err() {
            env::panic_str(format!("nft_creation failed:{:?}",call_result.err()).as_str());
        } else {
            env::log_str(format!("nft_creation was successful!").as_str());
            return true;
        }
    }

    
}
