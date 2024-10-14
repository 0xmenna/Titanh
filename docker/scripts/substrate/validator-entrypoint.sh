#!/bin/bash
set -e

# Insert the node keys into the keystore
$HOME/scripts/insert-keys.sh

# Run the validator node
$HOME/scripts/run-node.sh
