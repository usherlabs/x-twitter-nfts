use aurora_sdk::near_sdk::{Gas, PromiseError};
use aurora_sdk::{
    ecrecover, ethabi, near_sdk, Address, CallArgs, FunctionCallArgsV1, SubmitResult,
    TransactionStatus, H256,
};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::Token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha3::{Digest, Keccak256};

pub mod external;
pub use crate::external::*;

/// Selector for `isJournalVerified(bytes)`.
/// The value is computed by taking the first 4 bytes of the keccak hash of the type
/// signature for the function, see https://www.4byte.directory/signatures/?bytes4_signature=0xdb3e2198
const IS_JOURNAL_VERIFIED_SELECTOR: [u8; 4] = [181, 76, 30, 108];
const DEPOSIT: u128 = 15020000000000000000000;

/// The tweet structure obtained from the API
///
/// Contains the details about a tweet
#[derive(Debug, Deserialize, Serialize)]
pub struct Tweet {
    /// Data related to the tweet
    pub data: Vec<Data>,
    /// User information included with the tweet
    pub includes: Includes,
}

/// The data substructure of a tweet
///
/// Contains the details about a tweet
#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    /// Date the tweet was created
    pub created_at: String,
    /// Unique identifier for the tweet
    pub id: String,
    /// Public metrics associated with the tweet
    pub public_metrics: PublicMetrics,
    /// History of tweet edits
    pub edit_history_tweet_ids: Vec<String>,
    /// Author ID of the tweet
    pub author_id: String,
    /// Text content of the tweet
    pub text: String,
}

/// The PublicMetrics substructure of a tweet
///
/// Contains the details about a tweet's public metrics
#[derive(Debug, Deserialize, Serialize)]
pub struct PublicMetrics {
    /// Number of retweets
    pub retweet_count: u32,
    /// Number of replies
    pub reply_count: u32,
    /// Number of likes
    pub like_count: u32,
    /// Number of quotes
    pub quote_count: u32,
    /// Number of bookmarks
    pub bookmark_count: u32,
    /// Number of impressions
    pub impression_count: u32,
}

/// The Includes substructure of a metadata NFT
///
/// Contains the details about a metadata NFT
#[derive(Debug, Deserialize, Serialize)]
pub struct Includes {
    /// List of users associated with the tweet
    pub users: Vec<User>,
}

/// The User substructure of a tweet
///
/// Contains the details about a user associated with a tweet
#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    /// Unique identifier for the user
    pub id: String,
    /// Optional username of the user
    pub username: Option<String>, // Optional because it's not present in every user object
    /// Name of the user
    pub name: String,
    /// Date the user account was created
    pub created_at: String,
}

/// Generates a token metadata payload for a tweet NFT
///
/// # Arguments
///
/// * `json_tweet` - A JSON string representing the tweet
/// * `image_url` - URL of the image associated with the tweet
/// * `owner_account_id` - Account ID of the owner
///
/// # Returns
///
/// * `TokenMetadata` - The generated token metadata
pub fn generate_tweet_nft_payload(
    json_tweet: &str,
    image_url: String,
    owner_account_id: String,
) -> TokenMetadata {
    env::log_str(&format!("data:{}", json_tweet));

    // Deserialize the tweet JSON and extract the first tweet data and its public metrics
    let tweet: Tweet = serde_json::from_str(json_tweet).unwrap();
    let tweet_data = tweet.data.get(0).unwrap();
    let public_metric = &tweet_data.public_metrics;

    // Generate a token metadata object
    let token_metadata = TokenMetadata {
        title: Some(tweet_data.id.clone()), // Title of the NFT, using the tweet ID
        description: Some(tweet_data.text.clone()), // Description of the NFT, using the tweet text
        extra: Some(
            json!({
               "public_metric": public_metric,
               "minted_to":owner_account_id.clone(),
               "author_id":tweet_data.author_id.clone(),
               "user": (tweet.includes.users.get(0).unwrap_or(&User{
                    name: "".to_string(),
                    id:"".to_string(),
                    username: Some("".to_string()),
                    created_at:"".to_string()
                })).username
            })
            .to_string(),
        ), // Additional data stored on-chain, can be stringified JSON
        media: Some(image_url), // URL to associated media, preferably decentralized storage
        media_hash: None, // Base64-encoded sha256 hash of content referenced by the `media` field
        copies: Some(1), // Number of copies of this metadata in existence when token was minted
        issued_at: None, // ISO 8601 datetime when token was issued or minted
        expires_at: None, // ISO 8601 datetime when token expires
        starts_at: None, // ISO 8601 datetime when token starts being valid
        updated_at: None, // ISO 8601 datetime when token was last updated
        reference: None, // URL to an off-chain JSON file with more info
        reference_hash: None, // Base64-encoded sha256 hash of JSON referenced by the `reference` field
    };

    token_metadata
}

