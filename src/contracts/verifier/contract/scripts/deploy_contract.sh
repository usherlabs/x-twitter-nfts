export NEAR_VERIFIER_CONTRACT_ACCOUNT=local-verifier.testnet
export NEAR_NFT_CONTRACT=x-bitte-nfts.testnet
export EVM_VERIFIER_ADDRESS="0xa82219472be3fac01d0b20f043a5b03aea64fb25"
export IC_PUBLIC_KEY="0x67a50f578bd80deae3ebdd6ebf40e2aaf3b31431"



cargo build --target wasm32-unknown-unknown --release
near contract deploy $NEAR_VERIFIER_CONTRACT_ACCOUNT use-file target/wasm32-unknown-unknown/release/near_x_twitter_nfts.wasm with-init-call init json-args '{"contract_address":"'$EVM_VERIFIER_ADDRESS'", "aurora":"aurora","nft_account_id":"'$NEAR_NFT_CONTRACT'","ic_remote_public_key":"'$IC_PUBLIC_KEY'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send
