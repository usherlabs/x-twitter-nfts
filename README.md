# X (Twitter) NFTs

*by [Usher Labs](https://usherlabs.xyz)*

> [!WARNING]  
> **Welcome to the Boundless branch.**  
>
> This branch introduces a Proof of Concept for generating zkProofs directly on RiscZero's Boundless platform, a decentralized R0 ZK proving network.
>
> To explore the integration, begin with the `src/zkaf/apps/src/bin/publisher.rs` file, followed by `src/zkaf/apps/src/boundless.rs`. The `generate_boundless_proof` function in the latter file is central to this logic.
>
> For further details, please refer to the [`BOUNDLESS.md`](./BOUNDLESS.md) documentation.

Welcome to the X (Twitter) NFTs repository!

X NFTs are truly 1 of 1 NFTs, each reprensenting a single X Post (aka. Tweet).

What makes this project extremely unique is it's attention to cryptography, which guarantees that:

1. Only 1 NFT can ever be minted for a given Tweet. 
2. The price to mint an NFT is determined by the number of likes, retweets, and replies the Tweet has received.
3. No single party has authority to undermine the minting process, or tamper with the data sourced from X.
4. Each NFT is minted only if a zkTLS proof is successfully verified on-chain.
5. Technically, anyone can operate a zkTLS Prover of Tweets to mint the NFTs

X NFTs are minted on the NEAR blockchain, and (*for now*) are verified on the Aurora blockchain (a NEAR EVM L2 Chain).

X NFTs leverages the pluggable interfacs within the Near Ecosystem, such that:

1. [Bitte AI](https://www.bitte.ai/) offers a chat first interface to minting and managing these NFTs
2. [Mintbase](https://www.mintbase.xyz/) offers a marketplace for these NFTs whereby secondary sales are handled.

The flow of X NFTs is as follows:

1. Request a Tweet to be minted via your Bitte Wallet
2. Use AI to generate your NFT art, or use an automatic snapshot of the Tweet.
3. Provide a X (Twitter) user handle where notifications will be sent once the zkTLS proof is verified on-chain.
4. Wait for the zkProof verification and NFT minting to complete.
5. **Brag and bet with your friends on whether your 1-of-1 X NFT will represent a Tweet that will go viral!**

## Get Started

To development or operate the X NFTs project, you will need to:

1. Obtain the Verity CLI to run the Verity zkTLS Prover. Details on this can be requested from [Usher Labs via Discord](https://go.usher.so/discord).
2. Configure and run the [Bitte Plugin](./src/bitte_plugin/README.md)
3. Configure and run the [Near Indexer / Orchestrator](./src/near_indexer/README.md)

## Development Guide

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

2. Navigate to the `bitte_plugin` directory
3. Create environment variables  
   1. See all requires env variables: [.env.sample](./src/bitte_plugin/.env.sample)
   2. **Important:** Ensure you copy the `.env.sample` file to `plugin.env` and fill in the values.

4. Run the `bitte_plugin` [see](./src/bitte_plugin/README.md) 

5. Navigate to the `near_indexer` directory

6. Create environment variables  
   1. See all requires env variables: [.env.sample](./src/near_indexer/.env.sample)
   2. Here, you can copy the `.env.sample` file to `.env` and fill in the values.

7. Set up Near indexer [see](./src/near_indexer/README.md)

## Folder Structure

- `src/bitte_plugin` : The Bitte Plugin includes an API that allows Bitte AI to interact with the X NFTs project.
- `src/near_indexer` : The Near Indexer is essentially the orchestrator program for the X NFTs project, and includes Near NFT Contract Indexing, and management of ZK Prover processes.
- `src/zkaf` : The zkAF is an utility to generate the ZK Prover ELF binary, which is loaded into `src/near_indexer/src/generated/methods.rs`. This ELF binary represents zkVM guest code that is used by the RiscZero Proving system to generate zkProofs. Additionally, the zkAF includes the zkVM logic, and code generation for the ZK Verifier Smart Contracts.
- `src/contracts` : The Near Contracts that represent the X NFTs project on the Near blockchain.
  - `src/contracts/nft` : The Near NFT Contract that is used to accept mint intents, manage intent cancellations, verifiable metadata from the ZK Prover to mint the X NFTs on the Near blockchain.
  - `src/contracts/verifier` : The Aurora Contract that is used to verify the ZK Proofs on the Aurora blockchain. A Near <> Aurora bridge is *currently* used to bridge verified proofs from the *compatible* EVM environment to the NEAR environment. This is until R0 SNARK verification is finalised for the Near VM.

## Core (Near) Contracts

Please see the [Near Contracts Guide](./src/contracts/README.md) for more information on how to test the Near/Aurora contracts.

## (`zkaf`) ZK Verifier Contracts

Please see the [ZK Verifier Contracts Guide](./src/zkaf/README.md) for more information on how to deploy the ZK Verifier.
