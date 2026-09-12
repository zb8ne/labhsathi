# Build context is the repo root (see infra/docker/docker-compose.yml) --
# catalog-service depends on the workspace Cargo.toml/Cargo.lock and the
# path-local labhsathi-core crate, both outside this directory.
FROM rust:1-slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY services/ services/
RUN cargo build --release -p catalog-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# GitHub issue #16 (product review): see api-gateway/Dockerfile's comment --
# same fix, same reasoning.
RUN useradd --system --no-create-home --uid 10001 labhsathi
COPY --from=builder /app/target/release/catalog-service /usr/local/bin/catalog-service
USER labhsathi
EXPOSE 8090
CMD ["catalog-service"]
