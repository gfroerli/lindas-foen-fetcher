# Build
FROM rust:1-slim-bookworm AS builder
COPY . /src
RUN apt-get update \
    && apt-get install -y cmake pkg-config libssl-dev libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*
RUN cd /src && cargo build --release

# Create runtime container
# Note that we need a small init process for PID 1 that forwards signals.
# See https://github.com/Yelp/dumb-init
FROM debian:12-slim
RUN apt-get update && apt-get install -y dumb-init libssl3 sqlite3 curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/lindas-hydrodata-fetcher /usr/local/bin/
RUN addgroup --gid 2344 lindas-hydrodata-fetcher \
    && adduser --disabled-password --gecos "" --uid 2344 --gid 2344 lindas-hydrodata-fetcher \
    && chown lindas-hydrodata-fetcher:lindas-hydrodata-fetcher /usr/local/bin/lindas-hydrodata-fetcher
USER lindas-hydrodata-fetcher
WORKDIR /home/lindas-hydrodata-fetcher
ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD [ "lindas-hydrodata-fetcher", "--config", "/etc/lindas-hydrodata-fetcher.toml" ]
