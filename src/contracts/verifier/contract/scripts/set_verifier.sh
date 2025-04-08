# both accounts must be the same
export NEAR_CONTRACT_ACCOUNT=bitte-verifier.near
export NEAR_SIGNER_ACCOUNT=bitte-verifier.near
export VERIFIER_ADDRESS="0x2dfceaefbfd875c5a900de4635481e253ed7d2cb"

near contract call-function as-transaction $NEAR_CONTRACT_ACCOUNT set_verifier_address json-args '{"new_contract_address": "'$VERIFIER_ADDRESS'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as $NEAR_SIGNER_ACCOUNT network-config mainnet sign-with-keychain send