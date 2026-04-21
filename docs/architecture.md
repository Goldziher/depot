# Architecture

## Overview

Depot is a self-hosted, armored universal package registry. It acts as a pull-through cache and policy enforcement layer between package manager clients and upstream registries.

## Hexagonal Architecture

```mermaid
graph TB
    subgraph Clients
        pip[pip]
        npm_cli[npm]
        cargo_cli[cargo]
        mix[mix]
    end

    subgraph "Tower Middleware"
        trace[TraceLayer]
        cors[CorsLayer]
        auth[AuthLayer]
        compress[CompressionLayer]
    end

    subgraph "Inbound Adapters"
        pypi_adapter[PyPI Adapter<br/>PEP 503/691]
        npm_adapter[npm Adapter<br/>Registry API]
        cargo_adapter[Cargo Adapter<br/>Sparse Index]
        hex_adapter[Hex Adapter<br/>Repository API]
    end

    subgraph "Application Service"
        caching_svc[CachingPackageService<br/>depot-service]
        policy[Policy Engine]
        integrity[Blake3 Integrity<br/>sidecar .blake3 files]
        lockfile[Lock File]
    end

    subgraph "Core Domain"
        svc[PackageService trait<br/>depot-core]
    end

    subgraph "Outbound Adapters"
        storage[StoragePort<br/>OpenDAL]
        upstream_pypi[PyPI Upstream<br/>5min TTL cache]
        upstream_npm[npm Upstream<br/>5min TTL cache]
        upstream_cargo[Cargo Upstream<br/>5min TTL cache]
        upstream_hex[Hex Upstream<br/>5min TTL cache]
    end

    subgraph "Storage Backends"
        fs[Filesystem]
        s3[S3 / MinIO]
        gcs[GCS]
    end

    pip --> trace
    npm_cli --> trace
    cargo_cli --> trace
    mix --> trace

    trace --> cors --> auth --> compress

    compress --> pypi_adapter
    compress --> npm_adapter
    compress --> cargo_adapter
    compress --> hex_adapter

    pypi_adapter --> caching_svc
    npm_adapter --> caching_svc
    cargo_adapter --> caching_svc
    hex_adapter --> caching_svc

    pypi_adapter -.-> upstream_pypi
    npm_adapter -.-> upstream_npm
    cargo_adapter -.-> upstream_cargo
    hex_adapter -.-> upstream_hex

    caching_svc --> svc
    caching_svc --> policy
    caching_svc --> integrity
    caching_svc --> lockfile
    caching_svc --> storage

    storage --> fs
    storage --> s3
    storage --> gcs
```

## Request Flow

```mermaid
sequenceDiagram
    participant Client
    participant Adapter
    participant UpstreamClient
    participant CachingPackageService
    participant Storage

    Client->>Adapter: GET /pypi/simple/requests/
    Adapter->>CachingPackageService: list_versions(PyPI, "requests")
    CachingPackageService->>UpstreamClient: fetch_versions("requests")
    UpstreamClient-->>CachingPackageService: version list
    CachingPackageService-->>Adapter: VersionMetadata

    Note over Adapter,UpstreamClient: Adapter serves cached upstream response directly<br/>with URL rewriting (preserves protocol-specific fields)

    Adapter->>UpstreamClient: get cached response
    UpstreamClient-->>Adapter: cached response (5min TTL)
    Adapter-->>Client: PEP 691 JSON (URLs rewritten to depot)

    Client->>Adapter: GET /pypi/artifacts/requests/requests-2.31.0.tar.gz
    Adapter->>CachingPackageService: get_artifact(PyPI, "requests", ...)

    CachingPackageService->>CachingPackageService: check blocked_packages policy
    CachingPackageService->>Storage: exists("pypi/requests/...")
    alt Cached
        Storage-->>CachingPackageService: artifact data
        CachingPackageService->>CachingPackageService: verify blake3 from .blake3 sidecar
    else Not cached
        CachingPackageService->>UpstreamClient: fetch_artifact(artifact_id)
        UpstreamClient-->>CachingPackageService: artifact bytes
        CachingPackageService->>CachingPackageService: compute blake3, store .blake3 sidecar
        CachingPackageService->>Storage: put(key, data)
    end

    CachingPackageService-->>Adapter: artifact bytes
    Adapter-->>Client: artifact response
```

