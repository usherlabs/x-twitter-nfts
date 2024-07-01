export NEAR_CONTRACT_ACCOUNT=local-verifier.testnet

near contract call-function as-read-only $NEAR_CONTRACT_ACCOUNT get_verifier_address json-args {} network-config testnet now