# syntax=docker/dockerfile:1.7

# Multi-stage build for the BSE collaboration server.
#
# - The builder stage compiles `bse-server` in release mode against
#   the locked workspace.
# - The runtime stage is a minimal `debian:bookworm-slim` carrying
#   just the binary, ca-certificates, and a non-root user.

FROM rust:1.94-bookworm AS builder

WORKDIR /build

# Cache deps separately from sources so subsequent rebuilds skip the
# expensive crate compile when only the application code changed.
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build only the server binary in release mode.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release -p bse-server \
    && cp /build/target/release/bse-server /usr/local/bin/bse-server

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tini \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --shell /usr/sbin/nologin bse

USER bse
WORKDIR /home/bse

COPY --from=builder /usr/local/bin/bse-server /usr/local/bin/bse-server

ENV BSE_BIND_ADDR=0.0.0.0:8080 \
    BSE_SERVER_DATA_DIR=/home/bse/data \
    RUST_LOG=info

VOLUME ["/home/bse/data"]
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD wget -q --spider http://127.0.0.1:8080/health || exit 1

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/bse-server"]
