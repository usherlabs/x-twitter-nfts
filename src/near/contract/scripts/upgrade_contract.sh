export NEAR_CONTRACT_ACCOUNT=usherzkaf.testnet

cargo build --target wasm32-unknown-unknown --release
near contract deploy $NEAR_CONTRACT_ACCOUNT use-file ../target/wasm32-unknown-unknown/release/verifier.wasm without-init-call network-config testnet sign-with-keychain send
