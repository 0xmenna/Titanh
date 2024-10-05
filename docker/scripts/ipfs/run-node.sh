#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Check if IPFS_IDX is provided as an argument
if [[ -z "$1" ]]; then
    error_exit "IPFS_IDX argument is not provided."
fi

# Check if IPFS_IDX is an integer
if ! [[ "$1" =~ ^[0-9]+$ ]]; then
    error_exit "IPFS_IDX must be an integer."
fi

IPFS_IDX="$1"

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


# Function to add bootnodes from a JSON file
add_bootnodes_from_json() {
  local BOOTNODES_JSON_FILE="$1"
  local NODE_DIR="$2"

  # Check if the JSON file exists
  if [ ! -f "$BOOTNODES_JSON_FILE" ]; then
    echo "Error: Bootnodes JSON file not found at $BOOTNODES_JSON_FILE."
    exit 1
  fi

  echo "Parsing bootnodes from $BOOTNODES_JSON_FILE..."

  # Initialize an array to hold valid multiaddresses
  local MULTIADDRS=()

  # Iterate over each bootnode entry in the JSON
  while IFS= read -r BOOTNODE; do
    # Extract hostname, port, and peerId using jq
    local HOSTNAME PORT PEERID
    HOSTNAME=$(echo "$BOOTNODE" | jq -r '.hostname')
    PORT=$(echo "$BOOTNODE" | jq -r '.port')
    PEERID=$(echo "$BOOTNODE" | jq -r '.peerId')

    # Resolve the hostname to an IP address
    local RESOLVED_IP
    RESOLVED_IP=$(getent hosts "$HOSTNAME" | awk '{ print $1 }')

    if [ -z "$RESOLVED_IP" ]; then
      echo "Warning: Could not resolve IP for hostname '$HOSTNAME'. Skipping this bootnode."
      continue
    fi

    # Construct the multiaddress
    local MULTIADDR="/ip4/$RESOLVED_IP/udp/$PORT/quic-v1/p2p/$PEERID"
    # local MULTIADDR="/ip4/$RESOLVED_IP/tcp/$PORT/p2p/$PEERID"
    echo "Resolved Bootnode: $MULTIADDR"

    # Add to the array of multiaddresses
    MULTIADDRS+=("$MULTIADDR")
  done < <(jq -c '.bootnodes[]' "$BOOTNODES_JSON_FILE")

  # Check if there are valid bootnodes to add
  if [ ${#MULTIADDRS[@]} -eq 0 ]; then
    echo "No valid bootnodes to add from the JSON file."
    return
  fi

  # Add all bootnodes to the IPFS bootstrap list in a single command
  echo "Adding bootnodes to IPFS bootstrap list..."
  IPFS_PATH="$NODE_DIR" ipfs bootstrap add "${MULTIADDRS[@]}"
  echo "Bootnodes added successfully."
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

NODE_DIR="$HOME/.ipfs-node$IPFS_IDX"
SWARM_PORT=$((SWARM_BASE + IPFS_IDX - 1))
API_PORT=$((API_BASE + IPFS_IDX - 1))
GATEWAY_PORT=$((GATEWAY_BASE + IPFS_IDX - 1))
NODE_NAME="node$IPFS_IDX"

CONFIG_FILE=$IPFS_CONFIG_PATH
NODE_IDX=$((IPFS_IDX - 1))
PRIVKEY=$(get_node_config "$CONFIG_FILE" $NODE_IDX "privateKey")
PEERID=$(get_node_config "$CONFIG_FILE" $NODE_IDX "peerId")

# Initialize and configure the IPFS node
initialize_node "$NODE_DIR" "$SWARM_PORT" "$API_PORT" "$GATEWAY_PORT" "$PRIVKEY" "$PEERID"

# Start the IPFS daemon
start_daemon "$NODE_DIR" "$NODE_NAME"

echo "IPFS Node $IPFS_IDX started successfully."
echo "-------------------------------------------"
API_PORT=$((API_BASE + i - 1))
GATEWAY_PORT=$((GATEWAY_BASE + i - 1))
NODE_IDX=$((i - 1))
echo "Node $i:"
echo "  API Endpoint: http://127.0.0.1:$API_PORT"
echo "  Gateway Endpoint: http://127.0.0.1:$GATEWAY_PORT"
echo "  Log File: $HOME/node${i}_daemon.log"
echo "  PeerId: $(get_node_config "$CONFIG_FILE" $NODE_IDX "peerId")"
echo "-------------------------------------------"

# Wait for all background processes to finish
wait
