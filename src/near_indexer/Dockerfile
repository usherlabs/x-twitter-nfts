FROM rust:1.82 as builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    curl \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/x-twitter-nft


# GIT Config necessary for package install
RUN git config --global http.postBuffer 524288000
RUN git config --global core.compression 0

# forge is required by risc0-ethereum-contracts and risc0-ethereum-contracts is required by Groth16
RUN cargo install --git https://github.com/foundry-rs/foundry --rev 398ef4a --profile release --locked forge anvil
RUN forge init --no-git /test

# required for DB migration
RUN cargo install sea-orm-cli
    
# Set environment variable during runtime
ENV CARGO_TARGET_DIR=./src/near_indexer
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY . .
RUN cargo install --path=./src/near_indexer 
# RUN cd  ./src/near_indexer && cargo build --release


# CMD ["cd","./src/near_indexer","&&", "cargo", "run"]
CMD ["indexer"]
