export NEAR_VERIFIER_CONTRACT_ACCOUNT=local-verifier.testnet

cargo build --target wasm32-unknown-unknown --release
near contract deploy $NEAR_VERIFIER_CONTRACT_ACCOUNT use-file target/wasm32-unknown-unknown/release/near_x_twitter_nfts.wasm without-init-call network-config testnet sign-with-keychain send
