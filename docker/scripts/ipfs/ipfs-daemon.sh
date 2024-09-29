#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

if [ -z "$IPFS_REPLICAS" ]; then
  echo "Error: IPFS_REPLICAS environment variable is not set."
  exit 1
fi

# Base ports
SWARM_BASE=4001
API_BASE=5001
GATEWAY_BASE=8080

# Function to start an IPFS daemon
start_daemon() {
  local NODE_DIR=$1
  local NODE_NAME=$2

  echo "Starting IPFS daemon for $NODE_NAME..."
  IPFS_PATH="$NODE_DIR" ipfs daemon > "$HOME/${NODE_NAME}_daemon.log" 2>&1 &
  echo "$NODE_NAME daemon started with PID $!"
}

# Loop to initialize and start the specified number of IPFS nodes
for ((i=1; i<=$IPFS_REPLICAS; i++)); do
  NODE_DIR="$HOME/ipfs/.ipfs-node$i"
  SWARM_PORT=$((SWARM_BASE + i - 1))
  API_PORT=$((API_BASE + i - 1))
  GATEWAY_PORT=$((GATEWAY_BASE + i - 1))
  NODE_NAME="node$i"

  start_daemon "$NODE_DIR" "$NODE_NAME"
done

echo "All $IPFS_REPLICAS IPFS node(s) are up and running."

# Display the API and Gateway endpoints for each node
echo "-------------------------------------------"
for ((i=1; i<=$IPFS_REPLICAS; i++)); do
  API_PORT=$((API_BASE + i - 1))
  GATEWAY_PORT=$((GATEWAY_BASE + i - 1))
  echo "Node $i:"
  echo "  API Endpoint: http://127.0.0.1:$API_PORT"
  echo "  Gateway Endpoint: http://127.0.0.1:$GATEWAY_PORT"
  echo "  Log File: $HOME/node${i}_daemon.log"
  echo "-------------------------------------------"
done

wait
