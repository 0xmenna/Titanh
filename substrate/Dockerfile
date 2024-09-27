# Usa un'immagine base ufficiale di Rust
FROM rust:latest

# Installa le dipendenze necessarie per Substrate
RUN apt-get update && \
    apt-get install -y cmake pkg-config libssl-dev git clang curl libclang-dev make

# Installa toolchain specifica per Substrate (nightly con componenti aggiuntivi)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    git \
    clang \
    curl \
    libclang-dev \
    protobuf-compiler \
    make && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN rustup default stable && \
    rustup update && \
    rustup update nightly && \
    rustup target add wasm32-unknown-unknown --toolchain nightly && \
    rustup component add clippy rustfmt

RUN mkdir /substrate

# Copy the entire substrate directory into the container
COPY ./node /substrate/node
COPY ./pallets /substrate/pallets
COPY ./primitives /substrate/primitives
COPY ./runtime /substrate/runtime
COPY Cargo.toml /substrate
COPY rust-toolchain.toml /substrate
COPY Cargo.lock /substrate

# Imposta la working directory
WORKDIR /substrate/node

# Compila il progetto
RUN cargo build --release

# Espone le porte necessarie
EXPOSE 9944 9945 9615 30333

# Comando per eseguire il nodo Substrate con i parametri specifici
CMD ["/substrate/target/release/titanh-node", "--dev"]
