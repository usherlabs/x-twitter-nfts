#!/bin/bash

# Run cargo and wait for it to complete
cargo run &
PID=$!

# Start make-agent in the background
npx --yes make-agent dev -p 8007  &

# Wait for either cargo or make-agent to finish
wait $PID || wait

# Kill the cargo process if it's still running
kill $PID 2>/dev/null

exit $?
