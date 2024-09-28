# Usa un'immagine base ufficiale di Ubuntu
FROM ubuntu:latest

# Imposta le variabili di ambiente per non interagire durante l'installazione dei pacchetti
ENV DEBIAN_FRONTEND=noninteractive

# Installa le dipendenze necessarie per Substrate, il comando ping e protoc
RUN apt-get update && \
    apt-get install -y cmake pkg-config libssl-dev git clang curl libclang-dev make iputils-ping protobuf-compiler && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Installa Rust e la toolchain specifica per Substrate (nightly con componenti aggiuntivi)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Aggiunge Cargo al PATH in modo persistente
ENV PATH="/root/.cargo/bin:${PATH}"

# Imposta la toolchain di default e aggiunge i componenti necessari
RUN rustup default stable && \
    rustup update && \
    rustup update nightly && \
    rustup target add wasm32-unknown-unknown --toolchain nightly && \
    rustup component add clippy rustfmt

# Crea una directory per Substrate
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
RUN cargo clean && cargo build --release

# Espone le porte necessarie
EXPOSE 9944 9945 9615 30333
