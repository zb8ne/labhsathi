# Build context is the repo root (see infra/docker/docker-compose.yml) --
# api-gateway depends on the workspace Cargo.toml/Cargo.lock and the
# path-local labhsathi-core crate, both outside this directory.
FROM rust:1-slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev librdkafka-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY services/ services/
RUN cargo build --release -p api-gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    librdkafka1 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api-gateway /usr/local/bin/api-gateway
EXPOSE 8080
CMD ["api-gateway"]
