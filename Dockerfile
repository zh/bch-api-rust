# --- Builder stage ---
FROM rust:1.85-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

# --- Runtime stage ---
FROM gcr.io/distroless/static-debian12

COPY --from=builder /app/target/release/bch-api-rust /bch-api-rust

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

ENTRYPOINT ["/bch-api-rust"]