/// Converts a hexadecimal string (optionally prefixed with '0x') to a vector of bytes.
///
/// # Arguments
///
/// * `str` - A hexadecimal string
///
/// # Returns
///
/// * `Vec<u8>` - A vector of bytes representing the hexadecimal string
pub fn string_to_vec_u8(str: &str) -> Vec<u8> {
    let starts_from: usize;
    if str.starts_with("0x") {
        starts_from = 2;
    } else {
        starts_from = 0;
    }

    (starts_from..str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&str[i..i + 2], 16).unwrap())
        .collect::<Vec<u8>>()
}

/// Hashes an Ethereum message to prepare it for public key derivation
///
/// # Arguments
///
/// * `message` - The message to be hashed
///
/// # Returns
///
/// * `Vec<u8>` - The hashed message
fn hash_eth_message<T: AsRef<[u8]>>(message: T) -> Vec<u8> {
    const PREFIX: &str = "\x19Ethereum Signed Message:\n";

    let message = message.as_ref();
    let len = message.len();
    let len_string = len.to_string();

    let mut eth_message = Vec::with_capacity(PREFIX.len() + len_string.len() + len);
    eth_message.extend_from_slice(PREFIX.as_bytes());
    eth_message.extend_from_slice(len_string.as_bytes());
    eth_message.extend_from_slice(message);

    get_keccak256_hash(&eth_message).to_vec()
}

