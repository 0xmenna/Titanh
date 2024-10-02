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


# Define paths
CLI_PATH="$HOME/cli/pinning-committee/target/release/pinning-committee"

# Check if the CLI program exists
if [[ ! -x "$CLI_PATH" ]]; then
    error_exit "CLI program not found or not executable at $CLI_PATH"
fi


# Wait to ensure the chain is up
echo "Waiting for the chain to start..."
sleep 10

# Loop through each instance
for (( i=1; i<=PINNING_INSTANCES; i++ ))
do
    IPFS_SEEDS_PATH="$HOME/config/virtual-$i/ipfs_seeds"

    echo "=============================================="
    echo "Sending registration transaction on chain for virtual node $i..."
    echo "=============================================="
    
    "$CLI_PATH" register-pinning-node \
        --seed-phrase "$VALIDATOR_SEED" \
        --rpc "$CHAIN_RPC" \
        --seeds-file "$IPFS_SEEDS_PATH"
done
