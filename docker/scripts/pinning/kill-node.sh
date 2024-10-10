#!/bin/bash

# Function to display an error message and exit
error_exit() {
    echo "$1" >&2
    exit 1
}

# Check if at least one argument is provided
if [[ $# -eq 0 ]]; then
    error_exit "Usage: $0 <instance_numbers...>"
fi

# Loop through all the provided virtual instance numbers
for instance in "$@"; do
    PID_FILE="$HOME/pid_node_$instance"

    # Check if the PID file exists
    if [[ -f "$PID_FILE" ]]; then
        # Read the PID from the file
        PID=$(cat "$PID_FILE" | awk '{print $2}')

        # Check if the PID is a valid number
        if [[ "$PID" =~ ^[0-9]+$ ]]; then
            echo "Killing process with PID $PID from pid_node_$instance"
            kill -TERM $PID
            # Optionally, remove the PID file after killing the process
            rm -f "$PID_FILE"
        else
            echo "No valid PID found in $PID_FILE"
        fi
    else
        echo "PID file not found for pid_node_$instance"
    fi
done
