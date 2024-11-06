export NEAR_CONTRACT_ACCOUNT=local-verifier.testnet
export NEAR_SIGNER_ACCOUNT=local-verifier.testnet
export JOURNAL_OUTPUT=[5,108,54,197,77,213,222,181,61,76,115,208,127,194,249,223,58,239,223,155,228,195,6,75,135,140,46,65,10,139,122,164]

near contract call-function as-transaction $NEAR_CONTRACT_ACCOUNT verify_proof json-args '{"journal": '$JOURNAL_OUTPUT'}' prepaid-gas '300.0 Tgas' attached-deposit '0 NEAR' sign-as $NEAR_SIGNER_ACCOUNT network-config testnet sign-with-keychain send
