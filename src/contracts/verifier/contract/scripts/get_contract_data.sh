export NEAR_CONTRACT_ACCOUNT=cktls-verifier.testnet

near contract call-function as-read-only $NEAR_CONTRACT_ACCOUNT get_contract_state json-args {} network-config testnet now