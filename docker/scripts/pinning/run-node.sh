#!/bin/bash


set -e


error_exit() {
    echo "$1" >&2
    exit 1
}

# Check if NODE_IDX is provided as an argument
if [[ -z "$1" ]]; then
    error_exit "NODE_IDX argument is not provided."
fi

# Check if NODE_IDX is an integer
if ! [[ "$1" =~ ^[0-9]+$ ]]; then
    error_exit "NODE_IDX must be an integer."
fi

NODE_IDX="$1"

PINNING_NODE_PATH="$HOME/pinning-node/target/release/pinning_node"



# Check if the Pinning Node program exists
if [[ ! -x "$PINNING_NODE_PATH" ]]; then
    error_exit "Pinning Node program not found or not executable at $PINNING_NODE_PATH"
fi

# Set the logging level based on the argument, default to "info"
LOG_LEVEL=${2:-info}

mkdir -p "$HOME/virtual_$NODE_IDX"
IPFS_PUBKEYS_PATH="$HOME/config/virtual-$NODE_IDX/ipfs-pubkeys.json"

echo "=============================================="
echo "Starting virtual node $NODE_IDX"
echo "=============================================="

# Start the pinning node
RUST_LOG="$LOG_LEVEL" "$PINNING_NODE_PATH" start \
    --seed "$VALIDATOR_SEED" \
    --idx "$NODE_IDX" \
    --rpc "$CHAIN_RPC" \
    --retries "$FAILURE_RETRY" \
    --ipfs-peers-config "$IPFS_PUBKEYS_PATH" > "$HOME/virtual_$NODE_IDX/pinning.log" 2>&1 &

echo "PID $!" > "$HOME/virtual_$NODE_IDX/pid"
