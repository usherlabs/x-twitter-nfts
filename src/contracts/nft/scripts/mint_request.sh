# both accounts must be the same
export NEAR_CONTRACT_ACCOUNT="x-bitte-nfts.testnet"
export NEAR_SIGNER_ACCOUNT=local-verifier.testnet
export TWEET_ID=1583095539742408706
export IMAGE_URL="https://ipfs.io/ipfs/QmXLiZP95g8h71QTqzkPvQjKkc52cus4QnkYYbAmnekmRs"
export NOTIFY="xlassix"


near contract call-function as-transaction $NEAR_CONTRACT_ACCOUNT mint_tweet_request json-args '{"tweet_id": "'$TWEET_ID'","image_url":"'$IMAGE_URL'","notify":"'$NOTIFY'"}' prepaid-gas '100.0 Tgas' attached-deposit '10 NEAR' sign-as $NEAR_SIGNER_ACCOUNT network-config testnet sign-with-keychain send