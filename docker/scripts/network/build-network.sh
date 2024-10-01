#!/bin/bash

cd ../../dockerfiles/

# Build the substrate image
docker build -f ./substrate.Dockerfile -t titanh-substrate ../../substrate/.

# Build the ipfs image
docker build -f ./ipfs.Dockerfile -t titanh-ipfs .

docker build -f ./pinning-tmp.Dockerfile -t titanh-pinning ../../.
