#!/bin/bash

# Check if at least one argument (container ID) is provided
if [ "$#" -lt 1 ]; then
    echo "Usage: $0 <container_1>"
    exit 1
fi

# Get container ID from input argument
CONTAINER_1=$1
USER="titanh-pinning"

# Execute chown commands for CONTAINER_1 (as root user)
docker exec -u root -it $CONTAINER_1 /bin/bash -c "chown -R $USER:$USER /home/$USER/pinning-node"
docker exec -u root -it $CONTAINER_1 /bin/bash -c "chown -R $USER:$USER /home/$USER/cli"
docker exec -u root -it $CONTAINER_1 /bin/bash -c "chown -R $USER:$USER /home/$USER/api"
docker exec -u root -it $CONTAINER_1 /bin/bash -c "chown -R $USER:$USER /home/$USER/config"
docker exec -u root -it $CONTAINER_1 /bin/bash -c "chown -R $USER:$USER /home/$USER/scripts"
