# bch-api-rust

REST API gateway for Bitcoin Cash. Written in Rust with Axum. Drop-in replacement for the JavaScript [bch-api](https://github.com/Permissionless-Software-Foundation/bch-api) — same endpoints, same JSON shape. Aggregates a BCH full node, Fulcrum indexer, and SLP token indexer behind a single HTTP interface.

## What it does

- Proxies Bitcoin Cash full node RPC (blockchain, rawtransactions, mining, control, dsproof)
- Proxies Fulcrum/ElectrumX REST API (balance, UTXOs, transactions, broadcast)
- Proxies SLP token indexer (address, txid, token queries)
- Public-key recovery from on-chain transaction history
- BCH/USD price feed via CoinEx
- PSF file-pinning price feed via PSFFPP proxy
- x402 micropayment gating and/or Bearer token authentication
- Bulk POST routes for all query endpoints (up to 24 items)
- Automatic retry on 5xx and timeout errors (one retry)

## Quick Start

```bash
# Run locally (connects to full node at 127.0.0.1:8332)
cargo run

# Or with Docker
docker compose up

# Health check
curl http://localhost:5943/health

# API status
curl http://localhost:5943/v6/
```

## Configuration

All settings via environment variables (`.env` supported via dotenvy):

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `PORT` | u16 | `5943` | HTTP listen port |
| `API_PREFIX` | string | `/v6` | API route prefix |
| `NETWORK` | string | `mainnet` | Network: `mainnet`, `testnet3`, `regtest` |
| `RPC_BASEURL` | string | `http://127.0.0.1:8332` | BCH full node RPC URL |
| `RPC_USERNAME` | string | `""` | Full node RPC username |
| `RPC_PASSWORD` | string | `""` | Full node RPC password |
| `RPC_TIMEOUT_MS` | u64 | `15000` | Full node RPC timeout (ms) |
| `FULCRUM_API` | string | `""` | Fulcrum REST API base URL |
| `FULCRUM_TIMEOUT_MS` | u64 | `15000` | Fulcrum request timeout (ms) |
| `SLP_INDEXER_API` | string | `""` | SLP token indexer base URL |
| `SLP_INDEXER_TIMEOUT_MS` | u64 | `15000` | SLP indexer request timeout (ms) |
| `X402_ENABLED` | bool | `true` | Enable x402 micropayment auth |
| `SERVER_BCH_ADDRESS` | string | `bitcoincash:qqlr...` | BCH address for x402 payments |
| `FACILITATOR_URL` | string | `http://localhost:4345/facilitator` | x402 payment verifier URL |
| `X402_PRICE_SAT` | u64 | `200` | Price per request in satoshis |
| `USE_BASIC_AUTH` | bool | `false` | Enable Bearer token auth |
| `BASIC_AUTH_TOKEN` | string | `""` | Expected Bearer token value |
| `COINEX_API_URL` | string | `https://api.coinex.com/v1/market/ticker?market=bchusdt` | CoinEx price endpoint |
| `PSFFPP_PROXY_URL` | string | `""` | PSFFPP proxy base URL |

Boolean vars accept `true/1/yes/on` and `false/0/no/off` (case-insensitive).

## API Endpoints

Base path: `{API_PREFIX}` (default `/v6`)

### Health & Status

```
GET /health          → {"status": "ok", "service": "bch-api-rust", "version": "0.1.0"}
GET /v6/             → {"status": "bch-api-rust"}
```

### Blockchain — `/v6/full-node/blockchain`

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Status |
| GET | `/getBestBlockHash` | Best block hash |
| GET | `/getBlockchainInfo` | Blockchain info |
| GET | `/getBlockCount` | Block count |
| GET | `/getBlockHeader/{hash}?verbose=true` | Block header by hash |
| POST | `/getBlockHeader` | Bulk block headers (`{"hashes": [...], "verbose": true}`) |
| GET | `/getBlockHash/{height}` | Block hash by height |
| POST | `/getBlock` | Block by hash (`{"blockhash": "...", "verbosity": 1}`) |
| GET | `/getChainTips` | Chain tips |
| GET | `/getDifficulty` | Current difficulty |
| GET | `/getMempoolEntry/{txid}` | Mempool entry |
| POST | `/getMempoolEntry` | Bulk mempool entries (`{"txids": [...]}`) |
| GET | `/getMempoolAncestors/{txid}?verbose=true` | Mempool ancestors |
| GET | `/getMempoolInfo` | Mempool info |
| GET | `/getRawMempool?verbose=true` | Raw mempool |
| GET | `/getTxOut/{txid}/{n}?includeMempool=true` | Transaction output |
| POST | `/getTxOut` | Bulk tx outputs (`{"txid": "...", "vout": 0, "mempool": true}`) |
| GET | `/getTxOutProof/{txid}` | Tx output proof |
| POST | `/getTxOutProof` | Bulk tx output proofs (`{"txids": [...]}`) |
| GET | `/verifyTxOutProof/{proof}` | Verify tx output proof |
| POST | `/verifyTxOutProof` | Bulk verify proofs (`{"proofs": [...]}`) |

### Raw Transactions — `/v6/full-node/rawtransactions`

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Status |
| GET | `/decodeRawTransaction/{hex}` | Decode raw transaction |
| POST | `/decodeRawTransaction` | Bulk decode (`{"hexes": [...]}`) |
| GET | `/decodeScript/{hex}` | Decode script |
| POST | `/decodeScript` | Bulk decode scripts (`{"hexes": [...]}`) |
| GET | `/getRawTransaction/{txid}?verbose=true` | Raw transaction (verbose adds block height) |
| POST | `/getRawTransaction` | Bulk raw transactions (`{"txids": [...], "verbose": true}`) |
| GET | `/sendRawTransaction/{hex}` | Broadcast transaction |
| POST | `/sendRawTransaction` | Bulk broadcast — serial execution (`{"hexes": [...]}`) |

### Mining — `/v6/full-node/mining`

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Status |
| GET | `/getMiningInfo` | Mining info |
| GET | `/getNetworkHashPS?nblocks=120&height=-1` | Network hash rate |

### Control — `/v6/full-node/control`

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Status |
| GET | `/getNetworkInfo` | Network info |

### Double-Spend Proofs — `/v6/full-node/dsproof`

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Status |
| GET | `/getDSProof/{txid}?verbose=true` | Double-spend proof by txid |

### Fulcrum — `/v6/fulcrum`

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Status |
| GET | `/balance/{address}` | Address balance |
| POST | `/balance` | Bulk balances (`{"addresses": [...]}`) |
| GET | `/utxos/{address}` | Unspent outputs |
| POST | `/utxos` | Bulk UTXOs (`{"addresses": [...]}`) |
| GET | `/transactions/{address}` | Transaction history (up to 100, newest first) |
| GET | `/transactions/{address}/{all_txs}` | All transactions if `all_txs=true` |
| POST | `/transactions` | Bulk transaction history (`{"addresses": [...]}`) |
| GET | `/unconfirmed/{address}` | Unconfirmed transactions |
| POST | `/unconfirmed` | Bulk unconfirmed (`{"addresses": [...]}`) |
| GET | `/tx/data/{txid}` | Transaction data |
| POST | `/tx/data` | Bulk tx data |
| POST | `/tx/broadcast` | Broadcast transaction (`{"txHex": "..."}`) |
| GET | `/block/headers/{height}?count=1` | Block headers at height |
| POST | `/block/headers` | Bulk block headers |

### SLP Tokens — `/v6/slp`

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Status |
| GET | `/status` | SLP service status |
| POST | `/address` | Query by SLP address (`{"address": "..."}`) |
| POST | `/txid` | Query by txid (`{"txid": "..."}`) |
| POST | `/token` | Query token info |
| POST | `/token/data` | Token data (transforms response to genesisData format) |

### Encryption — `/v6/encryption`

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Status |
| GET | `/publickey/{address}` | Recover public key from on-chain tx history |

### Price — `/v6/price`

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Status |
| GET | `/bchusd` | BCH/USD price (`{"usd": 450.12}`) |
| GET | `/psffpp` | PSF file-pinning price (requires `PSFFPP_PROXY_URL`) |

```bash
# Example: get address balance
curl http://localhost:5943/v6/fulcrum/balance/bitcoincash:qp3sn6vlwz28ntmf3wmr7trr96qtt6sgm5mzm97yg

# Example: bulk UTXOs
curl -X POST http://localhost:5943/v6/fulcrum/utxos \
  -H "Content-Type: application/json" \
  -d '{"addresses": ["bitcoincash:qp3sn6vlwz28ntmf3wmr7trr96qtt6sgm5mzm97yg"]}'

# Example: broadcast transaction
curl -X POST http://localhost:5943/v6/fulcrum/tx/broadcast \
  -H "Content-Type: application/json" \
  -d '{"txHex": "0200000001..."}'
```

## Error Responses

All errors return consistent JSON:

```json
{"error": "description of what went wrong"}
```

| Status | Meaning |
|--------|---------|
| 400 | Bad request (invalid address, missing fields, bulk limit exceeded, RPC error) |
| 401 | Unauthorized (invalid Bearer token) |
| 402 | Payment required (x402 payment missing or invalid) |
| 429 | Too many requests (rate-limited by backend) |
| 501 | Not implemented (backend not configured) |
| 502 | Bad gateway (backend 5xx error) |
| 504 | Gateway timeout (backend did not respond in time) |

## Architecture

```
HTTP Request → Axum (CORS, tracing, 300s timeout, path normalization)
  → Auth middleware (Bearer token and/or x402 payment verification)
    → Route handler (input validation, address parsing)
      → FullNodeClient (BCH RPC via reqwest, Basic auth)
      → HttpProxyClient (Fulcrum REST, auto-retry on 5xx)
      → HttpProxyClient (SLP indexer REST, auto-retry on 5xx)
```

**Modules:**
- `config` — 19 env vars with defaults and boolean parsing
- `clients::full_node` — BCH full node JSON-RPC proxy
- `clients::fulcrum` — Fulcrum REST API proxy with retry
- `clients::slp` — SLP indexer REST API proxy with retry
- `clients::mod` — `ApiError` enum, retry logic, HTTP status mapping
- `middleware::auth` — Bearer token and x402 payment verification
- `routes` — 9 route modules, 72 endpoints, input validation helpers

**Middleware stack:** CORS (allow all origins) → request tracing → 300s timeout → trailing slash normalization → auth (conditional).

## Development

```bash
cargo test          # Run tests
cargo clippy        # Lint
cargo fmt           # Format
```

### Make Targets

| Target | Description |
|--------|-------------|
| `make build` | Debug build |
| `make release` | Release build (LTO, stripped, size-optimized) |
| `make run` | `cargo run` |
| `make test` | `cargo test` |
| `make clean` | `cargo clean` |
| `make lint` | `cargo clippy -- -D warnings` |
| `make fmt` | `cargo fmt --check` |
| `make cross-arm64` | Cross-compile for aarch64 (requires `cross`) |
| `make static-arm64` | Static musl build for aarch64 |
| `make docker` | `docker build -t bch-api-rust .` |
| `make docker-run` | `docker compose up` |

## Docker

Multi-stage build: `rust:1.83-slim` (builder) → `debian:bookworm-slim` (runtime).

```bash
# Build and run
docker compose up

# Connects to host services via host.docker.internal
```

The `docker-compose.yml` maps `host.docker.internal` to the host gateway, so full node, Fulcrum, and SLP indexer instances running on the host are reachable without extra network config.

## Cross-Compilation

For ARM64 targets (e.g., Raspberry Pi 5):

```bash
# Dynamic linking (requires cross: cargo install cross --git https://github.com/cross-rs/cross)
make cross-arm64

# Static binary (musl)
make static-arm64
```
