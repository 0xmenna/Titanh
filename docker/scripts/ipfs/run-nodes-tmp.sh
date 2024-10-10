#!/bin/bash
IPFS_CONFIG_PATH="$HOME/config/ipfs-keys.json"

# Read the number of keys from the JSON file
IPFS_REPLICAS=$(jq '.keys | length' "$IPFS_CONFIG_PATH")

# Check if the value is valid
if [[ ! "$IPFS_REPLICAS" =~ ^[0-9]+$ ]]; then
    echo "Error: Could not determine the number of IPFS replicas."
    exit 1
fi

# Loop to initialize and run the specified number of IPFS replicas
for ((i=1; i<=IPFS_REPLICAS; i++)); do
  $HOME/scripts/run-node.sh $i
done

echo "All $IPFS_REPLICAS IPFS node(s) are up and running."

tail -f /dev/null

