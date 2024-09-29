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

# Function to initialize and configure an IPFS node
initialize_node() {
  local NODE_DIR=$1
  local SWARM_PORT=$2
  local API_PORT=$3
  local GATEWAY_PORT=$4

  if [ ! -d "$NODE_DIR" ]; then
    echo "Initializing IPFS repository in $NODE_DIR..."
    IPFS_PATH="$NODE_DIR" ipfs init --profile=server
  else
    echo "IPFS repository already exists in $NODE_DIR."
  fi

  # Configure Swarm address as a JSON array
  echo "Configuring Swarm port to $SWARM_PORT..."
  IPFS_PATH="$NODE_DIR" ipfs config --json Addresses.Swarm "[\"/ip4/0.0.0.0/tcp/$SWARM_PORT\"]"

  # Configure API address as a string
  echo "Configuring API port to $API_PORT..."
  IPFS_PATH="$NODE_DIR" ipfs config Addresses.API "/ip4/0.0.0.0/tcp/$API_PORT"

  # Configure Gateway address as a string
  echo "Configuring Gateway port to $GATEWAY_PORT..."
  IPFS_PATH="$NODE_DIR" ipfs config Addresses.Gateway "/ip4/0.0.0.0/tcp/$GATEWAY_PORT"

  # (Optional) Disable PubSub if not needed
  # IPFS_PATH="$NODE_DIR" ipfs config --bool Pubsub.Enabled false

  # (Optional) Set a different bootstrap list to avoid overlapping peers
  # IPFS_PATH="$NODE_DIR" ipfs bootstrap rm --all
  # IPFS_PATH="$NODE_DIR" ipfs bootstrap add <bootstrap_peer>
}


# Loop to initialize and start the specified number of IPFS nodes
for ((i=1; i<=$IPFS_REPLICAS; i++)); do
  NODE_DIR="$HOME/ipfs/.ipfs-node$i"
  SWARM_PORT=$((SWARM_BASE + i - 1))
  API_PORT=$((API_BASE + i - 1))
  GATEWAY_PORT=$((GATEWAY_BASE + i - 1))
  NODE_NAME="node$i"

  initialize_node "$NODE_DIR" "$SWARM_PORT" "$API_PORT" "$GATEWAY_PORT"
done

wait
