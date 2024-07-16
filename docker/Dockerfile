FROM docker.io/library/ubuntu:22.04

# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
		ca-certificates && \
# apt cleanup
	apt-get autoremove -y && \
	apt-get clean && \
	find /var/lib/apt/lists/ -type f -not -name lock -delete; \
  # add user and link ~/.local/share/titanh to /data
  useradd -m -u 1000 -U -s /bin/sh -d /titanh titanh && \
  mkdir -p /data /titanh/.local/share && \
  chown -R titanh:titanh /data && \
  ln -s /data /titanh/.local/share/titanh-node

USER titanh

# copy the compiled binary to the container
COPY --chown=titanh:titanh --chmod=774 titanh-node /usr/bin/titanh-node

# check if executable works in this container
RUN /usr/bin/titanh-node --version

# ws_port
EXPOSE 9930 9333 9944 30333 30334

CMD ["/usr/bin/titanh-node"]
