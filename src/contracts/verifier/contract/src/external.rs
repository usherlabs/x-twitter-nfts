use aurora_sdk::near_sdk;
use aurora_sdk::near_sdk::{ext_contract, AccountId};
// Find all our documentation at https://docs.near.org
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::TokenId;

pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;
pub const MINIMIM_DEPOSIT: u128 = 5120000000000000000000;

// Validator interface, for cross-contract calls
#[ext_contract(nft_contract)]
pub trait NFTContract {
    fn nft_mint(token_id: TokenId, receiver_id: AccountId, token_metadata: TokenMetadata);
}
