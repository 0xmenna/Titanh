# Utilizza l'immagine ufficiale di Ubuntu come base
FROM ubuntu:latest

# Aggiorna i pacchetti e installa le dipendenze necessarie
RUN apt-get update && \
    apt-get install -y curl wget ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Scarica e installa IPFS
RUN wget https://dist.ipfs.io/go-ipfs/v0.18.1/go-ipfs_v0.18.1_linux-amd64.tar.gz && \
    tar -xvzf go-ipfs_v0.18.1_linux-amd64.tar.gz && \
    mv go-ipfs/ipfs /usr/local/bin/ipfs && \
    rm -rf go-ipfs go-ipfs_v0.18.1_linux-amd64.tar.gz

# Crea la directory per i dati di IPFS
RUN mkdir -p /data/ipfs

# Imposta la directory di lavoro per IPFS
WORKDIR /data/ipfs

# Espone le porte necessarie per IPFS
EXPOSE 4001 5001 8080

# Imposta la variabile d'ambiente per il profilo server
ENV IPFS_PROFILE=server

CMD ipfs daemon --init & bash
# Comando per avviare il nodo IPFS
#CMD ["ipfs", "daemon", "--init"]


# #!/bin/bash

# # Initialize node 1 if not already initialized
# if [ ! -f /ipfs/node1/config ]; then
#   export IPFS_PATH=/ipfs/node1
#   ipfs init
#   ipfs config Addresses.API /ip4/0.0.0.0/tcp/5001
#   ipfs config Addresses.Gateway /ip4/0.0.0.0/tcp/8080
#   ipfs config Addresses.Swarm '["/ip4/0.0.0.0/tcp/4001"]'
# fi

# # Initialize node 2 if not already initialized
# if [ ! -f /ipfs/node2/config ]; then
#   export IPFS_PATH=/ipfs/node2
#   ipfs init
#   ipfs config Addresses.API /ip4/0.0.0.0/tcp/5002
#   ipfs config Addresses.Gateway /ip4/0.0.0.0/tcp/8081
#   ipfs config Addresses.Swarm '["/ip4/0.0.0.0/tcp/4002"]'
# fi

# # Start node 1 in background
# export IPFS_PATH=/ipfs/node1
# ipfs daemon &

# # Start node 2 in background
# export IPFS_PATH=/ipfs/node2
# ipfs daemon &

# # Wait for both nodes to finish
# wait -n

# version: '3'
# services:
#   ipfs:
#     build: .
#     ports:
#       - "4001:4001"  # Swarm Node 1
#       - "5001:5001"  # API Node 1
#       - "8080:8080"  # Gateway Node 1
#       - "4002:4002"  # Swarm Node 2
#       - "5002:5002"  # API Node 2
#       - "8081:8081"  # Gateway Node 2
