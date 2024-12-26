# Near Indexer Project

## Table of Contents

- [Near Indexer Project](#near-indexer-project)
  - [Table of Contents](#table-of-contents)
  - [Introduction](#introduction)
  - [Prerequisites](#prerequisites)
  - [Setup](#setup)
  - [Usage](#usage)
  - [Configuration](#configuration)
  - [Database Migrations](#database-migrations)
  - [Running the Project](#running-the-project)
  - [Environment Variables](#environment-variables)
    - [General](#general)
    - [PostgreSQL](#postgresql)
    - [ZK Prover](#zk-prover)
    - [Near Signer](#near-signer)
    - [Near Contract](#near-contract)
    - [ZK Verifier / Aurora EVM](#zk-verifier--aurora-evm)
    - [Twitter API](#twitter-api)
    - [Twitter/X Notification](#twitterx-notification)

## Introduction

The Near Indexer Project is designed to efficiently fetch and index NFT data from the Near blockchain. Built with Rust and SeaORM, it ensures robust data processing and storage capabilities.

In addition to indexing, it also functions as an orchestrator for the entire NFT minting process. 

Users interact with the Near blockchain through the Bittle AI plugin (`bitte_plugin`) to send mint intents. 

Once indexed, the orchestration process includes:

- Fetching the Tweet from the X API.
- Generating a zkTLS proof of the Tweet, ensuring tamper-resistance and authenticity.

The zkProof is then verified and used to authenticate the NFT mint on the Near blockchain. 

Processed transactions and mint intents are managed and stored in a local PostgreSQL database.

## Prerequisites

Ensure the following are installed before proceeding:

- **Rust**: Version 1.81.0 or higher
- **Cargo**: Rust's package manager
- **Node.js**: Required for optional frontend development
- **PostgreSQL**: Database system

## Setup

1. **Clone the Repository**:
   
   ```bash
   git clone https://github.com/usherlabs/x-twitter-nfts.git
   cd src/near-indexer
   ```

2. **Install Dependencies**:
   
   ```bash
   cargo install sea-orm-cli
   ```

3. **Configure Environment**:
   Copy the sample environment file and update it with your specific configurations:
   
   ```bash
   cp .env.sample .env
   nano .env
   ```

## Usage

The project is intended to run as a background service. Follow these steps to start it:

1. **Run Database Migrations**:
   ```bash
   sea-orm-cli migrate up
   ```

2. **Start the Project**:
   ```bash
   cargo run
   ```

## Configuration

Configuration is managed through environment variables in the `.env` file. Key variables include:

- `DATABASE_URL`: PostgreSQL connection string
- `NEAR_RPC_URL`: Near RPC endpoint URL
- `CONTRACT_ADDRESS`: NFT contract address to index
- `TOKEN_ID`: NFT token ID to retrieve

## Database Migrations

To manage database schema changes:

1. **Create a Migration**:
   ```bash
   sea-orm-cli generate migration create_nfts_table
   ```

2. **Edit the Migration File**: Add your schema changes.

3. **Apply the Migration**:
   ```bash
   sea-orm-cli migrate up
   ```

## Running the Project

To execute the project:

1. Ensure the `.env` file is correctly configured.

2. Run the following command:
   ```bash
   cargo run
   ```

This will initiate the indexer, which will begin fetching NFT data from the Near blockchain.

## Environment Variables

### General

| Variable          | Description                        |
|-------------------|------------------------------------|
| RISC0_USE_DOCKER  | Use Docker (true/false)            |
| RUST_LOG          | Rust logging level                 |

### PostgreSQL

| Variable          | Description                        |
|-------------------|------------------------------------|
| DATABASE_URL      | PostgreSQL connection string       |
| POSTGRES_USER     | PostgreSQL username                |
| POSTGRES_DB       | PostgreSQL database name           |
| POSTGRES_PASSWORD | PostgreSQL password                |

### ZK Prover

| Variable          | Description                        |
|-------------------|------------------------------------|
| BONSAI_API_KEY    | API key for RiscZero Bonsai                 |
| BONSAI_API_URL    | RiscoZero Bonsai API URL                     |
| VERITY_PROVER_URL | Verity zkTLS Prover URL                     |

### Near Signer

| Variable                  | Description                        |
|---------------------------|------------------------------------|
| NEAR_SIGNER_ACCOUNT_ID    | Signer account ID on testnet       |
| NEAR_ACCOUNT_SECRET_KEY   | Account secret key                 |

### Near Contract

| Variable                          | Description                        |
|-----------------------------------|------------------------------------|
| NEAR_VERIFIER_CONTRACT_ACCOUNT_ID | Verifier contract account ID       |
| NEAR_RPC_URL                      | Near testnet RPC URL               |
| NFT_CONTRACT_ID                   | ID of the NFT contract             |

### ZK Verifier / Aurora EVM

| Variable                  | Description                        |
|---------------------------|------------------------------------|
| EVM_CHAIN_ID              | Ethereum chain ID                  |
| AURORA_RPC_URL            | AURORA RPC URL                     |
| AURORA_VERIFIER_CONTRACT  | EVM verifier smart contract address|

### Twitter API

| Variable                  | Description                        |
|---------------------------|------------------------------------|
| TWEET_BEARER              | Bearer token for Twitter API access|

### Twitter/X Notification

| Variable                  | Description                        |
|---------------------------|------------------------------------|
| TWEET_CONSUMER_KEY        | Twitter consumer key               |
| TWEET_CONSUMER_SECRET     | Twitter consumer secret            |
| TWEET_ACCESS_TOKEN        | Twitter access token               |
| TWEET_TOKEN_SECRET        | Twitter token secret               |

