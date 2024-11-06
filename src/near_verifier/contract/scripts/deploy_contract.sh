export NEAR_VERIFIER_CONTRACT_ACCOUNT=local-verifier.testnet
export NEAR_NFT_CONTRACT=local-nft.testnet
export EVM_VERIFIER_ADDRESS="0x063205d7605292c0B43b5508dF56733E697a15dF"


cargo build --target wasm32-unknown-unknown --release
near contract deploy $NEAR_VERIFIER_CONTRACT_ACCOUNT use-file target/wasm32-unknown-unknown/release/near_x_twitter_nfts.wasm with-init-call init json-args '{"contract_address":"'$EVM_VERIFIER_ADDRESS'", "aurora":"aurora","nft_account_id":"'$NEAR_NFT_CONTRACT'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send