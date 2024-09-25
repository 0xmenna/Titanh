# Usa un'immagine base ufficiale di Rust
FROM rust:latest

# Installa le dipendenze necessarie per Substrate
RUN apt-get update && \
    apt-get install -y cmake pkg-config libssl-dev git clang curl libclang-dev make

# Installa toolchain specifica per Substrate (nightly con componenti aggiuntivi)
RUN rustup default stable && \
    rustup update && \
    rustup update nightly && \
    rustup target add wasm32-unknown-unknown --toolchain nightly && \
    rustup component add clippy rustfmt

# Imposta la working directory
WORKDIR /app/substrate/node

COPY ./substrate/node .
# Compila il progetto
RUN cargo build --release

# Get from the volumes the necessary data
#COPY ./customSpecRaw.json /substrate-node-template/customSpecRaw.json

# Espone le porte necessarie
EXPOSE 9944 9945 9615 30333

# Comando per eseguire il nodo Substrate con i parametri specifici
# CMD ["./target/release/node", \
#     # "--base-path", "/tmp/node01", \
#     # "--chain", "./customSpecRaw.json", \
#     "--port", "30333", \
#     "--rpc-port", "9945", \
#     # "--telemetry-url", "wss://telemetry.polkadot.io/submit/ 0", \
#     "--validator", \
#     "--rpc-methods", "Unsafe", \
#     "--name", "MyNode01", \
#     "--password-interactive"
#     ]
CMD ["./target/release/node --dev"]