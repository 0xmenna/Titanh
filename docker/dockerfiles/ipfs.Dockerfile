# Use the latest Ubuntu base image
FROM ubuntu:latest

# Specify build arguments for architecture detection
ARG TARGETARCH

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV KUBO_VERSION=0.29.0

# Install necessary dependencies
RUN apt-get update && \
    apt-get install -y \
    wget \
    curl \
    tar \
    ca-certificates \
    gnupg \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# Create a new user 'titanh-ipfs' with a home directory and sudo privileges
RUN useradd -m -s /bin/bash titanh-ipfs && \
    echo "titanh-ipfs ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers.d/titanh-ipfs && \
    chmod 0440 /etc/sudoers.d/titanh-ipfs

# Switch to the new user
USER titanh-ipfs
WORKDIR /home/titanh-ipfs

# Determine the architecture and set the appropriate binary URL
RUN if [ "$TARGETARCH" = "arm64" ]; then \
    ARCH="arm64"; \
    else \
    ARCH="amd64"; \
    fi && \
    echo "Detected architecture: $ARCH" && \
    wget https://dist.ipfs.tech/kubo/v${KUBO_VERSION}/kubo_v${KUBO_VERSION}_linux-${ARCH}.tar.gz && \
    tar -xvzf kubo_v${KUBO_VERSION}_linux-${ARCH}.tar.gz && \
    cd kubo && \
    sudo mv ipfs /usr/local/bin/ && \
    sudo mv * /usr/local/bin/ && \
    cd .. && \
    rm -rf kubo_v${KUBO_VERSION}_linux-${ARCH}.tar.gz kubo


# Expose the necessary IPFS ports
EXPOSE 4001 5001 8080
