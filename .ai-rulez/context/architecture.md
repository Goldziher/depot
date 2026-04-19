---
priority: critical
---

# Architecture

Depot is a self-hosted, armored universal package registry built with hexagonal architecture.

## Crate Structure

All code lives under `crates/` — there is no top-level `src/`.

| Crate | Role |
|-------|------|
| `depot-core` | Domain types, port traits (`PackageService`, `StoragePort`, `UpstreamClient`), blake3 integrity, policy engine, lock file, config |
| `depot-storage` | OpenDAL-backed `StoragePort` implementation. Feature-gated backends: `backend-fs`, `backend-s3`, `backend-gcs`, `backend-memory` |
| `depot-adapters` | Inbound protocol adapters (axum routers) + outbound upstream clients. Feature-gated: `pypi`, `npm`, `cargo-registry`, `hex` |
| `depot-server` | Axum app assembly, Tower middleware stack (tracing, CORS, auth, compression), shared `AppState` |
| `depot-cli` | Binary crate. Clap CLI with commands: `serve`, `sync`, `lock`, `config` |

## Dependency Flow

`depot-cli → depot-server → depot-adapters → depot-core`
`depot-server → depot-storage → depot-core`

The core crate has zero framework dependencies — all I/O goes through port traits.

## Key Design Decisions

- Protocol adapters translate native requests (PEP 503, npm registry API, Cargo sparse index, Hex API) into `PackageService` trait calls
- Pull-through cache: fetch from upstream on miss, verify with blake3, apply policy, store via OpenDAL, serve
- Storage keys: `<ecosystem>/<name>/<version>/<filename>`
- Lock file: TOML-based, ecosystem-agnostic, blake3 hashes
- Feature flags control compile-time inclusion of adapters and storage backends
- TOML config with clap CLI

## ADRs

Architecture Decision Records are in `docs/adr/`. Read them before making architectural changes.
