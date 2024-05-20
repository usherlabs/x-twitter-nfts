use aurora_sdk::{
    ethabi, near_sdk, Address, CallArgs, FunctionCallArgsV1, SubmitResult, TransactionStatus, U256,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};
use uniswap_from_near_types::SerializableExactOutputSingleParams;

const DEFAULT_FEE: u64 = 500;
/// Selector for [exactOutputSingle](https://docs.uniswap.org/contracts/v3/reference/periphery/SwapRouter#exactoutputsingle).
/// The value is computing by taking the first 4 bytes of the keccak hash of the type
/// signature for the function, see https://www.4byte.directory/signatures/?bytes4_signature=0xdb3e2198
const EXACT_OUTPUT_SINGLE_SELECTOR: [u8; 4] = [103, 116, 54, 31]; // selector for `verify_proof()`
// const EXACT_OUTPUT_SINGLE_SELECTOR: [u8; 4] = [109, 76, 230, 60];

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct VerifierProxy {

}

#[near_bindgen]
impl VerifierProxy {

    pub fn get_greeting() -> String {
        "working".to_string()
    }

    pub fn verify_proof() -> Promise{
        // define dummy variables here
        // TODO pass as variables to the verify_proof function
        // let aurora: AccountId = "aurora".parse().unwrap();
        // let contract_address = "0x851b02B6f9DB2740A0f4aA9Da9bdfA920796E504".to_string();
        // let journal_output = hex::decode("4f8ad5ce1d0fc577d04d618880b8c77c9aced63740ca43b708f32425a95b11b7").unwrap();
        // let post_state_digest = hex::decode("0622b43513c8148548a5aa01d3842e669df2ed7cae1b498e98ae1a8b3aae1a14").unwrap();
        // let seal = hex::decode("1a61e905022e6727f77f55f5632d1b418040e236961222301b1b0df883d8124c225b443b81443529841cbd6845e65efc7a36547ff66003d5e94ac36f91ca83f42c81d508177c6b03d647ccc662e83ba94f0814f941dbef13dc374eea5897bd5a19578107ca5766c68e0a49b4576c0e3d30d4c82f5bdd34ffff81673d2141b6d6163b8ac07ddd56827824da2196046baade5cfd64dd3d0046a4faa1c0b2602f762a2d547b5bf7aaadca7c1f69f5a97eebf7f9c810fe420ab846081f7a8a16e1172735b48780dc334aaa3a8c4806c02a396746e6500cee439f8bf9b6de1c43a0e81345aac7a3fce0e8eed676b4307611b7b8e567f1e6f14333e29e032843a0fe1b").unwrap();

        // // // // TODO: Call aurora smart contract with the provided headers
        // let contract_address = aurora_sdk::parse_address(&contract_address).unwrap();
        
        // let journal_output = ethabi::Token::Bytes(journal_output.into());
        // let post_state_digest = ethabi::Token::FixedBytes(post_state_digest.into());
        // let seal = ethabi::Token::Bytes(seal.into());

        // let evm_input = ethabi::encode(&[journal_output,post_state_digest, seal]);
        let aurora_call_args = CallArgs::V1(FunctionCallArgsV1 {
            contract: contract_address,
            input: [
                EXACT_OUTPUT_SINGLE_SELECTOR.as_slice(),
                [].as_slice(),
            ]
            .concat(),
        });
        // TODO validate aurora deployment and contract call
        aurora_sdk::aurora_contract::ext(aurora.clone())
        .with_unused_gas_weight(3)
        // .with_unused_gas_weight(0)
        .call(aurora_call_args)
        .then(Self::ext(env::current_account_id()).parse_exact_output_single_result())
    }


    // /// Callback used to parse the output from the call to Aurora made in `exact_output_single`.
    #[private]
    pub fn parse_exact_output_single_result(
        #[serializer(borsh)]
        #[callback_unwrap]
        result: SubmitResult,
    ) -> () {
        match result.status {
            TransactionStatus::Succeed(bytes) => {
                // let amount_in = U256::from_big_endian(&bytes);
                // ExactOutputSingleResult {
                //     amount_in: amount_in.to_string(),
                // }
            }
            // TransactionStatus::Revert(bytes) => {
            //     let error_message =
            //         format!("Revert: {}", aurora_sdk::parse_evm_revert_message(&bytes));
            //     env::panic_str(&error_message)
            // }
            other => env::panic_str(&format!("Aurora Error: {other:?}")),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ExactOutputSingleResult {
    amount_in: String,
}

#[derive(Debug)]
struct ParseParamsError;

#[inline]
fn parse_address(input: &str) -> Result<Address, ParseParamsError> {
    aurora_sdk::parse_address(input).map_err(|_| ParseParamsError)
}

#[inline]
fn parse_u256(input: &str) -> Result<U256, ParseParamsError> {
    aurora_sdk::parse_u256_base10(input).map_err(|_| ParseParamsError)
}
