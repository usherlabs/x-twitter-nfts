# X (Twitter) NFTs

*by [Usher Labs](https://usherlabs.xyz)*

Welcome to the X (Twitter) NFTs repository!

X NFTs are truly 1 of 1 NFTs, each representing a single X Post (aka. Tweet).

## Unique Features

This project is unique due to its focus on cryptography, ensuring:

1. Only one NFT can ever be minted for a given Tweet.
2. The minting price is based on the Tweet's likes, retweets, replies, and other public metrics.
3. No single party can undermine the minting process or tamper with data from X.
4. Each NFT is minted only if a zkTLS proof is successfully verified on-chain.
5. Anyone can operate a zkTLS Prover of Tweets to mint the NFTs.

X NFTs are minted on the NEAR blockchain and verified on the Aurora blockchain (a NEAR EVM L2 Chain).

## Ecosystem Integration

X NFTs leverage the pluggable interfaces within the Near Ecosystem:

1. [Bitte AI](https://www.bitte.ai/) provides a chat-first interface for minting and managing NFTs.
2. [Mintbase](https://www.mintbase.xyz/) offers a marketplace for secondary sales.

## Workflow

1. Request a Tweet to be minted via your Bitte Wallet.
2. Use AI to generate NFT art or use an automatic snapshot of the Tweet.
3. Provide a Twitter handle for notifications once the zkTLS proof is verified on-chain.
4. Wait for zkProof verification and NFT minting to complete.
5. **Brag and bet with your friends on whether your 1-of-1 X NFT will represent a Tweet that will go viral!**

## Getting Started

To develop or operate the X NFTs project, you will need to:

1. Obtain the Verity CLI to run the Verity zkTLS Prover. Details can be requested from [Usher Labs via Discord](https://go.usher.so/discord).
2. Configure and run the [Bitte Plugin](./src/bitte_plugin/README.md).
3. Configure and run the [Near Indexer / Orchestrator](./src/near_indexer/README.md).

## Development Guide

**TL;DR**

To run the repo:

1. Install dependencies:
   - [Rust](https://www.rust-lang.org/tools/install) and [Foundry](https://book.getfoundry.sh/getting-started/installation), then restart your terminal.

   ```sh
   # Install Rust
   curl https://sh.rustup.rs -sSf | sh
   # Install Foundry
   curl -L https://foundry.paradigm.xyz | bash
   ```
   
   - Install the [necessary tool-chain to build the program](https://dev.risczero.com/api/zkvm/install)
   - Install and run [Docker](https://docker.com). Docker is necessary for deterministic building of the ZK Prover ELF binary and simplifies the operation of the Orchestrator / Near Indexer.

2. Navigate to the `bitte_plugin` directory.
3. Create environment variables:
   - See all required env variables: [.env.sample](./src/bitte_plugin/.env.sample).
   - **Important:** Copy the `.env.sample` file to `plugin.env` and fill in the values.

4. Run the `bitte_plugin` [see](./src/bitte_plugin/README.md).

5. Navigate to the `near_indexer` directory.

6. Create environment variables:
   - See all required env variables: [.env.sample](./src/near_indexer/.env.sample).
   - Copy the `.env.sample` file to `.env` and fill in the values.

7. Set up Near indexer [see](./src/near_indexer/README.md).

## Folder Structure

- `src/bitte_plugin`: The Bitte Plugin includes an API that allows Bitte AI to interact with the X NFTs project.
- `src/near_indexer`: The Near Indexer orchestrates the X NFTs project, including Near NFT Contract Indexing and ZK Prover processes management.
- `src/zkaf`: The zkAF utility generates the ZK Prover ELF binary, loaded into `src/near_indexer/src/generated/methods.rs`. It includes zkVM logic and code generation for the ZK Verifier Smart Contracts.
- `src/contracts`: The Near Contracts represent the X NFTs project on the Near blockchain.
  - `src/contracts/nft`: The Near NFT Contract manages mint intents, cancellations, and verifiable metadata from the ZK Prover.
  - `src/contracts/verifier`: The ZK Verifier Contract verifies ZK Proofs on the Aurora blockchain. A Near <> Aurora message prootocol is used to bridge verified proofs from the EVM environment to the NEAR environment.

## Core (Near) Contracts

Please see the [Near Contracts Guide](./src/contracts/README.md) for more information on managing the Near contracts.

## (`zkaf`) ZK Verifier Contracts

Please see the [ZK Verifier Contracts Guide](./src/zkaf/README.md) for more information on deploying the EVM ZK Verifier.
