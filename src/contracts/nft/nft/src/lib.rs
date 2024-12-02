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
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CostPerMetric {
    retweet_count: u128,
    reply_count: u128,
    like_count: u128,
    quote_count: u128,
    bookmark_count: u128,
    impression_count: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InputMintRequest {
    tweet_id: String,
    image_url: String,
    notify: String,
    public_metric: CostPerMetric,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    tweet_requests: LookupMap<String, (AccountId, u64, String, String)>,
    lock_time: u64,
    agent_verification_key: Vec<u8>,
    cost_per_metric: CostPerMetric,
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
}

const MIN_DEPOSIT: Balance = 20000000000000000000000;

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, agent_verification_key: String) -> Self {
        let key_hex = hex::decode(agent_verification_key);
        require!(key_hex.is_ok(), "Failed to parse hex String");
        let verification_key = key_hex.unwrap();
        require!(
            VerifyingKey::from_sec1_bytes(verification_key.as_slice()).is_ok(),
            "Invalid Verification Key"
        );
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
            verification_key.to_vec(),
            CostPerMetric {
                retweet_count: 50000000,
                reply_count: 10000000000,
                like_count: 200000000000000000,
                quote_count: 0,
                bookmark_count: 0,
                impression_count: 0,
            },
        )
    }

    #[init]
    pub fn new(
        owner_id: AccountId,
        metadata: NFTContractMetadata,
        agent_verification_key: Vec<u8>,
        cost_per_metric: CostPerMetric,
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
            agent_verification_key,
            cost_per_metric,
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
        token_metadata: TokenMetadata,
    ) -> Token {
        assert_eq!(
            env::predecessor_account_id(),
            self.tokens.owner_id,
            "NOT OWNER"
        );
        let token = self.tokens.internal_mint_with_refund(
            token_id,
            receiver_id.clone(),
            Some(token_metadata),
            Some(receiver_id),
        );
        self.tweet_requests.remove(&token.token_id);
        token
    }

    #[payable]
    pub fn mint_tweet_request(
        &mut self,
        input_data: String,
        signature: Vec<u8>,
    ) -> (AccountId, u64, String) {
        
        let input: InputMintRequest = serde_json::from_str(&input_data).unwrap();
        let signature = Signature::from_slice(&signature.as_slice());
        require!(signature.is_ok(), "Invalid signature");
        let signature = signature.unwrap();
        let verifying_key = VerifyingKey::from_sec1_bytes(self.agent_verification_key.as_slice())
            .ok()
            .expect("verifying_key must be correct");

        require!(
            env::attached_deposit().ge(&(MIN_DEPOSIT + self.compute_cost(input.public_metric))),
            format!(
                "Minimum deposit Not met of {}, you attached {}",
                &MIN_DEPOSIT,
                env::attached_deposit()
            )
        );

        assert!(verifying_key
            .verify(input_data.as_bytes(), &signature)
            .is_ok());

        if input.tweet_id.clone().parse::<u64>().is_err() {
            env::panic_str("tweet_id must be a positive number");
        }
        if self.tokens.owner_by_id.get(&input.tweet_id).is_some() {
            env::panic_str("tweet_id has been minted already");
        }

        if !self.is_tweet_available(input.tweet_id.clone()) {
            env::panic_str("This tweet_id has a lock on it");
        }
        // Get the signer's account ID
        let signer_account_id = env::predecessor_account_id();
        let now = env::block_timestamp_ms();
        let entry = (signer_account_id, now, input.image_url, input.notify);
        self.tweet_requests.insert(&input.tweet_id, &entry);

        // Log an event-like message
        let event = TweetMintRequest {
            tweet_id: input.tweet_id, // You might want to generate a unique ID here
            account: env::predecessor_account_id(),
            deposit: env::attached_deposit(),
            notify: entry.3,
        };
        event.emit();

        (entry.0, entry.1, entry.2)
    }

    pub fn cancel_mint_request(&mut self, tweet_id: String) {
        let tweet_request = self.tweet_requests.get(&tweet_id);
        if let Some((_id, timestamp, _, _)) = tweet_request {
            require!(
                env::block_timestamp_ms() - timestamp >= self.get_lock_time(),
                format!("CANT cancel until {}ms has PASSED", self.get_lock_time())
            );
            self.claim_funds(tweet_id);
        }
    }

    pub fn get_request(&self, tweet_id: String) -> Option<(AccountId, u64, String, String)> {
        self.tweet_requests.get(&tweet_id)
    }

    fn is_tweet_available(&self, tweet_id: String) -> bool {
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
            Some((_, timestamp, _, _)) => {
                env::block_timestamp_ms() - timestamp > self.get_lock_time()
            }
            None => true,
        }
    }

    pub fn get_lock_time(&self) -> u64 {
        self.lock_time
    }

    #[private]
    pub fn claim_funds(&mut self, tweet_id: String) {
        if let Some((id, _, _, _)) = self.tweet_requests.get(&tweet_id) {
            Promise::new(id).transfer(MIN_DEPOSIT);
            self.tweet_requests.remove(&tweet_id);
            let event = CancelMintRequest {
                tweet_id: tweet_id, // You might want to generate a unique ID here
                account: env::predecessor_account_id(),
                withdraw: MIN_DEPOSIT,
            };
            event.emit();
        }
    }

    #[private]
    pub fn set_agent_verification_key(&mut self, agent_verification_key: String) {
        let key_hex = hex::decode(agent_verification_key);
        require!(key_hex.is_ok(), "Failed to parse hex String");
        let verification_key = key_hex.unwrap();
        require!(
            VerifyingKey::from_sec1_bytes(verification_key.as_slice()).is_err(),
            "Invalid Verification Key"
        );
        self.agent_verification_key = verification_key.to_vec();
    }

    #[private]
    pub fn set_cost_per_metric(&mut self, cost_per_metric: CostPerMetric) {
        self.cost_per_metric = cost_per_metric;
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

    pub fn compute_cost(&mut self, public_metrics: CostPerMetric) -> u128 {
        let cost_per_metric = self.cost_per_metric.clone();
        cost_per_metric.bookmark_count * public_metrics.bookmark_count
            + cost_per_metric.impression_count * public_metrics.impression_count
            + cost_per_metric.like_count * public_metrics.like_count
            + cost_per_metric.quote_count * public_metrics.quote_count
            + cost_per_metric.reply_count * public_metrics.reply_count
            + cost_per_metric.retweet_count * public_metrics.retweet_count
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
    use hex;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use p256::ecdsa::{
        signature::Signer, signature::Verifier, Signature, SigningKey, VerifyingKey,
    };
    use p256::elliptic_curve::rand_core::OsRng;
    use std::collections::HashMap;
    use std::time::SystemTime;

    use super::*;

    const MINT_STORAGE_COST: u128 = 5870000000000000000000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn sample_token_metadata() -> TokenMetadata {
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
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    fn build_with_signature(tweet_id: &str, secret_key: SigningKey) -> (String, Vec<u8>) {
        let input = InputMintRequest {
            tweet_id: tweet_id.to_string(),
            image_url: format!("ipfs://"),
            notify: "".to_string(),
            public_metric: CostPerMetric {
                retweet_count: 64,
                reply_count: 64,
                like_count: 64,
                quote_count: 64,
                bookmark_count: 64,
                impression_count: 64,
            },
        };
        let input_data = serde_json::to_string(&input).unwrap();

        let _signature: Signature = secret_key.sign(&input_data.as_bytes());
        (input_data.to_owned(), _signature.to_vec())
    }

    #[test]
    fn test_signature() {
        let secret_key = SigningKey::random(&mut OsRng);
        let verifying_key: VerifyingKey = secret_key.verifying_key().to_owned();

        // Sign message
        let message = b"Hello, ECDSA!";
        let _signature: Signature = secret_key.sign(&message.clone());
        println!("signature valid?: {}", _signature);
        println!("verifying_key valid?: {:?}", verifying_key.to_sec1_bytes());
        println!("secret_key valid?: {:?}", secret_key.to_bytes());
        assert!(verifying_key.verify(message, &_signature).is_ok());
    }

    #[test]
    fn test_determinist_signature() {
        let secret_byte: [u8; 32] = [
            36, 134, 238, 249, 169, 144, 14, 88, 156, 220, 33, 126, 123, 186, 185, 21, 54, 141,
            121, 240, 254, 139, 234, 203, 21, 204, 88, 53, 28, 215, 138, 101,
        ];

        println!("secret_byte valid?: {}", hex::encode(secret_byte.clone()));

        let secret_key = SigningKey::from_slice(&secret_byte)
            .ok()
            .expect("Secret key parsing to be successful");
        let verifying_key: VerifyingKey = secret_key.verifying_key().to_owned();

        // Sign message
        let message = b"Hello, ECDSA!";
        let _signature: Signature = secret_key.sign(&message.clone());
        println!("signature valid?: {}", _signature);
        println!(
            "verifying_key valid?: {:?}",
            verifying_key.clone().to_sec1_bytes()
        );
        println!(
            "verifying_key valid?: {}",
            hex::encode(verifying_key.to_sec1_bytes())
        );
        println!("secret_key valid?: {:?}", secret_key.to_bytes());
        assert!(verifying_key.verify(message, &_signature).is_ok());
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(
            accounts(1).into(),
            "04e3350fbfb92fa0d761acdfc09610283cf977cf8254d6a93c7d8fc697f62df262ad10b284fbd77acc701776be9c256b21e414c574d1af638f430c9fb073254ec8".to_string(),
        );
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
    fn test_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(
            accounts(0).into(),
            "04e3350fbfb92fa0d761acdfc09610283cf977cf8254d6a93c7d8fc697f62df262ad10b284fbd77acc701776be9c256b21e414c574d1af638f430c9fb073254ec8".to_string(),
        );

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "0".to_string();
        let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id.to_string(), accounts(0).to_string());
        assert_eq!(token.metadata.unwrap(), sample_token_metadata());
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
    }

    #[test]
    fn test_get_lock_time() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(
            accounts(0).into(),
            "04e3350fbfb92fa0d761acdfc09610283cf977cf8254d6a93c7d8fc697f62df262ad10b284fbd77acc701776be9c256b21e414c574d1af638f430c9fb073254ec8".to_string(),
        );

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
        let contract = Contract::new_default_meta(
            accounts(0).into(),
            "04e3350fbfb92fa0d761acdfc09610283cf977cf8254d6a93c7d8fc697f62df262ad10b284fbd77acc701776be9c256b21e414c574d1af638f430c9fb073254ec8".to_string(),
        );

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
        let signer =
            hex::decode("2486eef9a9900e589cdc217e7bbab915368d79f0fe8beacb15cc58351cd78a65")
                .unwrap();
        let secret_key = SigningKey::from_slice(signer.as_slice())
            .ok()
            .expect("Secret key parsing need to be successful");
        let verifying_key: VerifyingKey = secret_key.verifying_key().to_owned();
        println!("verifying_key valid?: {:?}", verifying_key.to_sec1_bytes());
        let mut contract = Contract::new_default_meta(
            accounts(0).into(),
            hex::encode(verifying_key.to_sec1_bytes()),
        );

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(3))
            .block_timestamp(current_time.as_nanos() as u64)
            .build());

        // mint request
        let tweet_id = "XXX4071245224308850";
        let (input_data, _signature) = build_with_signature(tweet_id, secret_key);
        let entry = contract.mint_tweet_request(input_data, _signature);
        assert_eq!(entry.0, accounts(3));
    }
    #[test]
    #[should_panic(expected = "This tweet_id has a lock on it")]
    fn test_duplicate_mint_tweet_request() {
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let signer =
            hex::decode("2486eef9a9900e589cdc217e7bbab915368d79f0fe8beacb15cc58351cd78a65")
                .unwrap();
        let secret_key = SigningKey::from_slice(signer.as_slice())
            .ok()
            .expect("Secret key parsing need to be successful");
        let verifying_key: VerifyingKey = secret_key.verifying_key().to_owned();
        println!("verifying_key valid?: {:?}", verifying_key.to_sec1_bytes());
        let mut contract = Contract::new_default_meta(
            accounts(0).into(),
            hex::encode(verifying_key.to_sec1_bytes()),
        );
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(3))
            .block_timestamp(current_time.as_nanos() as u64)
            .build());

        // mint request
        let tweet_id = "1834071245224308850";
        let (input_data, _signature) = build_with_signature(tweet_id, secret_key);
        let entry = contract.mint_tweet_request(input_data.clone(), _signature.clone());
        assert_eq!(entry.0, accounts(3));
        assert_eq!(entry.1, current_time.as_millis() as u64);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(5))
            .block_timestamp(current_time.as_nanos() as u64)
            .build());

        let entry = contract.mint_tweet_request(input_data, _signature);
        assert_eq!(entry.0, accounts(3));
    }

    #[test]
    fn test_mint_tweet_request() {
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let signer =
            hex::decode("2486eef9a9900e589cdc217e7bbab915368d79f0fe8beacb15cc58351cd78a65")
                .unwrap();
        let secret_key = SigningKey::from_slice(signer.as_slice())
            .ok()
            .expect("Secret key parsing need to be successful");
        let verifying_key: VerifyingKey = secret_key.verifying_key().to_owned();
        println!("verifying_key valid?: {:?}", verifying_key.to_sec1_bytes());
        let mut contract = Contract::new_default_meta(
            accounts(0).into(),
            hex::encode(verifying_key.to_sec1_bytes()),
        );
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(3))
            .block_timestamp(current_time.as_nanos() as u64)
            .build());

        // mint request
        let tweet_id = "1862859048518844688";
        let (input_data, _signature) = build_with_signature(tweet_id, secret_key.clone());

        let entry = contract.mint_tweet_request(input_data, _signature);
        assert_eq!(entry.0, accounts(3));
        assert_eq!(entry.1, current_time.as_millis() as u64);

        let offset_sec = 1;
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(4))
            .block_timestamp(
                (current_time.as_nanos() as u64)
                    + (contract.get_lock_time() + offset_sec) * 1_000_000
            )
            .build());

        let (input_data, _signature) = build_with_signature(tweet_id, secret_key);
        let entry = contract.mint_tweet_request(input_data, _signature);
        assert_eq!(entry.0, accounts(4));
    }

    #[test]
    #[should_panic(expected = "NOT OWNER")]
    fn test_update_lock_time_other_user() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let signer =
            hex::decode("2486eef9a9900e589cdc217e7bbab915368d79f0fe8beacb15cc58351cd78a65")
                .unwrap();
        let secret_key = SigningKey::from_slice(signer.as_slice())
            .ok()
            .expect("Secret key parsing need to be successful");
        let verifying_key: VerifyingKey = secret_key.verifying_key().to_owned();
        println!("verifying_key valid?: {:?}", verifying_key.to_sec1_bytes());
        let mut contract = Contract::new_default_meta(
            accounts(0).into(),
            hex::encode(verifying_key.to_sec1_bytes()),
        );

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
        let signer =
            hex::decode("2486eef9a9900e589cdc217e7bbab915368d79f0fe8beacb15cc58351cd78a65")
                .unwrap();
        let secret_key = SigningKey::from_slice(signer.as_slice())
            .ok()
            .expect("Secret key parsing need to be successful");
        let verifying_key: VerifyingKey = secret_key.verifying_key().to_owned();
        println!("verifying_key valid?: {:?}", verifying_key.to_sec1_bytes());
        let mut contract = Contract::new_default_meta(
            accounts(0).into(),
            hex::encode(verifying_key.to_sec1_bytes()),
        );

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
        let signer =
            hex::decode("2486eef9a9900e589cdc217e7bbab915368d79f0fe8beacb15cc58351cd78a65")
                .unwrap();
        let secret_key = SigningKey::from_slice(signer.as_slice())
            .ok()
            .expect("Secret key parsing need to be successful");
        let verifying_key: VerifyingKey = secret_key.verifying_key().to_owned();
        println!("verifying_key valid?: {:?}", verifying_key.to_sec1_bytes());
        let mut contract = Contract::new_default_meta(
            accounts(0).into(),
            hex::encode(verifying_key.to_sec1_bytes()),
        );

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

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
            assert_eq!(token.metadata.unwrap(), sample_token_metadata());
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let signer =
            hex::decode("2486eef9a9900e589cdc217e7bbab915368d79f0fe8beacb15cc58351cd78a65")
                .unwrap();
        let secret_key = SigningKey::from_slice(signer.as_slice())
            .ok()
            .expect("Secret key parsing need to be successful");
        let verifying_key: VerifyingKey = secret_key.verifying_key().to_owned();
        println!("verifying_key valid?: {:?}", verifying_key.to_sec1_bytes());
        let mut contract = Contract::new_default_meta(
            accounts(0).into(),
            hex::encode(verifying_key.to_sec1_bytes()),
        );

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

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
        let signer =
            hex::decode("2486eef9a9900e589cdc217e7bbab915368d79f0fe8beacb15cc58351cd78a65")
                .unwrap();
        let secret_key = SigningKey::from_slice(signer.as_slice())
            .ok()
            .expect("Secret key parsing need to be successful");
        let verifying_key: VerifyingKey = secret_key.verifying_key().to_owned();
        println!("verifying_key valid?: {:?}", verifying_key.to_sec1_bytes());
        let mut contract = Contract::new_default_meta(
            accounts(0).into(),
            hex::encode(verifying_key.to_sec1_bytes()),
        );
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

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
        let signer =
            hex::decode("2486eef9a9900e589cdc217e7bbab915368d79f0fe8beacb15cc58351cd78a65")
                .unwrap();
        let secret_key = SigningKey::from_slice(signer.as_slice())
            .ok()
            .expect("Secret key parsing need to be successful");
        let verifying_key: VerifyingKey = secret_key.verifying_key().to_owned();
        println!("verifying_key valid?: {:?}", verifying_key.to_sec1_bytes());
        let mut contract = Contract::new_default_meta(
            accounts(0).into(),
            hex::encode(verifying_key.to_sec1_bytes()),
        );

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

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
