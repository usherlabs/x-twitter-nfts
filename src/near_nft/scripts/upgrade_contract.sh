export NEAR_VERIFIER_CONTRACT_ACCOUNT="local-nft.testnet"

cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./target/
near contract deploy $NEAR_VERIFIER_CONTRACT_ACCOUNT use-file target/non_fungible_token.wasm without-init-call network-config testnet sign-with-keychain send
