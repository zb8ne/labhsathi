# Build context is the repo root (see infra/docker/docker-compose.yml) --
# ocr-worker depends on the workspace Cargo.toml/Cargo.lock and the
# path-local labhsathi-core crate, both outside this directory.
FROM rust:1-slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev librdkafka-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY services/ services/
RUN cargo build --release -p ocr-worker

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    librdkafka1 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# GitHub issue #16 (product review): see api-gateway/Dockerfile's comment --
# same fix, same reasoning. This is the service that calls out to the
# vision API with real user documents; running it as non-root is worth
# doing here if anywhere.
RUN useradd --system --no-create-home --uid 10001 labhsathi
COPY --from=builder /app/target/release/ocr-worker /usr/local/bin/ocr-worker
USER labhsathi
CMD ["ocr-worker"]
