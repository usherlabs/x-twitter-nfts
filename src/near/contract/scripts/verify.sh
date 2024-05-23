export NEAR_CONTRACT_ACCOUNT=zkaf.testnet
export NEAR_SIGNER_ACCOUNT=zkaf.testnet
export JOURNAL_OUTPUT=[79,138,213,206,29,15,197,119,208,77,97,136,128,184,199,124,154,206,214,55,64,202,67,183,8,243,36,37,169,91,17,183] 

near contract call-function as-transaction $NEAR_CONTRACT_ACCOUNT verify_proof json-args '{"journal_output": '$JOURNAL_OUTPUT'}' prepaid-gas '300.0 Tgas' attached-deposit '0 NEAR' sign-as $NEAR_SIGNER_ACCOUNT network-config testnet sign-with-keychain send
