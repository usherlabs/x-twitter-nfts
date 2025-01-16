export NEAR_CONTRACT_ACCOUNT="x-bitte-nfts.testnet"
export ACCOUNT_WITH_NFT="local-verifier.testnet"

near contract call-function as-read-only $NEAR_CONTRACT_ACCOUNT nft_tokens_for_owner text-args '{"account_id": "'$ACCOUNT_WITH_NFT'"}' network-config testnet now