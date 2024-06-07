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
    1. First, [install Rust](https://www.rust-lang.org/tools/install) and [Foundry](https://book.getfoundry.sh/getting-started/installation), and then restart your terminal.

    ```sh
    # Install Rust
    curl https://sh.rustup.rs -sSf | sh
    # Install Foundry
    curl -L https://foundry.paradigm.xyz | bash
    ```
    
    2. Install the [necessary tool-chain to build the program](https://dev.risczero.com/api/zkvm/install)
    3. Install Docker

2. Navigate to the `zkaf` directory  
    Moved into X (Twitter) NFTs — https://github.com/usherlabs/x-twitter-nfts/blob/f6c7fb0448408eda30ba0c0402014e94a5c0b868/src/zkaf
    
3. Create environment variables  
    See all requires env variables: [.env.sample](https://github.com/usherlabs/x-twitter-nfts/blob/f6c7fb0448408eda30ba0c0402014e94a5c0b868/src/zkaf/.env.sample)
    
4. To deterministically build the ZK Circuit / Guest, Docker must be running — [Learn more](https://dev.risczero.com/terminology#deterministic-builds)

5. Run the publisher to generate and verify the proof.  

    ```shell
    cargo run --bin publisher
    ```

### Important Note

Building the guest code generates a unique identifier called the `image_id` , Which is used as a unique identifier for the program. In order to ensure your build is determinisitic i.e to ensure that you build the same guest code as initially intended, make sure the `RISC0_USE_DOCKER` flag is set to true. This would ensure the proofs generated are compatible with the verifiers deployed.


```jsx
# RISC0 Parameters
export RISC0_USE_DOCKER=true
```

### Smart Contracts

These contracts, along with the ZK circuit can be found in the `zkaf` folder. and its core components are listed below:

- `apps` : This folder contains the host code which is responsible for generating and verifying the proof onchain.
- `contracts` : This folder contains the solidity contracts which are responsible for on-chain verification.
- `methods` : This folder contains the ZK circuit which is used to generate a proof.

### Aurora

The Aurora contract which handles the proof verification on-chain would be deployed first, then the contract address obtained upon deployment will be used as a parameter to deploy the near contract. Then all the variables used would be filled in here: https://github.com/usherlabs/twitter_notary/blob/f6c7fb0448408eda30ba0c0402014e94a5c0b868/src/zkaf/.env.sh.sample in order to generate and verify the proof on-chain. 

### Testing the Aurora Contract

```jsx
cargo build
forge build
forge test
```

#### Deploying the contract

```jsx
export ETH_WALLET_PRIVATE_KEY="0x7a9dbc66cd59075f19a3d8d72e2bc04acceb7be9411c469e44dd310342a70eab" 

forge script script/Deploy.s.sol --rpc-url https://aurora-testnet.drpc.org --broadcast --legacy
```

Deploying the contract should provide the address, which would be noted for the next step which is the deployment of the near contract.

### Near Contract

The near contract is responsible for checking if a proof has been verified on the aurora chain, it does this my checking a mapping `isJournalVerified` on the aurora smart contract and throws an error if the variable returns false, indicating it has not been verified.

#### Prerequisites:

- [ ]  A near testnet account (https://testnet.mynearwallet.com/)
- [ ]  Install system dependency for rust-cli package

```bash
sudo apt install -y pkg-config libusb-1.0-0-dev libftdi1-dev
sudo apt-get install libudev-dev
```

- [ ]  Near Rust CLI (https://docs.near.org/tools/near-cli-rs)
- [ ]  Login to near on the CLI by running  `sh login.sh`
- [ ]  Get some near testnet tokens at https://near-faucet.io/

#### Deployment

- Initial deployment can be performed by running `sh deploy_contract.sh`

```bash
#deploy_contract.sh

export NEAR_CONTRACT_ACCOUNT=usherzkaf.testnet
export VERIFIER_ADDRESS="0xa4015D18436d266074eC43bb9D2f8DfBAb2a45D5"

...
...
...
### The VERIFIER_ADDRESS should be replaced with the evm address obtained in previous step
### The NEAR_CONTRACT_ACCOUNT should be replaced with the name of the logged in near account the contract was deployed to
```

- subsequently after initial deployment, further deployments are considered upgrades to the contract and can be persisted by running `sh upgrade_contract.sh`

#### Calling the contract

The respective methods on the contract can be called by running the corresponding script in the `scripts` directory.

#### Testing the Near Contract

The contract can be tested by running `cargo test` at the root of the `integration-tests` folder.

### ZK Proof

Upon deployment of the near and aurora smart contract, The details used for deployment of the near contracts and the EVM address of the aurora contract order to generate and verify a proof:

The following environment variables need to be filled, copied and pasted into the terminal in which the proof generation command would work:

```jsx
# RISC0 Parameters
export RISC0_USE_DOCKER=true

# EVM parameters
export EVM_CHAIN_ID=1313161555
export EVM_RPC_URL="https://testnet.aurora.dev"
export EVM_VERIFIER_CONTRACT=0xa4015D18436d266074eC43bb9D2f8DfBAb2a45D5 //replace

# NEAR parameters
export NEAR_RPC_URL="https://rpc.testnet.near.org"
export NEAR_ACCOUNT_ID="usherzkaf.testnet" //replace with near account
export NEAR_ACCOUNT_SECRET_KEY="" //replace with near account secret key
export NEAR_CONTRACT_ACCOUNT_ID="usherzkaf.testnet" //replace with near account contract was deployed to

# BONSAI parameters
export BONSAI_API_KEY="" # provided with your api key
export BONSAI_API_URL="https://api.bonsai.xyz/" 

# FORGE parameters
export ETH_WALLET_PRIVATE_KEY=  //EVM PK aith some aurora eth 
```

#### `env.sample`

Then proceed to run the following `cargo run --bin publisher` which would start by generating a proof using Bonsai, then verify the proof on aurora contract, it then uses the near contract to proxy a call to the aurora contract to confirm the validity of the recently generated proof which was verified on the aurora smart contract.