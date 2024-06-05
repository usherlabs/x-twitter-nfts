use aurora_sdk::{
    ethabi, near_sdk, Address, CallArgs, FunctionCallArgsV1, SubmitResult, TransactionStatus,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};

/// Selector for `isJournalVerified(bytes)`.
/// The value is computing by taking the first 4 bytes of the keccak hash of the type
/// signature for the function, see https://www.4byte.directory/signatures/?bytes4_signature=0xdb3e2198
const IS_JOURNAL_VERIFIED_SELECTOR: [u8; 4] = [ 181, 76, 30, 108 ];

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct VerifierProxy {
    aurora: AccountId,
    contract_address: Address,
}


#[near_bindgen]
impl VerifierProxy {

    #[init]
    pub fn init(contract_address: String) -> Self {
        Self {
            // This value only needs to be changed if you are running the aurora testnet locally
            aurora:  "aurora".parse().unwrap(),
            contract_address: aurora_sdk::parse_address(&contract_address).unwrap()
        }
    }


    pub fn get_verifier_address(&self) -> Address {
        self.contract_address.clone()
    }


    pub fn verify_proof(&self, journal_output: Vec<u8>) -> Promise{
        let journal_output = ethabi::Token::Bytes(journal_output.into());


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
            .then(Self::ext(env::current_account_id()).parse_exact_output_single_result())
    }

    #[private]
    pub fn set_verifier_address(&mut self, new_contract_address: String) {
        self.contract_address = aurora_sdk::parse_address(&new_contract_address).unwrap();
    }

    // Callback used to parse the output from the call to Aurora made in `exact_output_single`.
    #[private]
    pub fn parse_exact_output_single_result(
        #[serializer(borsh)]
        #[callback_unwrap]
        result: SubmitResult,
    ) -> () {
        match result.status {
            TransactionStatus::Succeed(bytes) => {
                // bytes is a vector of length 32, where the last bit is 1|0 depending on the truthy value
                // Parse only the last bit and use that to determine if this is true or false
                let is_valid = bytes.get(31).unwrap().clone() == 1;
                // if this proof is invalid then throw an error
                if !is_valid {env::panic_str("message")};
            }
            TransactionStatus::Revert(bytes) => {
                let error_message =
                    format!("Revert: {}", aurora_sdk::parse_evm_revert_message(&bytes));
                env::panic_str(&error_message)
            }
            other => env::panic_str(&format!("Aurora Error: {other:?}")),
        }
    }
}
