#!/bin/bash

# Check if a number is passed as an argument
if [ -z "$1" ]; then
  echo "Error: no number specified. Usage: ./script.sh <number>"
  exit 1
fi

# Set the IPFS_PATH based on the input number
export IPFS_PATH="/home/titanh-ipfs/.ipfs-node$1"