/// Computes the Keccak256 hash of a given message
///
/// # Arguments
///
/// * `eth_message` - The message to be hashed
///
/// # Returns
///
/// * `Vec<u8>` - The Keccak256 hash of the message
fn get_keccak256_hash(eth_message: &[u8]) -> Vec<u8> {
    let hash = Keccak256::digest(eth_message);
    hash.to_vec()
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct VerifierProxy {
    aurora: AccountId, // Account ID of the Aurora contract
    nft_account_id: AccountId, // Account ID of the NFT contract
    ic_remote_public_key: Address, // Remote public key address
    contract_address: Address, // Contract address
}

#[near_bindgen]
impl VerifierProxy {
    /// Initializes the VerifierProxy contract
    ///
    /// # Arguments
    ///
    /// * `aurora` - Account ID of the Aurora contract
    /// * `nft_account_id` - Account ID of the NFT contract
    /// * `ic_remote_public_key` - Remote public key as a string
    /// * `contract_address` - Contract address as a string
    ///
    /// # Returns
    ///
    /// * `Self` - An instance of VerifierProxy
    #[init]
    pub fn init(
        aurora: AccountId, // This can be safely set to "aurora"
        nft_account_id: AccountId,
        ic_remote_public_key: String,
        contract_address: String,
    ) -> Self {
        Self {
            // This value only needs to be changed if you are running the aurora testnet locally
            aurora,
            nft_account_id,
            ic_remote_public_key: aurora_sdk::parse_address(&ic_remote_public_key)
                .expect("ic_remote_public_key parse Error"),
            contract_address: aurora_sdk::parse_address(&contract_address)
                .expect("aurora_sdk parse Error"),
        }
    }

    /// Retrieves the current state of the contract
    ///
    /// # Returns
    ///
    /// * `(AccountId, Address, Address)` - Tuple containing the NFT account ID, contract address, and remote public key
    pub fn get_contract_state(&self) -> (AccountId, Address, Address) {
        return (
            self.nft_account_id.clone(),
            self.contract_address.clone(),
            self.ic_remote_public_key,
        );
    }

    /// Generates a Merkle tree from a vector of ProofResponse objects.
    /// Each ProofResponse is hashed to create the leaves of the tree.
    ///
    /// # Arguments
    ///
    /// * `proof` - A string representing the proof
    ///
    /// # Returns
    ///
    /// * `String` - The Merkle root as a hexadecimal string
    pub fn generate_merkle_tree(&self, proof: String) -> String {
        // Convert each ProofResponse into a 32-byte hash to serve as a leaf in the Merkle tree.
        let leaves: Vec<[u8; 32]> = vec![proof]
            .iter()
            .map(|proof_response| {
                let proof_byte_content = proof_response.as_bytes();
                Sha256::hash(proof_byte_content)
            })
            .collect();

        // Construct the Merkle tree from the hashed leaves.
        let tree: MerkleTree<Sha256> = MerkleTree::<Sha256>::from_leaves(&leaves);
        let merkle_root = tree.root().expect("NOT ENOUGH LEAVES");
        hex::encode(merkle_root)
    }

    /// Verifies an ECDSA signature against a given proof
    ///
    /// # Arguments
    ///
    /// * `proof` - A string representing the proof
    /// * `signature` - A string representing the signature
    ///
    /// # Returns
    ///
    /// * `bool` - True if the verification is successful, false otherwise
    pub fn ecdsa_verification(&self, proof: String, signature: String) -> bool {
        let root_hash = hash_eth_message(self.generate_merkle_tree(proof));
        let signature = string_to_vec_u8(&signature);
        if signature.len() != 65 {
            env::panic_str("INVALID_ETH_SIGNATURE");
        }
        let signature_bytes: [u8; 64] = signature[0..64].try_into().unwrap();
        let mut signature_bytes = signature_bytes.to_vec();
        signature_bytes.push(signature[64] - 27);
        let hash = H256::from_slice(&root_hash);
        let add = ecrecover(hash, &signature_bytes)
            .ok()
            .expect("successful ecrecover");
        add.encode().to_lowercase() == self.ic_remote_public_key.encode().to_lowercase()
    }

    /// Verifies a proof and mints an NFT if the verification is successful
    ///
    /// # Arguments
    ///
    /// * `proof` - A string representing the proof
    /// * `signature` - A string representing the signature
    /// * `image_url` - URL of the image associated with the NFT
    /// * `owner_address` - Account ID of the owner
    ///
    /// # Returns
    ///
    /// * `Promise` - A promise representing the NFT minting operation
    pub fn verify_proof_v2(
        &self,
        proof: String,
        signature: String,
        image_url: String,
        owner_address: AccountId,
    ) -> Promise {
        if self.ecdsa_verification(proof.clone(), signature) {
            // Find the start of the JSON
            if let (Some(start), Some(end)) = (proof.rfind("\r\n{"), proof.rfind("}}\r\n")) {
                let json_str = &proof[start + 2..=end + 1];
                env::log_str(&format!("json_str: {:?}", json_str));

                let token_metadata =
                    generate_tweet_nft_payload(json_str, image_url, owner_address.to_string());
                env::log_str(&format!("TokenMetadata: {:?}", token_metadata));

                #[derive(Deserialize)]
                struct MetadataExtra {
                    minted_to: AccountId,
                }

                // Mint the NFT here after a successful verification
                let token_id = token_metadata.title.clone().unwrap();
                let receiver_id: MetadataExtra =
                    serde_json::from_str(&token_metadata.clone().extra.unwrap_or(
                        json!({"minted_to": env::predecessor_account_id()}).to_string(),
                    ))
                    .unwrap();
                env::log_str(&format!("receiver_id: {:?}", receiver_id.minted_to));
                return nft_contract::ext(self.nft_account_id.clone())
                    .with_static_gas(Gas(5_000_000_000_000))
                    .with_attached_deposit(DEPOSIT)
                    .nft_mint(token_id, receiver_id.minted_to, token_metadata.clone())
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(5_000_000_000_000))
                            .nft_creation_callback(),
                    );
            } else {
                env::panic_str("No JSON found in input string.");
            }
        } else {
            env::panic_str("INVALID PROOF");
        }
    }

    /// Verifies a proof and provides the metadata for NFT minting
    ///
    /// # Arguments
    ///
    /// * `journal` - A vector of bytes representing the journal
    /// * `token_metadata` - Metadata of the token to be minted
    ///
    /// # Returns
    ///
    /// * `Promise` - A promise representing the NFT minting operation
    pub fn verify_proof(&self, journal: Vec<u8>, token_metadata: TokenMetadata) -> Promise {
        let journal_output = ethabi::Token::Bytes(journal.clone().into());

        // Assert that the journal is equal to the sha256 hash of the stringified token metadata
        let stringified_token_metadata = serde_json::to_string(&token_metadata).unwrap();
        let stringified_token_metadata = stringified_token_metadata.as_bytes();

        let hashed_token_metadata = env::sha256(stringified_token_metadata);
        assert_eq!(
            hex::encode(hashed_token_metadata),
            hex::encode(journal.clone()),
            "invalid token_metadata"
        );

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

    /// Sets a new contract address for the verifier
    ///
    /// # Arguments
    ///
    /// * `new_contract_address` - New contract address as a string
    #[private]
    pub fn set_verifier_address(&mut self, new_contract_address: String) {
        self.contract_address = aurora_sdk::parse_address(&new_contract_address).unwrap();
    }

    /// Sets a new NFT account
    ///
    /// # Arguments
    ///
    /// * `nft_contract` - New NFT contract as a string
    #[private]
    pub fn set_nft_account(&mut self, nft_contract: String) {
        self.nft_account_id = nft_contract.parse().unwrap();
    }

    /// Sets a new remote public key address
    ///
    /// # Arguments
    ///
    /// * `ic_public_address` - New remote public key address as a string
    #[private]
    pub fn set_ic_public_address(&mut self, ic_public_address: String) {
        self.ic_remote_public_key = aurora_sdk::parse_address(&ic_public_address).unwrap()
    }

    /// Callback used to parse the output from the call to Aurora made in `exact_output_single`.
    /// TODO: Pass in the NFT payload to this function and mint the NFT after this callback
    ///
    /// # Arguments
    ///
    /// * `token_metadata` - Metadata of the token to be minted
    /// * `result` - Result of the Aurora call
    ///
    /// # Returns
    ///
    /// * `Promise` - A promise representing the NFT minting operation
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

                // If this proof is invalid then throw an error
                if !is_valid {
                    env::panic_str(&format!("invalid Payload: {:?}", bytes));
                }

                #[derive(Deserialize)]
                struct MetadataExtra {
                    minted_to: AccountId,
                }

                // Mint the NFT here after a successful verification
                let token_id = token_metadata.title.clone().unwrap();
                let receiver_id: MetadataExtra =
                    serde_json::from_str(&token_metadata.clone().extra.unwrap_or(
                        json!({"minted_to": env::predecessor_account_id()}).to_string(),
                    ))
                    .unwrap();

                env::log_str(&format!("receiver_id: {:?}", receiver_id.minted_to));
                return nft_contract::ext(self.nft_account_id.clone())
                    .with_static_gas(Gas(5_000_000_000_000))
                    .with_attached_deposit(DEPOSIT)
                    .nft_mint(token_id, receiver_id.minted_to, token_metadata.clone())
                    .then(
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

    /// Callback function to handle the result of the NFT creation
    ///
    /// # Arguments
    ///
    /// * `call_result` - Result of the NFT creation call
    ///
    /// # Returns
    ///
    /// * `bool` - True if the NFT creation was successful, false otherwise
    #[private]
    pub fn nft_creation_callback(
        &mut self,
        #[callback_result] call_result: Result<Token, PromiseError>,
    ) -> bool {
        // Return whether or not the promise succeeded using the method outlined in external.rs
        if call_result.is_err() {
            env::panic_str(format!("nft_creation failed:{:?}", call_result.err()).as_str());
        } else {
            env::log_str(format!("nft_creation was successful!").as_str());
            return true;
        }
    }
}
