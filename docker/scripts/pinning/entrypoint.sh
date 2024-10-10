#!/bin/bash

# Sleep for 10 seconds to wait for the chain to start
echo "Waiting for the chain to start..."
sleep 10

# Execute the register nodes script
$HOME/scripts/register-nodes.sh

# Sleep for 10 seconds
sleep 10

# Run the specified nodes with info logging level
$HOME/scripts/run-nodes.sh 1 2 3 info

wait
