#!/bin/bash

# Read the number of keys from the JSON file
IPFS_REPLICAS=$(jq '.keys | length' "$IPFS_CONFIG_PATH")
# Loop to initialize and run the specified number of IPFS replicas
for ((i=1; i<=$IPFS_REPLICAS; i++)); do
  $HOME/scripts/run-node.sh $i
done

echo "All $IPFS_REPLICAS IPFS node(s) are up and running."

# Wait for all background processes to finish
wait
