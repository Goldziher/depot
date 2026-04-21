# ADR-0001: Hexagonal Architecture

## Status

Accepted

## Context

Depot must support multiple package registry protocols (PyPI, npm, Cargo, Hex) and multiple storage backends (filesystem, S3, GCS). These integrations have different APIs, serialization formats, and behaviors. We need an architecture that isolates the core domain logic from these external concerns.

## Decision

We adopt a hexagonal (ports and adapters) architecture. The core domain defines trait-based ports:

- **`PackageService`** (inbound port): the API that protocol adapters call. Defines operations like `list_versions`, `get_artifact`, `get_version_metadata`.
- **`StoragePort`** (outbound port): abstraction over artifact storage. Implementations live in `depot-storage`.
- **`UpstreamClient`** (outbound port): abstraction over upstream registry communication. Implementations live in `depot-adapters`.

The `depot-core` crate has zero dependencies on web frameworks, storage libraries, or HTTP clients. All I/O happens through trait implementations injected at startup.

## Implementation Notes

The `depot-service` crate provides the application service layer between adapters and core. `CachingPackageService` implements the `PackageService` port trait with pull-through caching, blake3 integrity verification, and policy enforcement.

Each protocol adapter defines its own state trait (`HasPypiState`, `HasNpmState`, `HasCargoState`, `HasHexState`) that provides access to both the `PackageService` and the ecosystem-specific upstream client. This allows handlers to serve cached upstream data directly (preserving protocol-specific fields) while still going through `PackageService` for the caching lifecycle.

## Consequences

- Core business logic (policy enforcement, integrity verification, lock file management) is testable without any I/O.
- Adding a new protocol or storage backend requires only a new adapter — no core changes.
- The indirection adds a trait boundary at every I/O point, which has minor ergonomic cost but no meaningful runtime cost (monomorphization or `Arc<dyn Trait>`).
- The `HasXxxState` adapter traits allow protocol-specific data to flow through without lossy conversion to domain types.
