#!/bin/bash
 
set -e
 
error_exit() {
    echo "$1" >&2
    exit 1
}
 
# Check if arguments are provided

if [[ $# -eq 0 ]]; then
    error_exit "No NODE_IDX arguments provided. Usage: ./run-nodes.sh 1 2 3 [log_level]"
fi
 
# Check if PINNING_INSTANCES is set
if [[ -z "$PINNING_INSTANCES" ]]; then
    error_exit "PINNING_INSTANCES environment variable is not set."
fi

# Check if the last argument is a valid log level
LOG_LEVEL="info" # Default value
if [[ "${!#}" =~ ^(info|debug|warn|error|trace)$ ]]; then
    LOG_LEVEL="${!#}"
    NODE_ARGS=("${@:1:$(($#-1))}") # All but the last argument
else
    NODE_ARGS=("$@") # All arguments
fi
 
# Loop through all provided arguments (except log level)
for NODE_IDX in "${NODE_ARGS[@]}"; do
    # Check if NODE_IDX is an integer
    if ! [[ "$NODE_IDX" =~ ^[0-9]+$ ]]; then
        error_exit "NODE_IDX '$NODE_IDX' must be an integer."
    fi
 
    # Check if NODE_IDX is within the allowed range
    if (( NODE_IDX < 1 || NODE_IDX > PINNING_INSTANCES )); then
        error_exit "NODE_IDX '$NODE_IDX' must be between 1 and $PINNING_INSTANCES."
    fi
 
    PINNING_NODE_PATH="$HOME/pinning-node/target/release/pinning_node"
 
    # Check if the Pinning Node program exists
    if [[ ! -x "$PINNING_NODE_PATH" ]]; then
        error_exit "Pinning Node program not found or not executable at $PINNING_NODE_PATH"
    fi
 
    mkdir -p "$HOME/virtual_$NODE_IDX"
    IPFS_PUBKEYS_PATH="$HOME/config/virtual-$NODE_IDX/ipfs-pubkeys.json"
    echo "=============================================="
    echo "Starting pinning node $NODE_IDX with log level $LOG_LEVEL"
    echo "=============================================="
 
    # Start the pinning node
    RUST_LOG="$LOG_LEVEL" "$PINNING_NODE_PATH" start \
        --seed "$VALIDATOR_SEED" \
        --rpc "$CHAIN_RPC" \
        --retries "$FAILURE_RETRY" \
        --ipfs-peers-config "$IPFS_PUBKEYS_PATH" \
        --rep-factor "$REPLICATION_FACTOR" \
        --keytable-log \
        --latency  > "$HOME/pinning_$NODE_IDX.log" 2>&1 &
 
    echo "PID $!" > "$HOME/pid_node_$NODE_IDX"

done
