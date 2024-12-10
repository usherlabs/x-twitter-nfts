# both accounts must be the same
export NEAR_CONTRACT_ACCOUNT=local-verifier.testnet
export NFT_CONTRACT="x-bitte-nft.testnet"

near contract call-function as-transaction $NEAR_CONTRACT_ACCOUNT set_nft_account json-args '{"nft_contract": "'$NFT_CONTRACT'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as $NEAR_CONTRACT_ACCOUNT network-config testnet sign-with-keychain send