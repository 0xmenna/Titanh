#!/bin/bash

set -e

# Check for required environment variables
if [ -z "$CHAIN_SPEC" ]; then
  echo "Error: CHAIN_SPEC environment variable is not set."
  exit 1
fi

if [ -z "$NODE_NAME" ]; then
  echo "Error: NODE_NAME environment variable is not set."
  exit 1
fi

NODE_PATH="/home/titanh/target/release/titanh-node"

# Function to fetch BOOTNODE_ID using curl
fetch_bootnode_id() {
  local host="$1"
  
  echo "Fetching BOOTNODE_ID from host: $host"

  # Perform the curl request and store the response
  response=$(curl -s -H "Content-Type: application/json" -d \
    '{"jsonrpc":"2.0","method":"system_localPeerId","params":[],"id":1}' \
    "http://$host:9944")

  # Check if curl command was successful
  if [ $? -ne 0 ] || [ -z "$response" ]; then
    echo "Error: Failed to fetch BOOTNODE_ID from $host"
    exit 1
  fi

  # Parse the JSON response to extract the BOOTNODE_ID
  BOOTNODE_ID=$(echo "$response" | awk -F'"result":"' '{print $2}' | awk -F'"' '{print $1}')

  # Validate that BOOTNODE_ID was extracted
  if [ -z "$BOOTNODE_ID" ] || [ "$BOOTNODE_ID" == "null" ]; then
    echo "Error: BOOTNODE_ID not found in the response."
    echo "Response: $response"
    exit 1
  fi

  echo "Successfully fetched BOOTNODE_ID: $BOOTNODE_ID"
}

# Initialize the command array
CMD=("$NODE_PATH"
  --base-path "/tmp/node"
  --chain "$CHAIN_SPEC"
  --port 30333
  --rpc-port 9944
  --rpc-cors=all
  --rpc-external
  --validator
  --rpc-methods "Unsafe"
  --name "$NODE_NAME")

# If BOOTNODE_HOST is set, fetch BOOTNODE_ID and add --bootnodes
if [ -n "$BOOTNODE_HOST" ]; then
  sleep 5
  fetch_bootnode_id "$BOOTNODE_HOST"
  
  echo "Resolving BOOTNODE_HOST: $BOOTNODE_HOST"
  bootnode_ip=$(getent hosts "$BOOTNODE_HOST" | awk '{ print $1 }')
  if [ -z "$bootnode_ip" ]; then
    echo "Error: Could not resolve IP address for BOOTNODE_HOST: $BOOTNODE_HOST"
    exit 1
  fi
  echo "Resolved bootnode IP: $bootnode_ip"
  
  CMD+=("--bootnodes" "/ip4/$bootnode_ip/tcp/30333/p2p/$BOOTNODE_ID")
else
  echo "BOOTNODE_HOST not set. Running without bootnodes."
fi

# Run the node
"${CMD[@]}"
