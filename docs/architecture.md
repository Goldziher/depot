# Depot Architecture

## Overview

Depot is a self-hosted, armored universal package registry. It acts as a pull-through cache and policy enforcement layer between package manager clients (pip, npm, cargo, mix) and their upstream registries (PyPI, npmjs, crates.io, hex.pm).

Clients configure their package manager to point at a depot instance. Depot speaks each registry's native protocol, so no client-side tooling changes are needed beyond the registry URL.

## Hexagonal Architecture

```text
┌──────────────────────────────────────────────────────────────┐
│                      Tower Middleware                         │
│  (tracing, CORS, rate limiting, auth, compression)           │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐          │
│   │  PyPI   │ │   npm   │ │  Cargo  │ │   Hex   │          │
│   │ Adapter │ │ Adapter │ │ Adapter │ │ Adapter │          │
│   └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘          │
│        │           │           │           │                 │
│        └───────────┴─────┬─────┴───────────┘                 │
│                          │                                   │
│              ┌───────────▼───────────┐                       │
│              │    PackageService     │  ◄── Inbound Port     │
│              │  (core domain logic)  │                       │
│              └───┬───────────────┬───┘                       │
│                  │               │                           │
│        ┌─────────▼──┐    ┌──────▼────────┐                   │
│        │ StoragePort │    │UpstreamClient │  ◄── Outbound     │
│        └─────────┬──┘    └──────┬────────┘      Ports        │
│                  │              │                             │
├──────────────────┼──────────────┼────────────────────────────┤
│                  │              │                             │
│         ┌────────▼──┐    ┌─────▼──────┐                      │
│         │  OpenDAL  │    │  Upstream   │                      │
│         │  Storage  │    │  HTTP       │                      │
│         │ (fs/S3/…) │    │  Clients   │                      │
│         └───────────┘    └────────────┘                      │
└──────────────────────────────────────────────────────────────┘
```

## Request Flow

1. Client sends a request in the native protocol (e.g. `pip install --index-url http://depot/pypi/simple/ requests`)
2. The protocol adapter translates the request into a `PackageService` call
3. `PackageService` checks local storage for the artifact
4. If not cached: fetches from upstream via `UpstreamClient`, verifies integrity (blake3), applies policy checks, stores via `StoragePort`
5. Returns the artifact to the adapter, which formats the response in the native protocol

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `depot-core` | Domain types, port traits (`PackageService`, `StoragePort`, `UpstreamClient`), integrity (blake3), policy engine, lock file format, config |
| `depot-storage` | `StoragePort` implementation via OpenDAL. Feature-gated backends: fs, S3, GCS, memory |
| `depot-adapters` | Inbound adapters (axum routers per protocol) + outbound upstream clients. Feature-gated per ecosystem |
| `depot-server` | Axum application assembly, Tower middleware stack, shared state |
| `depot-cli` | Binary crate. CLI (clap) with commands: `serve`, `sync`, `lock`, `config` |

Dependency flow: `depot-cli → depot-server → depot-adapters → depot-core`, `depot-server → depot-storage → depot-core`

## Storage Key Scheme

Artifacts are stored with the key: `<ecosystem>/<name>/<version>/<filename>`

Examples:

- `pypi/requests/2.31.0/requests-2.31.0.tar.gz`
- `npm/lodash/4.17.21/lodash-4.17.21.tgz`
- `cargo/serde/1.0.200/serde-1.0.200.crate`

## Lock File

Depot uses its own TOML-based lock file (`depot-lock.toml`) with blake3 hashes for integrity verification. The format is ecosystem-agnostic — a single lock file can pin packages across all supported ecosystems. See [ADR-0004](adr/0004-blake3-lockfile.md).

## Feature Flags

Compile-time feature flags allow minimal builds. See [ADR-0006](adr/0006-feature-flags.md).

## Adding a New Protocol Adapter

1. Create a module under `depot-adapters/src/<protocol>/` with `mod.rs`, `models.rs`, `upstream.rs`
2. Implement an axum `Router` that translates protocol requests into `PackageService` calls
3. Implement `UpstreamClient` for fetching from the upstream registry
4. Add a feature flag in `depot-adapters/Cargo.toml`
5. Register the router in `depot-server/src/app.rs`

No changes to core types or traits required.
