export NEAR_VERIFIER_CONTRACT_ACCOUNT=local-nft.testnet

cargo build --target wasm32-unknown-unknown --release
near contract deploy $NEAR_VERIFIER_CONTRACT_ACCOUNT use-file target/non_fungible_token.wasm without-init-call network-config testnet sign-with-keychain send
