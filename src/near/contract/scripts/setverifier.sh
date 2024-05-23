export NEAR_CONTRACT_ACCOUNT=zkaf.testnet
export NEAR_SIGNER_ACCOUNT=zkaf.testnet
export VERIFIER_ADDRESS="0xa4015D18436d266074eC43bb9D2f8DfBAb2a45D5"

near contract call-function as-transaction $NEAR_ACCOUNT set_verifier_address json-args '{"new_contract_address": "'$VERIFIER_ADDRESS'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as $NEAR_SIGNER_ACCOUNT network-config testnet sign-with-keychain send