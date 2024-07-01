export NEAR_VERIFIER_CONTRACT_ACCOUNT=local-verifier.testnet

near contract call-function as-read-only $NEAR_VERIFIER_CONTRACT_ACCOUNT get_nft_account_id json-args {} network-config testnet now