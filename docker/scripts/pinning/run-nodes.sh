#!/bin/bash

PINNING_NODE_PATH="$HOME/pinning-node/target/release/pinning_node"

# Function to display an error message and exit
error_exit() {
    echo "$1" >&2
    exit 1
}

# Check if the Pinning Node program exists
if [[ ! -x "$PINNING_NODE_PATH" ]]; then
    error_exit "Pinning Node program not found or not executable at $PINNING_NODE_PATH"
fi

# Set the logging level based on the argument, default to "info"
LOG_LEVEL=${1:-info}

# Start the pinning nodes
for (( i=1; i<=PINNING_INSTANCES; i++ ))
do
    mkdir -p "$HOME/virtual_$i"
    IPFS_PUBKEYS_PATH="$HOME/config/virtual-$i/ipfs-pubkeys.json"

    echo "=============================================="
    echo "Starting virtual node $i with LOG_LEVEL=$LOG_LEVEL..."
    echo "=============================================="
    
    # Start the pinning node
    RUST_LOG="$LOG_LEVEL" "$PINNING_NODE_PATH" start \
        --seed "$VALIDATOR_SEED" \
        --idx "$i" \
        --rpc "$CHAIN_RPC" \
        --retries "$FAILURE_RETRY" \
        --ipfs-peers-config "$IPFS_PUBKEYS_PATH" > "$HOME/virtual_$i/pinning.log" 2>&1 &
    
    echo "PID $!" > "$HOME/virtual_$i/pid"
done
