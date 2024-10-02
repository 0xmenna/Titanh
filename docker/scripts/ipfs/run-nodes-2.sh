#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Base ports
SWARM_BASE=4001
API_BASE=5001
GATEWAY_BASE=8080

IPFS_CONFIG_PATH="$HOME/config/ipfs-keys.json"
BOOTSTRAP_CONFIG_PATH="$HOME/config/bootnodes.json"

# Function to retrieve privateKey and peerId from the JSON file
get_node_config() {
  local FILE_PATH=$1
  local NODE_INDEX=$2
  local FIELD=$3
  jq -r ".keys[$NODE_INDEX].$FIELD" "$FILE_PATH"
}

# Function to initialize and configure an IPFS node
initialize_node() {
  local NODE_DIR=$1
  local SWARM_PORT=$2
  local API_PORT=$3
  local GATEWAY_PORT=$4
  local PRIVKEY=$5
  local PEERID=$6

  # Initialize the node if not already initialized
  if [ ! -d "$NODE_DIR" ]; then
    echo "Initializing IPFS repository in $NODE_DIR..."
    IPFS_PATH="$NODE_DIR" ipfs init --profile=server
  else
    echo "IPFS repository already exists in $NODE_DIR."
  fi

  CONFIG_FILE="$NODE_DIR/config"
  
  if [ -f "$CONFIG_FILE" ]; then
    echo "Configuring private key and PeerID for $NODE_DIR..."
    jq '.Identity.PrivKey = "'"$PRIVKEY"'" | .Identity.PeerID = "'"$PEERID"'"' "$CONFIG_FILE" > "$CONFIG_FILE.tmp" && mv "$CONFIG_FILE.tmp" "$CONFIG_FILE"
  else
    echo "Error: config file not found in $NODE_DIR."
    exit 1
  fi

  # Configure the Swarm addresses
  SWARM_JSON="[\
    \"/ip4/0.0.0.0/tcp/$SWARM_PORT\",\
    \"/ip4/0.0.0.0/udp/$SWARM_PORT/quic-v1\",\
    \"/ip4/0.0.0.0/udp/$SWARM_PORT/quic-v1/webtransport\"\
]"
  IPFS_PATH="$NODE_DIR" ipfs config --json Addresses.Swarm "$SWARM_JSON"

  # Configure API and Gateway
  IPFS_PATH="$NODE_DIR" ipfs config Addresses.API "/ip4/0.0.0.0/tcp/$API_PORT"
  IPFS_PATH="$NODE_DIR" ipfs config Addresses.Gateway "/ip4/0.0.0.0/tcp/$GATEWAY_PORT"

  # Add bootnodes (optional, depending on your setup)
  add_bootnodes_from_json "$HOME/config/bootnodes.json" "$NODE_DIR"
}

# Function to start the IPFS daemon
start_daemon() {
  local NODE_DIR=$1
  local NODE_NAME=$2

  echo "Starting IPFS daemon for $NODE_NAME..."
  IPFS_PATH="$NODE_DIR" ipfs daemon > "$HOME/${NODE_NAME}_daemon.log" 2>&1 &
  echo "$NODE_NAME daemon started with PID $!"
}

# Read the number of keys from the JSON file
IPFS_REPLICAS=$(jq '.keys | length' "$IPFS_CONFIG_PATH")
# Loop to initialize and run the specified number of IPFS replicas
for ((i=1; i<=$IPFS_REPLICAS; i++)); do
  NODE_DIR="$HOME/.ipfs-node$i"
  SWARM_PORT=$((SWARM_BASE + i - 1))
  API_PORT=$((API_BASE + i - 1))
  GATEWAY_PORT=$((GATEWAY_BASE + i - 1))
  NODE_NAME="node$i"

  CONFIG_FILE=$IPFS_CONFIG_PATH
  NODE_IDX = $i - 1
  PRIVKEY=$(get_node_config "$CONFIG_FILE" $NODE_IDX "privateKey")
  PEERID=$(get_node_config "$CONFIG_FILE" $NODE_IDX "peerId")

  # Initialize and configure the IPFS node
  initialize_node "$NODE_DIR" "$SWARM_PORT" "$API_PORT" "$GATEWAY_PORT" "$PRIVKEY" "$PEERID"

  # Start the IPFS daemon
  start_daemon "$NODE_DIR" "$NODE_NAME"
done

echo "All $IPFS_REPLICAS IPFS node(s) are up and running."
