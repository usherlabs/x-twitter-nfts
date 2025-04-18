# both accounts must be the same
export NEAR_CONTRACT_ACCOUNT=local-verifier.testnet
export IC_PUBLIC_KEY="0xba12a284e29aeaaea28ff233118841950f353990"

near contract call-function as-transaction $NEAR_CONTRACT_ACCOUNT set_ic_public_address json-args '{"ic_public_address": "'$IC_PUBLIC_KEY'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as $NEAR_CONTRACT_ACCOUNT network-config testnet sign-with-keychain send