export NEAR_NFT_ACCOUNT_ID="bitte-x-nfts.near"
# this is the account that would be able to mint nft's, it is also the account of the verifier contract
export NEAR_VERIFIER_CONTRACT_ACCOUNT="bitte-verifier.near"
export NEAR_ROYALTY_MANAGER="5fd9395dbfffbbf9388724cf65a9f3d918e75b5115c36d2c9ac964a1a3f7bcd6"


rustup target add wasm32-unknown-unknown
cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./target/
near contract deploy $NEAR_NFT_ACCOUNT_ID use-file target/non_fungible_token.wasm with-init-call new_default_meta json-args '{"owner_id":"'$NEAR_VERIFIER_CONTRACT_ACCOUNT'","royalty_manager":"'$NEAR_VERIFIER_CONTRACT_ACCOUNT'"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config mainnet sign-with-keychain send