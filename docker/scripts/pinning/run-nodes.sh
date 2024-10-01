#!/bin/bash


# Register the nodes on chain: uncomment this line if you want to register the nodes
# $HOME/scripts/register-nodes.sh

# Remove this stuff after testing
PINNING_NODE_PATH="$HOME/pinning-node/target/release/pinning_node"
IPFS_PUBKEYS_PATH="$HOME/config/ipfs_pubkeys.json"

PINNING_INSTANCES=1 # Just for testing (only one node)

# Start the pinning nodes
for (( i=1; i<=PINNING_INSTANCES; i++ ))
do
    echo "=============================================="
    echo "Starting virtual node $i..."
    echo "=============================================="
    
    idx=$((i - 1))

    # Start the pinning node
    "$PINNING_NODE_PATH" start \
        --seed "$VALIDATOR_SEED" \
        --idx "$idx" \
        --rpc "$CHAIN_RPC" \
        --retries "$FAILURE_RETRY" \
        --ipfs-peers-config "$IPFS_PUBKEYS_PATH"
done
