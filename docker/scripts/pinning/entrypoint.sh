#!/bin/bash

# Sleep for 10 seconds to wait for the chain to start
echo "Waiting for the chain to start..."
sleep 10

# Execute the register nodes script
./register-nodes.sh

# Sleep for 3 seconds
sleep 3

# Run the specified nodes with info logging level
./run-nodes.sh 1 2 3 info

wait
