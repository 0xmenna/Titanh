# Utilizza l'immagine ufficiale di IPFS come base

FROM ipfs/go-ipfs:latest

# Crea la directory per i dati di IPFS

RUN mkdir -p /data/ipfs

# Imposta la directory di lavoro per IPFS
WORKDIR /data/ipfs

# Espone le porte necessarie per IPFS
EXPOSE 4001 5001 8080

# Imposta la variabile d'ambiente per il profilo server
ENV IPFS_PROFILE=server

# Comando per avviare il nodo IPFS
CMD ["ipfs", "daemon", "--init"]
