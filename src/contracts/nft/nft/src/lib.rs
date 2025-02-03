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
use near_contract_standards::non_fungible_token::events::NftMint;
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

const DATA_IMAGE_PNG: &str =
 "data:image/png;base64,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";

 
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
                name: "X NFTs".to_string(),
                symbol: "XNFTS".to_string(),
                icon: Some(DATA_IMAGE_PNG.to_string()),
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
                Some(receiver_id.clone()),
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
            self.claim_funds(token_id.clone(), request, MintRequestStatus::IsFulfilled);
            NftMint{
                owner_id: &receiver_id,
                token_ids: &[&token_id],
                memo: None,
            }.emit();
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

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

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
