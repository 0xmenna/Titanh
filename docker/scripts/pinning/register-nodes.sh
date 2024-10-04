#!/bin/bash

set -e

error_exit() {
    echo "Error: $1" >&2
    exit 1
}

# Check if PINNING_INSTANCES is set
if [[ -z "$PINNING_INSTANCES" ]]; then
    error_exit "PINNING_INSTANCES environment variable is not set."
fi

# Check if PINNING_INSTANCES is a positive integer
if ! [[ "$PINNING_INSTANCES" =~ ^[1-9][0-9]*$ ]]; then
    error_exit "PINNING_INSTANCES must be a positive integer."
fi


# Loop through each instance
for (( i=1; i<=PINNING_INSTANCES; i++ ))
do
    $HOME/scripts/register-node.sh $i
done
