FROM rust:1.82 as builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/indexer

RUN git config --global http.postBuffer 524288000
RUN git config --global core.compression 0

RUN cargo install --git https://github.com/foundry-rs/foundry --rev 398ef4a --profile release --locked forge anvil
RUN forge init --no-git /test
# Set environment variable during runtime
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY . .
RUN cargo install --path=./
RUN cargo build --release --manifest-path ./


CMD ["cargo", "watch", "-x", "run"]