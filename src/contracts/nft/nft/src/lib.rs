/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
mod events;

use crate::events::TweetMintRequest;
use events::CancelMintRequest;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_tools::standard::nep297::Event;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{
    env, near_bindgen, require, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise,
    PromiseOrValue,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone, Copy)]
pub struct PublicMetric {
    bookmark_count: u128,
    impression_count: u128,
    like_count: u128,
    quote_count: u128,
    reply_count: u128,
    retweet_count: u128,
}

#[derive(Serialize, Deserialize)]
pub struct NFtMetaDataExtra {
    minted_to: String,
    public_metric: PublicMetric,
    author_id: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Clone)]
enum MintRequestStatus {
    Created,
    Cancelled,
    Unsuccessful,
    IsFulfilled,
    RoyaltyClaimed,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq)]
enum RoyaltyOperation {
    Increase,
    Decrease,
    Erase,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
pub struct MintRequestData {
    minter: AccountId,
    lock_time: u64,
    claimable_deposit: Balance,
    status: MintRequestStatus,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    tweet_requests: LookupMap<String, MintRequestData>,
    lock_time: u64,
    min_deposit: Balance,
    price_per_point: Balance,
    // NOTE DENOMINATOR is 10e6
    cost_per_metric: PublicMetric,

    royalty_manager: AccountId,
    royalty_balances: LookupMap<String, Balance>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str =
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    TweetRequests,
    RoyaltyBalances,
}

const PRICE_PER_POINT: Balance = 2000000000000000000000;

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, royalty_manager: AccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "USHER NEAR non-fungible token".to_string(),
                symbol: "USHER".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
            royalty_manager,
        )
    }

    #[init]
    pub fn new(
        owner_id: AccountId,
        metadata: NFTContractMetadata,
        royalty_manager: AccountId,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            tweet_requests: LookupMap::new(StorageKey::TweetRequests),
            lock_time: 30 * 60 * 1000,

            // Min deposit is calculated dynamically based on storage cost.
            min_deposit: env::storage_byte_cost() * 1024,
            price_per_point: PRICE_PER_POINT,

            // NOT DENOMINATOR 10e6
            cost_per_metric: PublicMetric {
                bookmark_count: 1190000,
                impression_count: 100,
                like_count: 500000,
                quote_count: 5000000,
                reply_count: 2000000,
                retweet_count: 1400000,
            },

            royalty_manager,
            royalty_balances: LookupMap::new(StorageKey::RoyaltyBalances),
        }
    }

    /// Mint a new token with ID=`token_id` belonging to `receiver_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. `self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
    /// initialization call to `new`.
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: AccountId,
        mut token_metadata: TokenMetadata,
    ) -> Token {
        // Get the mint request for the given token ID
        let mut request = self
            .get_request(token_id.clone())
            .expect("Invalid: No mint Request Found");

        // This token metadata is passed in from the verifier contract.
        // Veriifer contract is responsible for verifying the input metadata matches the zkVerified metadata
        let extra: NFtMetaDataExtra =
            serde_json::from_str(&token_metadata.clone().extra.expect("nft extra must exit"))
                .unwrap();

        // Check if the caller is the owner of the contract
        require!(
            env::predecessor_account_id().eq(&self.tokens.owner_id),
            "NOT OWNER"
        );

        // Check if the request has enough deposit to cover costs
        if request
            .claimable_deposit
            .ge(&self.compute_cost(extra.public_metric.clone()))
        {
            // Create extra metadata for the NFT
            let json_extra = json!([
                    {
                        "trait_type": "website",
                        "display_type": "website",
                        "value": format!("https://x.com/x/status/{}",&token_id),
                      },
                      {
                        "trait_type": "text",
                        "display_type": "metadata",
                        "value": token_metadata.extra,
                      },

            ])
            .to_string();

            // Update the extra metadata in the token metadata
            token_metadata.extra = Some(json_extra);

            // Mint the NFT
            let token = self.tokens.internal_mint_with_refund(
                token_id.clone(),
                receiver_id.clone(),
                Some(token_metadata.clone()),
                Some(receiver_id),
            );

            // Calculate refund amount
            let refund_amount =
                request.claimable_deposit - (&self.compute_cost(extra.public_metric.clone()));
            let value = request.claimable_deposit
                - &self.min_deposit
                - &refund_amount
                - (env::used_gas().0 as u128);

            // We're allocating 80% of the deposit to the author.
            // 20% is used to cover the cost of the minting.
            // 8 / 10 = 80 / 100
            // The remaining 20% of the value remains in the contract.
            self.royalty_operation(extra.author_id, value * 8 / 10, RoyaltyOperation::Increase);

            // Update request status and refund amount
            request.claimable_deposit = refund_amount;
            self.tweet_requests.insert(&token.token_id, &request);
            self.claim_funds(token_id, request, MintRequestStatus::IsFulfilled);
            return token;
        } else {
            // penalize user by decreasing Claimable Balance
            self.claim_funds(token_id, request.clone(), MintRequestStatus::Cancelled);
            env::panic_str(&format!(
                "Minimum deposit Not met of {}, you attached {} while minting.",
                self.compute_cost(extra.public_metric),
                request.claimable_deposit
            ))
        }
    }

    #[payable]
    pub fn mint_tweet_request(
        &mut self,
        tweet_id: String,
        image_url: String,
        notify: String,
    ) -> MintRequestData {
        require!(
            env::attached_deposit().ge(&self.min_deposit),
            format!(
                "Minimum deposit Not met of {}, you attached {}",
                &self.min_deposit,
                env::attached_deposit()
            )
        );
        if tweet_id.clone().parse::<u64>().is_err() {
            env::panic_str("tweet_id must be a positive number");
        }
        if self.tokens.owner_by_id.get(&tweet_id).is_some() {
            env::panic_str("tweet_id has been minted already");
        }

        if !self.is_tweet_available(tweet_id.clone()) {
            env::panic_str("This tweet_id has a lock on it");
        }

        let entry = MintRequestData {
            // Get the signer's account ID
            minter: env::predecessor_account_id(),
            //Current Block Time
            lock_time: env::block_timestamp_ms(),

            claimable_deposit: env::attached_deposit(),
            status: MintRequestStatus::Created,
        };
        self.tweet_requests.insert(&tweet_id, &entry);

        // Log an event-like message
        let event = TweetMintRequest {
            tweet_id: tweet_id, // You might want to generate a unique ID here
            account: env::predecessor_account_id(),
            deposit: env::attached_deposit(),
            image_url,
            notify: notify,
        };
        event.emit();

        entry
    }

    #[payable]
    pub fn cancel_mint_request(&mut self, tweet_id: String) {
        let tweet_request = self.tweet_requests.get(&tweet_id);
        if let Some(mint_request) = tweet_request {
            require!(
                env::block_timestamp_ms() - mint_request.lock_time >= self.get_lock_time(),
                format!("CANT cancel until {}ms has PASSED", self.get_lock_time())
            );
            require!(
                mint_request.minter.eq(&env::predecessor_account_id()),
                "You cant cancel A mint intent you didn't create"
            );
            self.claim_funds(tweet_id, mint_request, MintRequestStatus::Unsuccessful);
        }
    }

    pub fn get_request(&self, tweet_id: String) -> Option<MintRequestData> {
        self.tweet_requests.get(&tweet_id)
    }

    #[private]
    fn is_tweet_available(&mut self, tweet_id: String) -> bool {
        let entry = self.tweet_requests.get(&tweet_id);

        if self
            .tokens
            .owner_by_id
            .get(&format!("{}", tweet_id))
            .is_some()
        {
            return false;
        }
        //replace env::block_timestamp with
        match entry {
            Some(mint_request) => {
                if env::block_timestamp_ms() - mint_request.lock_time > self.get_lock_time() {
                    self.claim_funds(
                        tweet_id.clone(),
                        mint_request,
                        MintRequestStatus::Unsuccessful,
                    );
                    return true;
                }
                return false;
            }
            None => true,
        }
    }

    #[private]
    fn royalty_operation(
        &mut self,
        author_id: String,
        amount: Balance,
        operation: RoyaltyOperation,
    ) {
        if operation == RoyaltyOperation::Erase {
            self.royalty_balances.insert(&author_id, &0);
        } else if self.royalty_balances.contains_key(&author_id) {
            let balance = self.royalty_balances.get(&author_id).unwrap();
            match operation {
                // This is never called.
                // It's added because Storage settings are immutable and cannote be changed.
                // However, the logic of the contract can be upgraded.
                RoyaltyOperation::Decrease => {
                    if balance < amount {
                        env::panic_str(
                            format!(
                                "Amount Request {} is greater than royalty {}",
                                amount, balance
                            )
                            .as_str(),
                        )
                    } else {
                        self.royalty_balances
                            .insert(&author_id, &(balance - amount));
                    }
                }
                RoyaltyOperation::Increase => {
                    // We're simply adding a an amount in a balance map for authors by their X id
                    self.royalty_balances
                        .insert(&author_id, &(balance + amount));
                }
                // Once on-chain payouts are integrated, balances can be automatically erased on payout
                RoyaltyOperation::Erase => {}
            }
        } else {
            if RoyaltyOperation::Increase == operation {
                self.royalty_balances.insert(&author_id, &amount);
            } else {
                env::panic_str(format!("Invalid operation {} has no royalty", author_id).as_str())
            }
        }
    }

    pub fn royalty_withdraw(&mut self, amount: Balance) {
        // Check ensures that the royalty manager does not need to be deployer/AccountID of the Contract.
        require!(
            env::predecessor_account_id() == self.royalty_manager,
            "Insufficient Access"
        );
        let storage_cost = u128::from(env::storage_usage()) * env::storage_byte_cost();
        if env::account_balance() - storage_cost >= amount {
            Promise::new(self.royalty_manager.clone()).transfer(amount);
        } else {
            env::panic_str(
                format!(
                    "Invalid Amount: Claimable Account Balance: {}",
                    storage_cost - env::account_balance()
                )
                .as_str(),
            )
        }
    }

    pub fn update_royalty_manager(&mut self, account: AccountId) {
        // Check ensures that the royalty manager does not need to be deployer/AccountID of the Contract.
        require!(
            env::predecessor_account_id() == self.royalty_manager,
            "Insufficient Access"
        );
        self.royalty_manager = account
    }

    pub fn get_lock_time(&self) -> u64 {
        self.lock_time
    }

    #[private]
    fn claim_funds(
        &mut self,
        tweet_id: String,
        mint_request: MintRequestData,
        status: MintRequestStatus,
    ) {
        if status == MintRequestStatus::IsFulfilled {
            Promise::new(mint_request.minter.clone()).transfer(mint_request.claimable_deposit);
            self.tweet_requests.insert(
                &tweet_id,
                &MintRequestData {
                    minter: mint_request.minter,
                    lock_time: mint_request.lock_time,
                    claimable_deposit: 0,
                    status: MintRequestStatus::IsFulfilled,
                },
            );
        } else if status == MintRequestStatus::Cancelled
            || status == MintRequestStatus::Unsuccessful
        {
            let amount = if status == MintRequestStatus::Cancelled {
                mint_request.claimable_deposit * 9 / 10 // Transferring 90% of the origin deposit back to the minter.
            } else {
                mint_request.claimable_deposit // Else if unsuccessful, transferring 100% of the origin deposit back to the minter.
            };
            Promise::new(mint_request.minter.clone()).transfer(amount);
            self.tweet_requests.remove(&tweet_id);
            let event = CancelMintRequest {
                tweet_id: tweet_id, // You might want to generate a unique ID here
                account: env::predecessor_account_id(),
                // TODO: Set the withdraw amount once, and re-use
                withdraw: amount,
            };
            event.emit();
        }
    }

    pub fn update_lock_time(&mut self, new_value: u64) -> u64 {
        require!(
            env::predecessor_account_id() == self.tokens.owner_id,
            "NOT OWNER"
        );
        self.lock_time = new_value;
        // Log an event-like message
        env::log_str(format!("lock_time updated: {}", new_value).as_str());
        new_value
    }

    // NOT DENOMINATOR 10e6
    #[private]
    pub fn set_cost_per_metric(&mut self, cost_per_metric: PublicMetric) {
        self.cost_per_metric = cost_per_metric;
    }

    // This function is called internally, and externally.
    // External calls are used to determine the cost of minting an NFT.
    pub fn compute_cost(&self, public_metrics: PublicMetric) -> u128 {
        let cost_per_metric = self.cost_per_metric.clone();
        let cost = self.min_deposit
            + (self.price_per_point
                * (cost_per_metric.bookmark_count * public_metrics.bookmark_count
                    + cost_per_metric.impression_count * public_metrics.impression_count
                    + cost_per_metric.like_count * public_metrics.like_count
                    + cost_per_metric.quote_count * public_metrics.quote_count
                    + cost_per_metric.reply_count * public_metrics.reply_count
                    + cost_per_metric.retweet_count * public_metrics.retweet_count)
                / 1000000);
        if cost.lt(&self.min_deposit) {
            // The 5x is a buffer that allows the UX to determine what minimum it should deposit at the time of intent submission.
            // This prevents extremely early X posts that haven't accumulated any metrics from being rejected from minting.
            return self.min_deposit * 5;
        }
        cost
    }

    pub fn update_min_deposit(&mut self, min_deposit: Balance) {
        require!(
            env::predecessor_account_id() == self.tokens.owner_id,
            "NOT OWNER"
        );
        self.min_deposit = min_deposit;
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

// #[near_bindgen]
// impl NonFungibleTokenMetadataProvider for Contract {
//     fn nft_metadata(&self) -> NFTContractMetadata {
//         self.metadata.get().unwrap()
//     }
// }

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use std::collections::HashMap;
    use std::default;
    use std::time::SystemTime;

    use super::*;

    fn get_test_public_metrics(likes: u128) -> PublicMetric {
        PublicMetric {
            impression_count: 0,
            bookmark_count: 1,
            quote_count: 0,
            like_count: likes,
            reply_count: 0,
            retweet_count: 0,
        }
    }

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn sample_token_metadata(likes: u128) -> TokenMetadata {
        let json_string = r#"
        {"minted_to":"eclipse_interop.testnet","public_metric":{"bookmark_count":1,"impression_count":0,"like_count": like_count_num,"quote_count":0,"reply_count":0,"retweet_count":0},"author_id":"1234"}
        "#.replace("like_count_num",&likes.to_string());

        TokenMetadata {
            title: Some("Olympus Mons".into()),
            description: Some("The tallest mountain in the charted solar system".into()),
            media: None,
            media_hash: None,
            copies: Some(1u64),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: Some(json_string.into()),
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), accounts(5));
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.nft_token("1".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_update_min_deposit() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.compute_cost(get_test_public_metrics(1)))
            .predecessor_account_id(accounts(0))
            .build());

        let default_min_deposit = contract.min_deposit;

        contract.update_min_deposit(default_min_deposit * 2);

        assert_eq!(contract.min_deposit, default_min_deposit * 2);
    }

    #[test]
    #[should_panic(contains = "Minimum deposit Not met of")]
    fn test_invalid_nft_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        let likes: u128 = 1 as u128;

        let deposit = contract.compute_cost(get_test_public_metrics(likes));
        let balance = env::account_balance();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(deposit)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "1".to_string();
        contract.mint_tweet_request(token_id.clone(), format!("ipfs://"), "@xxxxxx".to_owned());
        let _ = contract.nft_mint(
            token_id.clone(),
            accounts(0),
            sample_token_metadata(likes + 1),
        );
        assert_eq!(balance, env::account_balance());
    }

    #[test]
    fn test_cancel_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));
        let _ = contract.update_lock_time(0);

        let likes: u128 = 1 as u128;

        let deposit = contract.compute_cost(get_test_public_metrics(likes));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(deposit)
            .predecessor_account_id(accounts(2))
            .build());

        let token_id = "1".to_string();
        contract.mint_tweet_request(token_id.clone(), format!("ipfs://"), "@xxxxxx".to_owned());

        let balance = env::account_balance();
        let _ = contract.cancel_mint_request(token_id.clone());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(deposit)
            .predecessor_account_id(accounts(2))
            .build());
        assert_eq!(balance, env::account_balance());
    }

    #[test]
    #[should_panic(contains = "Minimum deposit Not met of")]
    fn test_deposit_nft_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        let likes: u128 = 1 as u128;

        let deposit = contract.compute_cost(get_test_public_metrics(likes));
        let balance = env::account_balance();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(deposit)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "1".to_string();
        contract.mint_tweet_request(token_id.clone(), format!("ipfs://"), "@xxxxxx".to_owned());
        let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata(likes));

        // duplicated mint
        let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata(likes));

        assert_eq!(balance, env::account_balance());
    }

    #[test]
    fn test_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        let likes: u128 = 1 as u128;

        let deposit = contract.compute_cost(get_test_public_metrics(likes));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(deposit)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "0".to_string();
        contract.mint_tweet_request(
            token_id.clone(),
            format!("ipfs://"),
            "@xxxxxx".to_owned(),
            // get_test_public_metrics(1),
        );
        let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata(likes));
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id.to_string(), accounts(0).to_string());
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(5))
            .build());
        let balance = env::account_balance();
        contract.royalty_withdraw(deposit * 2 / 10);
        assert_eq!(balance, env::account_balance() + deposit * 2 / 10);
    }

    #[test]
    fn test_get_lock_time() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .is_view(true)
            .build());

        let time = contract.get_lock_time();
        assert_eq!(time, 30 * 60 * 1000);
    }

    #[test]
    fn test_is_valid_request() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .is_view(true)
            .build());

        // let tweet_id = "1834071245224308850".to_string();

        let random_tweet_id = format!("{}", env::random_seed().into_iter().sum::<u8>());
        let is_valid = contract.is_tweet_available(random_tweet_id);
        assert!(is_valid);
    }

    #[test]
    #[should_panic(expected = "tweet_id must be a positive number")]
    fn test_is_invalid_request() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.compute_cost(get_test_public_metrics(1)))
            .predecessor_account_id(accounts(3))
            .block_timestamp(current_time.as_nanos() as u64)
            .build());

        // mint request
        let tweet_id = "XXX4071245224308850";
        let entry = contract.mint_tweet_request(
            tweet_id.to_string(),
            format!("ipfs://"),
            format!(""),
            // get_test_public_metrics(1),
        );
        assert_eq!(entry.minter, accounts(3));
    }
    #[test]
    #[should_panic(expected = "This tweet_id has a lock on it")]
    fn test_duplicate_mint_tweet_request() {
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.compute_cost(get_test_public_metrics(1)))
            .predecessor_account_id(accounts(3))
            .block_timestamp(current_time.as_nanos() as u64)
            .build());

        // mint request
        let tweet_id = "1834071245224308850";
        let entry = contract.mint_tweet_request(
            tweet_id.to_string(),
            format!("ipfs://"),
            format!(""),
            // get_test_public_metrics(1),
        );
        assert_eq!(entry.minter, accounts(3));
        assert_eq!(entry.lock_time, current_time.as_millis() as u64);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.compute_cost(get_test_public_metrics(1)))
            .predecessor_account_id(accounts(5))
            .block_timestamp(current_time.as_nanos() as u64)
            .build());
        let entry = contract.mint_tweet_request(
            tweet_id.to_string(),
            format!("ipfs://"),
            format!(""),
            // get_test_public_metrics(1),
        );
        assert_eq!(entry.minter, accounts(3));
    }

    #[test]
    fn test_mint_tweet_request() {
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.compute_cost(get_test_public_metrics(1)))
            .predecessor_account_id(accounts(3))
            .block_timestamp(current_time.as_nanos() as u64)
            .build());

        // mint request
        let tweet_id = "1834071245224308850";
        let entry = contract.mint_tweet_request(
            tweet_id.to_string(),
            format!("ipfs://"),
            format!(""),
            // get_test_public_metrics(1),
        );
        assert_eq!(entry.minter, accounts(3));
        assert_eq!(entry.lock_time, current_time.as_millis() as u64);

        let offset_sec = 1;
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.compute_cost(get_test_public_metrics(1)))
            .predecessor_account_id(accounts(4))
            .block_timestamp(
                (current_time.as_nanos() as u64)
                    + (contract.get_lock_time() + offset_sec) * 1_000_000
            )
            .build());

        let entry = contract.mint_tweet_request(
            tweet_id.to_string(),
            format!("ipfs://"),
            format!(""),
            // get_test_public_metrics(1),
        );
        assert_eq!(entry.minter, accounts(4));
    }

    #[test]
    #[should_panic(expected = "NOT OWNER")]
    fn test_update_lock_time_other_user() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .predecessor_account_id(accounts(4))
            .build());

        contract.update_lock_time(1000000);
    }

    #[test]
    fn test_update_lock_time() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .predecessor_account_id(accounts(0))
            .build());

        let time = contract.update_lock_time(1000000);
        assert_eq!(time, contract.get_lock_time());
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        let likes: u128 = 1 as u128;

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.compute_cost(get_test_public_metrics(likes)))
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.mint_tweet_request(
            token_id.clone(),
            format!("ipfs://"),
            "@xxxxxx".to_owned(),
            // get_test_public_metrics(1),
        );
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata(likes));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), token_id.clone(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(token_id.clone()) {
            assert_eq!(token.token_id, token_id);
            assert_eq!(token.owner_id.to_string(), accounts(1).to_string());
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.compute_cost(get_test_public_metrics(1)))
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.mint_tweet_request(
            token_id.clone(),
            format!("ipfs://"),
            "@xxxxxx".to_owned(),
            // get_test_public_metrics(1),
        );
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata(1));

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }

    #[test]
    fn test_revoke() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.compute_cost(get_test_public_metrics(1)))
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.mint_tweet_request(
            token_id.clone(),
            format!("ipfs://"),
            "@xxxxxx".to_owned(),
            // get_test_public_metrics(1),
        );
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata(1));

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke(token_id.clone(), accounts(1));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
    }

    #[test]
    fn test_revoke_all() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into(), accounts(5));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.compute_cost(get_test_public_metrics(1)))
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.mint_tweet_request(
            token_id.clone(),
            format!("ipfs://"),
            "@xxxxxx".to_owned(),
            // get_test_public_metrics(1),
        );
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata(1));

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke_all(token_id.clone());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }
}
