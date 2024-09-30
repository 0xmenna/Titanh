#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Cehck if the variable IPFS_REPLICAS is set
if [ -z "$IPFS_REPLICAS" ]; then
  echo "Error: IPFS_REPLICAS environment variable is not set."
  exit 1
fi

# Base ports
SWARM_BASE=4001
API_BASE=5001
GATEWAY_BASE=8080

# Get the node env varible associated
get_node_variable() {
  local NODE_NUM=$1
  local VAR_TYPE=$2 # PRIVKEY o PEERID
  local VAR_NAME="NODE_${NODE_NUM}_${VAR_TYPE}"

  # Ottiene il valore della variabile dinamicamente
  local VAR_VALUE="${!VAR_NAME}"
  if [ -z "$VAR_VALUE" ]; then
    echo "Error: $VAR_NAME is not set."
    exit 1
  fi
  echo "$VAR_VALUE"
}

# Init and config a node ipfs
initialize_node() {
  local NODE_DIR=$1
  local SWARM_PORT=$2
  local API_PORT=$3
  local GATEWAY_PORT=$4
  local PRIVKEY=$5
  local PEERID=$6

  if [ ! -d "$NODE_DIR" ]; then
    echo "Initializing IPFS repository in $NODE_DIR..."
    IPFS_PATH="$NODE_DIR" ipfs init --profile=server
  else
    echo "IPFS repository already exists in $NODE_DIR."
  fi

  CONFIG_FILE="$NODE_DIR/config"
  # if [ -f "$CONFIG_FILE" ]; then
  #   echo "Configuring private key and PeerID for $NODE_DIR..."
  #   jq '.Identity.PrivKey = "'"$PRIVKEY"'" | .Identity.PeerID = "'"$PEERID"'"' "$CONFIG_FILE" > "$CONFIG_FILE.tmp" && mv "$CONFIG_FILE.tmp" "$CONFIG_FILE"
  # else
  #   echo "Error: config file not found in $NODE_DIR."
  #   exit 1
  # fi

  # Configura le porte
  echo "Configuring Swarm port to $SWARM_PORT..."
  IPFS_PATH="$NODE_DIR" ipfs config --json Addresses.Swarm "[\"/ip4/0.0.0.0/tcp/$SWARM_PORT\"]"

  echo "Configuring API port to $API_PORT..."
  IPFS_PATH="$NODE_DIR" ipfs config Addresses.API "/ip4/0.0.0.0/tcp/$API_PORT"

  echo "Configuring Gateway port to $GATEWAY_PORT..."
  IPFS_PATH="$NODE_DIR" ipfs config Addresses.Gateway "/ip4/0.0.0.0/tcp/$GATEWAY_PORT"
  # add_bootnodes_from_json "$HOME/config/bootnodes.json" "$NODE_DIR"
}


# Improved function to add bootnodes from a JSON file
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
    local MULTIADDR="/ip4/$RESOLVED_IP/tcp/$PORT/p2p/$PEERID"
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


# Starts a daemon process ipfs
start_daemon() {
  local NODE_DIR=$1
  local NODE_NAME=$2

  echo "Starting IPFS daemon for $NODE_NAME..."
  IPFS_PATH="$NODE_DIR" ipfs daemon > "$HOME/${NODE_NAME}_daemon.log" 2>&1 &
  echo "$NODE_NAME daemon started with PID $!"
}

# A loop to initalize and running the specified replicas
for ((i=1; i<=$IPFS_REPLICAS; i++)); do
  NODE_DIR="$HOME/.ipfs-node$i"
  SWARM_PORT=$((SWARM_BASE + i - 1))
  API_PORT=$((API_BASE + i - 1))
  GATEWAY_PORT=$((GATEWAY_BASE + i - 1))
  NODE_NAME="node$i"

  # Retrieve dinamically priv_key and peer_id
  PRIVKEY=$(get_node_variable "$i" "PRIVKEY")
  PEERID=$(get_node_variable "$i" "PEERID")

  # Initialize and configure the IPFS node
  initialize_node "$NODE_DIR" "$SWARM_PORT" "$API_PORT" "$GATEWAY_PORT" "$PRIVKEY" "$PEERID"

  # Starts daemon
  start_daemon "$NODE_DIR" "$NODE_NAME"
done

echo "All $IPFS_REPLICAS IPFS node(s) are up and running."

echo "-------------------------------------------"
for ((i=1; i<=$IPFS_REPLICAS; i++)); do
  API_PORT=$((API_BASE + i - 1))
  GATEWAY_PORT=$((GATEWAY_BASE + i - 1))
  echo "Node $i:"
  echo "  API Endpoint: http://127.0.0.1:$API_PORT"
  echo "  Gateway Endpoint: http://127.0.0.1:$GATEWAY_PORT"
  echo "  Log File: $HOME/node${i}_daemon.log"
  echo "  PeerId: $(get_node_variable "$i" "PEERID")"
  echo "-------------------------------------------"
done

wait
