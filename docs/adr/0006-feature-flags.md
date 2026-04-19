# ADR-0006: Feature Flags for Compile-Time Configuration

## Status

Accepted

## Context

Depot supports multiple protocol adapters and storage backends. Not every deployment needs all of them. Compiling unused adapters increases binary size and build time, and pulls in unnecessary dependencies.

## Decision

We use Cargo feature flags to control which components are compiled:

### `depot-adapters`

- `pypi` (default) — PyPI PEP 503 adapter
- `npm` (default) — npm registry adapter
- `cargo-registry` (default) — Cargo sparse index adapter
- `hex` (default) — Hex.pm adapter

### `depot-storage`

- `backend-fs` (default) — local filesystem via OpenDAL
- `backend-s3` — S3-compatible storage
- `backend-gcs` — Google Cloud Storage
- `backend-memory` — in-memory (testing)

### `depot-core`

- `encryption` — at-rest encryption support

### `depot-cli`

- `full` (default) — all adapters + `backend-fs`

A minimal build (e.g. PyPI-only with S3) would use:

```sh
cargo build -p depot-cli --no-default-features --features pypi,backend-s3
```

## Consequences

- Deployments only compile what they need, reducing binary size and attack surface.
- Feature flags are additive (Cargo convention) — combining features never breaks builds.
- CI must test feature combinations to avoid conditional compilation bugs.
- Default features include all adapters + filesystem storage, so `cargo build` produces a fully-featured binary.
