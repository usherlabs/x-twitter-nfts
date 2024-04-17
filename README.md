# X (Twitter) NFTs

1. Configure X (Twitter) API v2 Keys and Conversation/Tweet ID in `./src/twitter/.env`
2. Start the Notary Server - *This sever runs locally, but will be offered by Usher Labs' decentralised data security network for production environments.*
   ```shell
    ./start_notary_server.sh
   ```
3. Generate Twitter TLS Proof
   ```shell
    ./generate_twitter_proof.sh
   ```