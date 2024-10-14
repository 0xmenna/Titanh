# Titanh: A Datastore Framework for Web3

## Overview

Titanh is a decentralized data store framework built using Substrate and IPFS, designed for transparent, verifiable, and decentralized content management.

## Project Structure

- **`substrate/`**: Contains blockchain-related components.
  - **`titanh-node/`**: Basic node template offered by Substrate.
  - **`runtime/`**: Blockchain runtime, it integrates the datastore custom pallets.
- **`pinning-node/`**: Contains the pinning node responsible for ensuring content availability on IPFS.
- **`garbage-collector/`**: The garbage collector node.

- **`examples/`**: Contains an example use case.

## Running the Architecture

1. Navigate to the Docker network scripts directory:

   ```bash
   cd docker/scripts/network
   ./start-network.sh
   ```

2. To run the example case, you can compile and run it with Cargo.
   You can specify the --help for its usage

   ```bash
   cd example
   cargo build --release
   ./target/release/example --help
   ```

<p align="center">
  <img src="docs/media/sub.gif" width="400" style="margin-right: 20px;"> <img src="docs/media/ipfs.png" width="150" style="margin-left: 20px;">
</p>
