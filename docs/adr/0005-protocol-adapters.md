# ADR-0005: Protocol Adapters as Axum Routers

## Status

Accepted

## Context

Depot must serve packages using each ecosystem's native protocol so that existing tools (pip, npm, cargo, mix) work without modification. Each protocol has different URL schemes, response formats, and behaviors.

## Decision

Each protocol adapter is implemented as an axum `Router` that:

1. Handles incoming requests in the native protocol format
2. Translates them into `PackageService` trait calls
3. Formats the response back into the native protocol format

Adapters are mounted under path prefixes:

| Prefix | Protocol | Spec |
|--------|----------|------|
| `/pypi` | PEP 503 Simple Repository API | HTML index pages + file downloads |
| `/npm` | npm registry API | JSON metadata + tarball downloads |
| `/cargo` | Cargo sparse index | JSON config + version metadata |
| `/hex` | Hex.pm API | JSON/protobuf metadata + tarball downloads |

Each adapter also provides an `UpstreamClient` implementation for fetching from the corresponding public registry.

## Implementation Notes

### Adapter State Traits

Each adapter defines its own state trait for accessing both `PackageService` and the ecosystem-specific upstream client:

- `HasPypiState` — `package_service` + `pypi_upstream`
- `HasNpmState` — `package_service` + `npm_upstream`
- `HasCargoState` — `package_service` + `cargo_upstream`
- `HasHexState` — `package_service` + `hex_upstream`

This lets handlers serve cached upstream data directly (preserving all protocol-specific fields) while still going through `PackageService` for the caching lifecycle.

### Serving Cached Upstream Data

Adapters call `list_versions` to trigger the caching lifecycle, then serve the upstream client's cached response with URL rewriting rather than reconstructing responses from `VersionMetadata`. This preserves protocol-specific data (npm dependencies, PyPI requires-python, Cargo deps/features) that would be lost in conversion to domain types.

### npm Raw JSON

The npm adapter stores and serves `serde_json::Value` instead of a typed `NpmPackument` struct. This handles the wide variety of npm field shapes without deserialization failures. When a packument is served, all transitive dependencies are pre-fetched using BFS with a visited set and max depth of 10 levels.

### Hex Protobuf Registry Proxy

The Hex adapter includes a protobuf registry proxy at `/hex/packages/{name}` that proxies the protobuf registry entry from `repo.hex.pm`. This is required for mix checksum verification.

### Cache TTL

All upstream client caches use 5-minute TTL via `(Instant, T)` tuples. Cached data is served directly until the TTL expires, then re-fetched from upstream.

## Consequences

- Each adapter is self-contained: protocol-specific types, handlers, and upstream client in one module.
- Adding a new protocol requires no changes to existing code — only a new module and router registration.
- Feature flags gate each adapter, so unused protocols are not compiled.
- Adapters share no protocol-specific logic with each other; all shared behavior goes through `PackageService`.
- All four registries pass client-level integration tests (pip install, npm install, cargo fetch, mix hex.package fetch).
