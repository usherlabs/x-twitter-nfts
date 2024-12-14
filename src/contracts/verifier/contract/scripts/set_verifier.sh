# both accounts must be the same
export NEAR_CONTRACT_ACCOUNT=local-verifier.testnet
export NEAR_SIGNER_ACCOUNT=local-verifier.testnet
export VERIFIER_ADDRESS="0x5d877510E20EB00202C906c004cDdBc7a0be2C33"

near contract call-function as-transaction $NEAR_CONTRACT_ACCOUNT set_verifier_address json-args '{"new_contract_address": "'$VERIFIER_ADDRESS'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as $NEAR_SIGNER_ACCOUNT network-config testnet sign-with-keychain send