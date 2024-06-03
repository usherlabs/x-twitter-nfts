# X (Twitter) NFTs

1. Configure X (Twitter) API v2 Keys and Conversation/Tweet ID in `./src/twitter/.env`
2. Start the Notary Server - *This server runs locally for testing purposes, but will be offered by Usher Labs' decentralised data security network for production environments.*
   ```shell
    ./start_notary_server.sh
   ```
3. Generate Twitter TLS Proof
   ```shell
    ./generate_twitter_proof.sh
   ```
   
## ZKAF Near/Aurora Testing Guide

**TL;DR** 

To run the repo:
1. Install dependencies  
    First, [install Rust] and [Foundry], and then restart your terminal.

    ```sh
    # Install Rust
    curl https://sh.rustup.rs -sSf | sh
    # Install Foundry
    curl -L https://foundry.paradigm.xyz | bash
    ```

2. Navigate to the `zkaf` directory  
    Moved into X (Twitter) NFTs — https://github.com/usherlabs/x-twitter-nfts/blob/f6c7fb0448408eda30ba0c0402014e94a5c0b868/src/zkaf
    
3. Provide all environment variables  
    See all requires env variables: https://github.com/usherlabs/x-twitter-nfts/blob/f6c7fb0448408eda30ba0c0402014e94a5c0b868/src/zkaf/.env.sh.sample

4. Run the publisher to generate and verify the proof.  

    ```shell
    cargo run --bin publisher
    ```

### Near Contract

The near contract is responsible for checking if a proof has been verified on the aurora chain, it does this my checking a mapping `isJournalVerified` on the aurora smart contract and throws an error if the variable returns false, indicating it has not been verified.

#### Prerequisites:

- [ ]  A near testnet account (https://testnet.mynearwallet.com/)
- [ ]  Near Rust CLI (https://docs.near.org/tools/near-cli-rs)
- [ ]  Login to near on the CLI by running  `sh login.sh`

#### Deployment

- Initial deployment can be performed by running `sh deploy_contract.sh`
- subsequently after initial deployment, further deployments are considered upgrades to the contract and can be persisted by running `sh upgrade_contract.sh`

#### Calling the contract

The respective methods on the contract can be called by running the corresponding script in the `scripts` directory.

#### Testing the contract

The contract can be tested by running `cargo test` at the root of the `integration-tests` folder.

## Aurora Contract

These contracts, along with the ZK circuit can be found in the `zkaf` folder. and its core components are listed below:

- `apps` : This folder contains the host code which is responsible for generating and verifying the proof onchain.
- `contracts` : This folder contains the solidity contracts which are responsible for on-chain verification.
- `methods` : This folder contains the ZK circuit which is used to generate a proof.

#### Testing the contract

```jsx
cargo build
forge build
forge test
```

#### Deploying the contract

```jsx
export ETH_WALLET_PRIVATE_KEY="" 
forge script script/Deploy.s.sol --rpc-url https://aurora-testnet.drpc.org --broadcast --legacy
```

## ZK Proof

In order to generate and verify a proof:

The following environment variables need to be filled, copied and pasted into the terminal in which the proof generation command would work.:

```jsx
# EVM parameters
export EVM_CHAIN_ID=1313161555
export EVM_RPC_URL="https://testnet.aurora.dev"
export EVM_VERIFIER_CONTRACT=0xa4015D18436d266074eC43bb9D2f8DfBAb2a45D5

# NEAR parameters
export NEAR_RPC_URL="https://rpc.testnet.near.org"
export NEAR_ACCOUNT_ID="zkaf.testnet"
export NEAR_ACCOUNT_SECRET_KEY=""
export NEAR_CONTRACT_ACCOUNT_ID="zkaf.testnet"

# BONSAI parameters
export BONSAI_API_KEY="" # provided with your api key
export BONSAI_API_URL="https://api.bonsai.xyz/" 

# FORGE parameters
export ETH_WALLET_PRIVATE_KEY= 
```

### `env.sample`

Then proceed to run the following `cargo run --bin publisher` which would start by generating a proof using Bonsai, then verify the proof on aurora contract, it then uses the near contract to proxy a call to the aurora contract to confirm the validity of the recently generated proof which was verified on the aurora smart contract.