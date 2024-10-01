#!/bin/bash

# Run cargo and wait for it to complete
cargo run &
PID=$!

wait $PID

# Start make-agent in the background
npx --yes make-agent dev -p 8007 &

# Kill the cargo process if it's still running
kill $PID 2>/dev/null

# Wait for the Rocket server to exit gracefully
wait $PID

exit $?
