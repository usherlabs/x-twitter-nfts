# both accounts must be the same
export NEAR_ACCOUNT=local-verifier.testnet
export NEAR_SIGNER_ACCOUNT=local-nft.testnet
export ACCESS_GRATED=true

near contract call-function as-transaction $NEAR_SIGNER_ACCOUNT grant_mint_authority json-args '{"user": "'$NEAR_ACCOUNT'","has_access":'$ACCESS_GRATED'}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as $NEAR_SIGNER_ACCOUNT network-config testnet sign-with-keychain send