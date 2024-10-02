#!/bin/bash

PINNING_NODE_PATH="$HOME/pinning-node/target/release/pinning_node"


# Check if the Pinning Node program exists
if [[ ! -x "$PINNING_NODE_PATH" ]]; then
    error_exit "Pinning Node program not found or not executable at $PINNING_NODE_PATH"
fi

PINNING_INSTANCES=1 # Just for testing (only one node)

# Start the pinning nodes
for (( i=1; i<=PINNING_INSTANCES; i++ ))
do
    IPFS_PUBKEYS_PATH="$HOME/config/virtual-$i/ipfs-pubkeys.json"

    echo "=============================================="
    echo "Starting virtual node $i..."
    echo "=============================================="
    
    # Start the pinning node
    "$PINNING_NODE_PATH" start \
        --seed "$VALIDATOR_SEED" \
        --rpc "$CHAIN_RPC" \
        --retries "$FAILURE_RETRY" \
        --ipfs-peers-config "$IPFS_PUBKEYS_PATH"
done
