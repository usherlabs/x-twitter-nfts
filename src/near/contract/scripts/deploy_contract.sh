export NEAR_CONTRACT_ACCOUNT=usherzkaf.testnet
export VERIFIER_ADDRESS="0xa82219472Be3faC01D0b20F043a5B03AeA64FB25"


cargo build --target wasm32-unknown-unknown --release
near contract deploy $NEAR_CONTRACT_ACCOUNT use-file ../target/wasm32-unknown-unknown/release/verifier.wasm with-init-call init json-args '{"contract_address":"'$VERIFIER_ADDRESS'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send