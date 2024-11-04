# Aurora Contract

This contract, along with the ZK circuit can be found in the `zkaf` folder, and its core components are listed below:

- `apps` : This folder contains the host code which is responsible for generating and verifying the proof onchain.
- `contracts` : This folder contains the solidity contracts which are responsible for on-chain verification.
- `methods` : This folder contains the ZK circuit.


### Testing the contract

```jsx
// to build and test the generated solidity assets
cargo build
cargo test

forge build
forge test
```

### Verifying the build

After building the project, the image ID must be verified to ensure that the same proof is generated across multiple systems for this repository.

The file `contracts/ImageID.sol` should match the following identifier following a succesfull build.

```solidity
library ImageID {
    bytes32 public constant VERIFY_ID = bytes32(0xa68e16e0815455b36f043963fab7bc701e57e81876d0181e7a02d5d0faac1b23);
}
```

### Deploying the contract

```jsx
export ETH_WALLET_PRIVATE_KEY="" 
forge script script/Deploy.s.sol --rpc-url https://aurora-testnet.drpc.org --broadcast --legacy
```

# ZK Proof

In order to generate and verify a proof:

The following environment variables need to be filled, copied and pasted into the terminal in which the proof generation command would work.:

```jsx
# EVM parameters
export EVM_CHAIN_ID=1313161555
export EVM_RPC_URL="https://testnet.aurora.dev"
export EVM_VERIFIER_CONTRACT=0x885c2a9c3596328b0ab4dffa9a1946c928b4113f

# NEAR parameters
export NEAR_RPC_URL="https://rpc.testnet.near.org"
export NEAR_SIGNER_ACCOUNT_ID="local-verifier.testnet"
export NEAR_ACCOUNT_SECRET_KEY=""
export NEAR_VERIFIER_CONTRACT_ACCOUNT_ID="local-verifier.testnet"
export NEAR_NFT_CONTRACT_ACCOUNT_ID="local-nft.testnet"

# BONSAI parameters
export BONSAI_API_KEY="" # provided with your api key
export BONSAI_API_URL="https://api.bonsai.xyz/" 

# FORGE parameters
export ETH_WALLET_PRIVATE_KEY= 
```

`env.sample`

Then proceed to run the following `cargo run --bin publisher` which would start by generating a proof using Bonsai, then verify the proof on aurora contract, it then uses the near contract to proxy a call to the aurora contract to confirm the validity of the recently generated proof which was verified on the aurora smart contract.