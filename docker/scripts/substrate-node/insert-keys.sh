#!/bin/bash

set -e

# Check if environment variables are set
if [ -z "$SEED_PHRASE" ]; then
  echo "Error: SEED_PHRASE environment variable is not set."
  exit 1
fi

if [ -z "$CHAIN_SPEC" ]; then
  echo "Error: CHAIN_SPEC environment variable is not set."
  exit 1
fi


NODE_PATH="/home/titanh/target/release/titanh-node"

# Insert aura key
echo "Inserting aura key..."
$NODE_PATH key insert --base-path /tmp/node \
  --chain "$CHAIN_SPEC" \
  --scheme Sr25519 \
  --suri "$SEED_PHRASE" \
  --key-type aura

# Insert gran key
echo "Inserting grandpa key..."
$NODE_PATH key insert --base-path /tmp/node \
  --chain "$CHAIN_SPEC" \
  --scheme Ed25519 \
  --suri "$SEED_PHRASE" \
  --key-type gran

echo "Keys have been successfully inserted."
