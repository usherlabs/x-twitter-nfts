# (`zkaf`) ZK Verifier (Aurora / EVM) Contracts

This project contains the ZK Verifier contracts and associated utilities for the Aurora and EVM environments. The contracts are generated using the `zkaf` RiscZero utility, which provides native support for Solidity.

## Overview

Aurora is an EVM Layer 2 (L2) solution for the NEAR blockchain. This project leverages Aurora's capabilities to enable cross-chain verification of NFT metadata derived from Tweets, using verified zkProofs of X (Twitter) data.

## Project Components

- **`apps`**: Contains utilities for generating zkProofs from the command line interface (CLI) and copying the generated ELF binary representing the zkVM guest logic to the orchestrator (Near Indexer).
- **`contracts`**: Contains the generated Solidity contracts responsible for on-chain EVM verification.
- **`methods`**: Contains the ZK logic and guest programs for the zkVM.

## Testing the Contracts

To build and test the generated Solidity assets, use the following commands:

```bash
cargo build
cargo test

forge build
forge test
```

## Verifying the Build

After building the project, verify the image ID to ensure consistent proof generation across different systems. The file `contracts/ImageID.sol` should match the following identifier after a successful build:

```solidity
library ImageID {
    bytes32 public constant VERIFY_ID = bytes32(0xa68e16e0815455b36f043963fab7bc701e57e81876d0181e7a02d5d0faac1b23);
}
```

## Deploying the Contract

To deploy the contract, use the following command:

```bash
export ETH_WALLET_PRIVATE_KEY=""
forge script script/Deploy.s.sol --rpc-url https://aurora-testnet.drpc.org --broadcast --legacy
```

## ZK Proof Generation and Verification

To generate and verify a proof, set the following environment variables in your terminal:

```bash
# EVM parameters
export EVM_CHAIN_ID=1313161555
export AURORA_RPC_URL="https://testnet.aurora.dev"
export EVM_VERIFIER_CONTRACT=0xa4015D18436d266074eC43bb9D2f8DfBAb2a45D5

# NEAR parameters
export NEAR_RPC_URL="https://rpc.testnet.near.org"
export NEAR_SIGNER_ACCOUNT_ID="local-verifier.testnet"
export NEAR_ACCOUNT_SECRET_KEY=""
export NEAR_VERIFIER_CONTRACT_ACCOUNT_ID="local-verifier.testnet"
export NFT_CONTRACT_ID="x-bitte-nfts.testnet"

# BONSAI parameters
export BONSAI_API_KEY="" # provided with your API key
export BONSAI_API_URL="https://api.bonsai.xyz/"

# FORGE parameters
export ETH_WALLET_PRIVATE_KEY=""
```

Then, run the following command to start the proof generation and verification process:

```bash
cargo run --bin publisher
```

This command generates a proof using Bonsai, verifies it on the Aurora contract, and uses the NEAR contract to proxy a call to the Aurora contract to confirm the validity of the proof.

## RiscZero

For more information on RiscZero, refer to the [RiscZero Documentation](https://docs.risczero.com/getting-started/overview) or review the included [RiscZero Guide](./R0.md) in the `zkaf` folder.