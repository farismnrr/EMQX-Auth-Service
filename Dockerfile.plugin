# ============================================================
# 🧩 EMQX Auth Plugin — Multi-Stage Dockerfile
# ============================================================
# Description:
#   This Dockerfile builds and runs the Rust-based EMQX HTTP Auth Plugin
#   designed for fast authentication and ACL checks with RocksDB integration.
#
#   It uses Debian Bookworm in both stages to ensure glibc compatibility
#   The image follows OCI labeling conventions for better visibility
#   on registries like GHCR or Docker Hub.
# ============================================================

# ------------------------------------------------------------
# 🏗️ Stage 1 — Build Stage
# ------------------------------------------------------------
FROM debian:bookworm-slim AS builder

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    clang \
    libclang-dev \
    pkg-config \
    librocksdb-dev \
    && rm -rf /var/lib/apt/lists/* \
    && curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable

ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release


# ------------------------------------------------------------
# 🚀 Stage 2 — Runtime Stage
# ------------------------------------------------------------
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    librocksdb-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/emqx_auth_plugin /app/emqx_auth_plugin

RUN useradd -m -u 1000 plugin && chown -R plugin:plugin /app && \
    mkdir -p /data && chown -R plugin:plugin /data
USER plugin

EXPOSE 5500

HEALTHCHECK --interval=30s --timeout=5s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:5500/ || exit 1

CMD ["/app/emqx_auth_plugin"]
