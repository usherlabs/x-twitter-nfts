export NEAR_VERIFIER_CONTRACT_ACCOUNT=bitte-x-nfts.near

cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./target/
near contract deploy $NEAR_VERIFIER_CONTRACT_ACCOUNT use-file target/non_fungible_token.wasm without-init-call network-config mainnet sign-with-keychain send
