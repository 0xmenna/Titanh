#!/bin/bash

# Set the logging level based on the argument, default to "info"
LOG_LEVEL=${1:-info}

# Start the pinning nodes
for (( i=1; i<=PINNING_INSTANCES; i++ ))
do
    $HOME/scripts/run-node.sh $i $LOG_LEVEL
done
