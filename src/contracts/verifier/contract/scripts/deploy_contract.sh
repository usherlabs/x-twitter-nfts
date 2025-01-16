export NEAR_VERIFIER_CONTRACT_ACCOUNT=local-verifier.testnet
export NEAR_NFT_CONTRACT=x-bitte-nfts.testnet
export EVM_VERIFIER_ADDRESS="0xa82219472Be3faC01D0b20F043a5B03AeA64FB25"


cargo build --target wasm32-unknown-unknown --release
near contract deploy $NEAR_VERIFIER_CONTRACT_ACCOUNT use-file target/wasm32-unknown-unknown/release/near_x_twitter_nfts.wasm with-init-call init json-args '{"contract_address":"'$EVM_VERIFIER_ADDRESS'", "aurora":"aurora","nft_account_id":"'$NEAR_NFT_CONTRACT'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send