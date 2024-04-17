#!/bin/sh
#! the notary server should have been started, this can be started by running `sh ./start_notary_server.sh`
#? use this to run the code and generate a proof `src/twitter/twitter_dm_proof.json`
cd src/twitter
cargo run --release
cd ..
