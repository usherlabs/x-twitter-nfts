export NEAR_NFT_ACCOUNT_ID="local-nft.testnet"
# this is the account that would be able to mint nft's, it is also the account of the verifier contract
export NEAR_VERIFIER_CONTRACT_ACCOUNT="local-verifier.testnet"

near contract deploy $NEAR_NFT_ACCOUNT_ID use-file target/non_fungible_token.wasm with-init-call new_default_meta json-args '{"owner_id":"'$NEAR_VERIFIER_CONTRACT_ACCOUNT'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send