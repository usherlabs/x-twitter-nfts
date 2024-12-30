# Core (Near) Contracts

The Core Contracts in the X NFTs project are deployed on the NEAR blockchain. These contracts facilitate the creation and management of NFTs based on verified X (Twitter) data.

## Key Responsibilities

1. **Minting Intents**: Accept and manage NFT mint intents from users.
2. **Metadata Verification**: Cross-reference NFT metadata derived from Tweets with verified zkProofs of X (Twitter) data.
3. **NFT Minting**: Mint NFTs on the NEAR blockchain using verified metadata.

## Prerequisites

- **NEAR Testnet Account**: Create an account at [NEAR Testnet Wallet](https://testnet.mynearwallet.com/).
- **System Dependencies**: Install necessary packages for Rust CLI.

  ```bash
  sudo apt install -y pkg-config libusb-1.0-0-dev libftdi1-dev
  sudo apt-get install libudev-dev
  ```

- **NEAR Rust CLI**: Follow the [NEAR CLI Documentation](https://docs.near.org/tools/near-cli-rs) to install.
- **Login**: Authenticate with NEAR using `sh login.sh`.
- **Testnet Tokens**: Obtain tokens from [NEAR Faucet](https://near-faucet.io/).

## Deployment

- **Initial Deployment**: Execute `sh deploy_contract.sh` to deploy the contracts.

  ```bash
  # deploy_contract.sh

  export NEAR_CONTRACT_ACCOUNT=usherzkaf.testnet
  export VERIFIER_ADDRESS="0xa4015D18436d266074eC43bb9D2f8DfBAb2a45D5"

  ...
  # Replace VERIFIER_ADDRESS with the EVM address obtained in the previous step
  # Replace NEAR_CONTRACT_ACCOUNT with the logged-in NEAR account name
  ```

- **Upgrades**: For contract upgrades, run `sh upgrade_contract.sh`.

## Contract Interaction

Invoke contract methods using scripts located in the `scripts` directory.

## Testing

Run `cargo test` in the `integration-tests` folder to test the contract.

> **Warning**
> Check the `integration-tests` folder to identify any breaking changes or the presence of a `WARNING.md` file.

## NEAR Smart Contract Callback Functionality for NFTs

NEAR smart contracts support cross-contract calls, allowing one contract to interact with others by querying information and executing functions. This feature is particularly useful in creating modular smart contract architectures, enabling the reuse of business functionalities across different contracts. Here's a breakdown of the key aspects and steps involved in implementing callback functionality for cross-contract calls on NEAR:

**Understanding Cross-Contract Calls**

- **Independence**: Cross-contract calls in NEAR are independent, meaning you need separate functions for initiating the call and receiving the result.
- **Asynchronicity**: These calls are asynchronous, introducing a delay between the call and the callback execution, typically spanning 1 or 2 blocks. During this period, the contract remains active and can receive other calls.

**Implementing Cross-Contract Calls**

1. **Define the Interface for the Called Contract**: Use the `ext_contract` attribute to define the interface of the contract you wish to call. This involves specifying the function signatures that match those in the target contract. For example, if you're calling an `nft_mint` function, you'd define an interface like this:

```rust
// Validator interface, for cross-contract calls
#[ext_contract(nft_contract)]
pub trait NFTContract {
    fn nft_mint(
        token_id: TokenId,
        receiver_id: AccountId,
        token_metadata: TokenMetadata,
    );
}
```

2. **Define the Local Callback Interface:** After making the cross-contract call, you'll need a local callback function to handle the result.

```rust
#[private]
pub fn nft_creation_callback(
    &mut self,
    #[callback_result] call_result: Result<Token, PromiseError>,
) -> bool {
    // Return whether or not the promise succeeded using the method outlined in external.rs
    if call_result.is_err() {
        env::panic_str(format!("nft_creation failed: {:?}", call_result.err()).as_str());
    } else {
        env::log_str("nft_creation was successful!".as_str());
        return true;
    }
}
```

3. **Initiate the Cross-Contract Call:** Within your main contract, initiate the cross-contract call using the Promise object. Specify the contract to call, the function to execute, and attach any necessary gas.

```rust
pub fn verify_proof(&self, journal: Vec<u8>, token_metadata: TokenMetadata) -> Promise {
    return nft_contract::ext(self.nft_account_id.clone())
        .with_static_gas(Gas(5_000_000_000_000))
        .with_attached_deposit(DEPOSIT)
        .nft_mint(
            token_id,
            receiver_id.clone(),
            token_metadata.clone(),
        )
        .then(
            // Create a callback
            Self::ext(env::current_account_id())
            .with_static_gas(Gas(5_000_000_000_000))
            .nft_creation_callback(),
        );
}
```

In this case, we attach the callback we had previously implemented and have it handle the response appropriately if it was a success or a failure, depending on the execution of the initial function.

### NFT Metadata Handling Research

```rust
pub struct TokenMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub media: Option<String>,
    pub media_hash: Option<Base64VecU8>,
    pub copies: Option<u64>,
    pub issued_at: Option<String>,
    pub expires_at: Option<String>,
    pub starts_at: Option<String>,
    pub updated_at: Option<String>,
    pub extra: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<Base64VecU8>,
}
```

The metadata is stored on-chain, and it is populated with the following details from the tweet:

```rust
TokenMetadata.title = tweet.id;
TokenMetadata.description = tweet.text;
TokenMetadata.extra = tweet.public_metrics // {"retweet_count": 2, "reply_count": 1, "like_count": 7, "quote_count": 0, "bookmark_count": 0, "impression_count": 357}
```

#### Technical Metadata Key Points

- The output of the Zero-Knowledge (ZK) proof generation is a hash, specifically a SHA-256 hash of the serialized Token Metadata representation.
- Subsequently, a verification step is executed on the NEAR blockchain to ensure the equality between `sha256(token_metadata)` and `zk_journal_output`. This validation process confirms the authenticity of the payload by matching it against the immutable output of the ZK process, which inherently carries a singular, unchangeable identity.
- To ensure token uniqueness, the `token_id` of the Non-Fungible Token (NFT) is determined by the `TokenMetadata.title`, which corresponds to the tweet's identifier. This approach guarantees that each NFT is exclusively linked to a single tweet, thus eliminating duplication.
- The `TokenMetadata.extra` field encompasses tweet-specific metrics, which can be utilised in the derivation of pricing algorithms for the NFT.