# ============================================================
# EMQX Auth Service — Multi-Stage Dockerfile (AMD64)
# ============================================================
# Description:
#   Rust-based EMQX HTTP Auth Service with multi-database support
#   (SQLite, PostgreSQL, MySQL). Uses Debian Bookworm for glibc
#   compatibility.
# ============================================================

# ------------------------------------------------------------
# Stage 1 — Build Stage
# ------------------------------------------------------------
FROM debian:bookworm-slim AS builder

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    clang \
    libclang-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable

ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY migration ./migration
COPY src ./src

RUN cargo build --release

# ------------------------------------------------------------
# Stage 2 — Runtime Stage
# ------------------------------------------------------------
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/emqx_auth_service /app/emqx_auth_service

RUN useradd -m -u 1000 service && chown -R service:service /app
USER service

EXPOSE 5500

HEALTHCHECK --interval=30s --timeout=5s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:5500/ || exit 1

CMD ["/app/emqx_auth_service"]
