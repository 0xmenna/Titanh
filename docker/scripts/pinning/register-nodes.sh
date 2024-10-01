#!/bin/bash


set -e

error_exit() {
    echo "Error: $1" >&2
    exit 1
}

# Check if PINNING_INSTANCES is set
if [[ -z "$PINNING_INSTANCES" ]]; then
    error_exit "PINNING_INSTANCES environment variable is not set."
fi

# Check if PINNING_INSTANCES is a positive integer
if ! [[ "$PINNING_INSTANCES" =~ ^[1-9][0-9]*$ ]]; then
    error_exit "PINNING_INSTANCES must be a positive integer."
fi

# List of required environment variables
REQUIRED_VARS=("VALIDATOR_SEED" "CHAIN_RPC" "FAILURE_RETRY")

# Check if each required environment variable is set
for var in "${REQUIRED_VARS[@]}"; do
    if [[ -z "${!var}" ]]; then
        error_exit "$var environment variable is not set."
    fi
done

# Set the path for seed
IPFS_SEEDS_PATH="$HOME/config/ipfs_seeds"

# Set the path for ipfs peers config
IPFS_PUBKEYS_PATH="$HOME/config/ipfs_pubkeys.json"

# Define paths
CLI_PATH="$HOME/cli/pinning-committee/target/release/pinning-committee"

#SEEDS_FILE="$HOME/config/ipfs_seeds_$i"
PINNING_NODE_PATH="$HOME/pinning-node/target/release/pinning_node"

# Check if the CLI program exists
if [[ ! -x "$CLI_PATH" ]]; then
    error_exit "CLI program not found or not executable at $CLI_PATH"
fi

# Check if the Pinning Node program exists
if [[ ! -x "$PINNING_NODE_PATH" ]]; then
    error_exit "Pinning Node program not found or not executable at $PINNING_NODE_PATH"
fi

# Check if the seeds file exists
if [[ ! -f "$IPFS_SEEDS_PATH" ]]; then
    error_exit "Seeds file not found at $IPFS_SEEDS_PATH"
fi

if [[ ! -f "$IPFS_PUBKEYS_PATH" ]]; then
    error_exit "IPFS pubkeys path not found at $IPFS_PUBKEYS_PATH"
fi

# Wait to ensure the chain is up
echo "Waiting for the chain to start..."
sleep 10

# Loop through each instance
for (( i=1; i<=PINNING_INSTANCES; i++ ))
do
    echo "=============================================="
    echo "Setting up virtual node $i..."
    echo "=============================================="
    
    
    # Register the pinning node
    echo "Sending registration transaction on chain for the virtual node $i..."
    "$CLI_PATH" register-pinning-node \
        --seed-phrase "$VALIDATOR_SEED" \
        --rpc "$CHAIN_RPC" \
        --seeds-file "$IPFS_SEEDS_PATH"
done
