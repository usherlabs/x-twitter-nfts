
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
  - [Contributing](#contributing)
  - [License](#license)

## Introduction

This project is a Near indexer designed to fetch and index NFT data from the Near blockchain. It utilizes Rust and SeaORM for efficient data processing and storage.

## Prerequisites

Before you begin, make sure you have the following installed:

- Rust (version 1.81.0 or higher)
- Cargo (Rust package manager)
- Node.js (for optional frontend development)
- PostgreSQL (database system)

## Setup

1. Clone the repository:
   ```
   git clone https://github.com/usherlabs/x-twitter-nfts.git
   cd src/near-indexer
   ```

2. Install dependencies:
   ```
   cargo install sea-orm-cli
   ```

3. Copy `.env.example` to `.env` and fill in the necessary values:
   ```
   cp .env.example .env
   nano .env
   ```

## Usage

This project is designed to run as a background service. To start it:

1. Run database migrations:
   ```
   sea-orm-cli migrate up
   ```

2. Start the project:
   ```
   cargo run
   ```

## Configuration

The main configuration is done through environment variables in the `.env` file. Make sure to set the following:

- `DATABASE_URL`: Your PostgreSQL connection string
- `NEAR_RPC_URL`: The URL of your Near RPC endpoint
- `CONTRACT_ADDRESS`: The address of the NFT contract you want to index
- `TOKEN_ID`: The ID of the NFT token you want to retrieve

## Database Migrations

To apply database schema changes:

1. Create migration:
   ```
   sea-orm-cli generate migration create_nfts_table
   ```

2. Edit the generated migration file to add your schema changes

3. Apply migration:
   ```
   sea-orm-cli migrate up
   ```

## Running the Project

To run the project:

1. Ensure you've filled in the `.env` file with the required values

2. Run the following command:
   ```
   cargo run
   ```

This will start the indexer and begin fetching NFT data from the Near blockchain.

## Environment Variables

| Variable | Description |
|----------|-------------|
| DATABASE_URL | PostgreSQL connection string |
| POSTGRES_USER | Username for PostgreSQL |
| POSTGRES_DB | Database name for PostgreSQL |
| POSTGRES_PASSWORD | Password for PostgreSQL |
| TWEET_BEARER | Bearer token for Twitter API access |
| SIGNER_SK | Secret key for signing transactions |
| THIRDWEB_CLIENT_ID | Client ID for Thirdweb integration |
| NFT_CONTRACT_ID | ID of the NFT contract |
| NEAR_RPC | Near RPC endpoint URL |
| BONSAI_API_KEY | API key for Bonsai |
| BONSAI_API_URL | Bonsai API URL |
| RISC0_USE_DOCKER | Flag to use Docker (true/false) |
| EVM_VERIFIER_CONTRACT | EVM verifier smart contract address |
| NEAR_RPC_URL | Near testnet RPC URL |
| NEAR_SIGNER_ACCOUNT_ID | Signer account ID on testnet |
| NEAR_ACCOUNT_SECRET_KEY | Account secret key |
| NEAR_VERIFIER_CONTRACT_ACCOUNT_ID | Verifier contract account ID |
| NEAR_NFT_CONTRACT_ACCOUNT_ID | NFT contract account ID |
| EVM_CHAIN_ID | Ethereum chain ID |
| EVM_RPC_URL | Ethereum RPC URL |
| ETH_WALLET_PRIVATE_KEY | Ethereum wallet private key |
| RUST_LOG | Rust logging level |
| TWEET_CONSUMER_KEY | Twitter consumer key |
| TWEET_CONSUMER_SECRET | Twitter consumer secret |
| TWEET_ACCESS_TOKEN | Twitter access token |
| TWEET_TOKEN_SECRET | Twitter token secret |