# Use an official Ubuntu base image
FROM ubuntu:latest

ENV DEBIAN_FRONTEND=noninteractive

# Install necessary dependencies for Substrate
RUN apt-get update && \
    apt-get install -y cmake pkg-config libssl-dev git clang curl libclang-dev make iputils-ping protobuf-compiler && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Create a user titanh-substrate
RUN useradd -m -s /bin/bash titanh-substrate

# Switch to the 'titanh' user
USER titanh-substrate

# Set the working directory to the user's home
WORKDIR /home/titanh-substrate

# Install Rust and the specific toolchain for Substrate (nightly with additional components)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add Cargo to the PATH persistently
ENV PATH="/home/titanh-substrate/.cargo/bin:${PATH}"

# Set the default toolchain and add necessary components
RUN rustup default stable && \
    rustup update && \
    rustup update nightly && \
    rustup target add wasm32-unknown-unknown --toolchain nightly && \
    rustup component add clippy rustfmt

# Copy the entire Substrate directory into the container
COPY --chown=titanh-substrate:titanh-substrate . /home/titanh-substrate

# Build the project
RUN cargo clean && cargo build --release

# Expose the necessary ports
EXPOSE 9944 9945 9615 30333
