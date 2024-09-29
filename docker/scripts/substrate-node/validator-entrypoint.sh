#!/bin/bash
set -e

# Insert the node keys into the keystore
/home/titanh/scripts/insert-keys.sh

# Run the validator node
/home/titanh/scripts/run-node.sh
