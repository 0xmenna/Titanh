#!/bin/bash

# Build docker images
./build-network.sh
cd ..
# Run the infrastructure
docker compose up -d

