# both accounts must be the same
export NEAR_CONTRACT_ACCOUNT=bitte-x-nfts.near
export VERIFIER_ADDRESS=cktls-verifier.near

near contract call-function as-transaction $NEAR_CONTRACT_ACCOUNT change_owner json-args '{"owner_id": "'$VERIFIER_ADDRESS'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as $NEAR_CONTRACT_ACCOUNT network-config mainnet sign-with-keychain send