#!/bin/bash

set -e

# Colors for beautifying output
GREEN="\033[1;32m"
RED="\033[1;31m"
YELLOW="\033[1;33m"
RESET="\033[0m"
BOLD="\033[1m"

# Error handling function
error_exit() {
    echo -e "${RED}[ERROR]${RESET} $1" >&2
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

IPFS_SEEDS_PATH="$HOME/config/virtual-$NODE_IDX/ipfs_seeds"

# Display header
echo -e "${YELLOW}==============================================================="
echo -e "${GREEN}${BOLD}Registering Pinning Node${RESET} for Virtual Node ${BOLD}#${NODE_IDX}${RESET}"
echo -e "${YELLOW}===============================================================${RESET}"

# Show registration details
echo -e "${BOLD}Validator Seed: ${RESET}${VALIDATOR_SEED}"
echo -e "${BOLD}Chain RPC: ${RESET}${CHAIN_RPC}"
echo -e "${BOLD}IPFS Seeds File: ${RESET}${IPFS_SEEDS_PATH}"
echo ""

# Simulate progress
echo -e "${GREEN}[*] Sending registration transaction on chain...${RESET}"
sleep 1

# Execute the CLI command
"$CLI_PATH" register-pinning-node \
    --seed-phrase "$VALIDATOR_SEED" \
    --rpc "$CHAIN_RPC" \
    --seeds-file "$IPFS_SEEDS_PATH"

# On success
echo -e "${GREEN}[SUCCESS]${RESET} Registration transaction for virtual node ${BOLD}#${NODE_IDX}${RESET} has been successfully sent!"
