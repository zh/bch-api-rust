# --- Build stage ---
FROM rust:1.83-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release && strip target/release/bch-api-rust

# --- Runtime stage ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/bch-api-rust /usr/local/bin/bch-api-rust

ENV PORT=5943
ENV API_PREFIX=/v6
ENV NETWORK=mainnet
ENV RPC_BASEURL=http://127.0.0.1:8332
ENV RPC_USERNAME=""
ENV RPC_PASSWORD=""
ENV RPC_TIMEOUT_MS=15000
ENV FULCRUM_API=""
ENV FULCRUM_TIMEOUT_MS=15000
ENV SLP_INDEXER_API=""
ENV SLP_INDEXER_TIMEOUT_MS=15000
ENV X402_ENABLED=false
ENV USE_BASIC_AUTH=false

EXPOSE 5943

CMD ["bch-api-rust"]
