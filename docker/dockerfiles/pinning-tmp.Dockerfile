# Use an official Ubuntu base image
FROM ubuntu:latest

# Set environment variables to non-interactive to prevent prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install necessary dependencies for Substrate, the ping command, and protoc
RUN apt-get update && \
    apt-get install -y cmake pkg-config libssl-dev git clang curl libclang-dev make iputils-ping protobuf-compiler && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user 'titanh-pinning'
RUN useradd -m -s /bin/bash titanh-pinning

# Switch to the 'titanh-pinning' user
USER titanh-pinning

# Set the working directory to the user's home
WORKDIR /home/titanh-pinning

# Install Rust and the specific toolchain for Substrate (nightly with additional components)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . $HOME/.cargo/env

# Add Cargo to the PATH persistently (fixed the path here)
ENV PATH="/home/titanh-pinning/.cargo/bin:${PATH}"

# Verify rustup is installed and accessible
RUN echo $PATH && rustup --version

# Set the default toolchain and add necessary components
RUN rustup default stable && \
    rustup update && \
    rustup update nightly && \
    rustup target add wasm32-unknown-unknown --toolchain nightly && \
    rustup component add clippy rustfmt