## Crate Dependencies

```mermaid
graph LR
    cli[depot-cli] --> server[depot-server]
    server --> adapters[depot-adapters]
    server --> service[depot-service]
    server --> storage[depot-storage]
    adapters --> core[depot-core]
    service --> core
    storage --> core
```

| Crate | Purpose |
|-------|---------|
| `depot-core` | Domain types, port traits (`PackageService`, `StoragePort`, `UpstreamClient`), policy engine, lock file, config |
| `depot-service` | Application service layer. `CachingPackageService`: pull-through caching, blake3 integrity (sidecar `.blake3` files), policy enforcement |
| `depot-storage` | `StoragePort` via OpenDAL — feature-gated backends (fs, S3, GCS, memory) |
| `depot-adapters` | Protocol adapters (axum routers) + upstream clients — feature-gated per ecosystem. Each adapter defines a state trait (`HasPypiState`, etc.) for accessing `PackageService` + upstream client |
| `depot-server` | Axum app assembly, Tower middleware stack, shared `AppState` |
| `depot-cli` | Binary crate, clap CLI: `serve`, `sync`, `lock`, `config` |
| `tests/integration` | Integration test crate with 31 tests covering pip, npm, cargo, and mix client workflows |

## Storage Key Scheme

```text
<ecosystem>/<name>/<version>/<filename>
```

Examples:

- `pypi/requests/2.31.0/requests-2.31.0.tar.gz`
- `npm/lodash/4.17.21/lodash-4.17.21.tgz`
- `cargo/serde/1.0.200/serde-1.0.200.crate`
- `hex/phoenix/1.7.12/phoenix-1.7.12.tar`

## Registry Protocol Support

| Protocol | Spec | Endpoints | Format |
|----------|------|-----------|--------|
| PyPI | PEP 503/691 | `/pypi/simple/<name>/`, `/pypi/artifacts/<name>/<filename>` | JSON (PEP 691) |
| npm | Registry API | `/npm/<package>`, `/npm/<package>/-/<filename>` | JSON (raw `serde_json::Value`, BFS dep prefetch) |
| Cargo | Sparse Index (RFC 2789) | `/cargo/index/<prefix>/<name>`, `/cargo/api/v1/crates/<name>/<version>/download` | NDJSON |
| Hex | Repository API | `/hex/packages/<name>` (JSON + protobuf registry proxy), `/hex/tarballs/<name>-<version>.tar` | JSON / Protobuf |

## JSON Schemas

Canonical schemas for all registry protocols and depot's own formats live in `schemas/`:

```text
schemas/
├── registries/
│   ├── pypi.schema.json
│   ├── npm.schema.json
│   ├── cargo.schema.json
│   └── hex.schema.json
└── depot/
    ├── config.schema.json
    └── lockfile.schema.json
```

Registry types derive `JsonSchema` via `schemars` and are validated at runtime with `jsonschema`.

## ADRs

- [0001 — Hexagonal Architecture](adr/0001-hexagonal-architecture.md)
- [0002 — Tower Middleware](adr/0002-tower-middleware.md)
- [0003 — OpenDAL Storage](adr/0003-opendal-storage.md)
- [0004 — Blake3 & Lock File](adr/0004-blake3-lockfile.md)
- [0005 — Protocol Adapters](adr/0005-protocol-adapters.md)
- [0006 — Feature Flags](adr/0006-feature-flags.md)
- [0007 — JSON Schema Validation](adr/0007-json-schema-validation.md)
