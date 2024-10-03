#!/bin/bash

set -e

error_exit() {
    echo "Error: $1" >&2
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

# List of required environment variables
REQUIRED_VARS=("VALIDATOR_SEED" "CHAIN_RPC" "IPFS_RPC" "REPLICATION_FACTOR")

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

IPFS_SEEDS_PATH="$HOME/config/virtual-$NODE_IDX/ipfs_seeds"

echo "=============================================="
echo "Uploading node keytable to IPFS and sending the leave tx on chain..."
echo "=============================================="

"$CLI_PATH" leave-pinning-committee \
    --seed-phrase "$VALIDATOR_SEED" \
    --seeds-file "$IPFS_SEEDS_PATH" \
    --idx "$NODE_IDX" \
    --chain-rpc "$CHAIN_RPC" \
    --ipfs-rpc "$IPFS_RPC" \
    --table-rows "$REPLICATION_FACTOR"
