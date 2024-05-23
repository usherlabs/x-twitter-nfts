export NEAR_ACCOUNT=zkaf.testnet
export VERIFIER_ADDRESS="0xa4015D18436d266074eC43bb9D2f8DfBAb2a45D5"


cargo build --target wasm32-unknown-unknown --release
near contract deploy $NEAR_ACCOUNT use-file ../target/wasm32-unknown-unknown/release/verifier.wasm with-init-call init json-args '{"contract_address":"'$VERIFIER_ADDRESS'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send