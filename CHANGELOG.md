# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `depot-service` crate: application service layer with `CachingPackageService` implementing pull-through caching, blake3 integrity verification, and policy enforcement.
- Blake3 integrity verification: hash computed on first artifact fetch, stored as `.blake3` sidecar file, verified on every cache read.
- Policy enforcement on `get_artifact`: `blocked_packages` checked before fetching to prevent policy bypass.
- Adapter state traits (`HasPypiState`, `HasNpmState`, `HasCargoState`, `HasHexState`) for accessing `PackageService` + ecosystem-specific upstream client.
- npm recursive BFS dependency prefetch with visited set and max depth of 10 levels.
- Hex protobuf registry proxy at `/hex/packages/{name}` for mix checksum verification.
- 5-minute TTL cache for all upstream client responses using `(Instant, T)` tuples.
- Integration test crate (`tests/integration/`) with 31 tests covering pip, npm, cargo, and mix client workflows.
- All four registries now pass client-level integration tests.

### Changed

- Adapters now serve cached upstream data directly with URL rewriting instead of reconstructing responses from `VersionMetadata`. This preserves protocol-specific fields (npm dependencies, PyPI requires-python, Cargo deps/features).
- npm adapter uses raw `serde_json::Value` instead of typed `NpmPackument` struct to handle the variety of npm field shapes.
- Upstream hashes preserved in `ArtifactDigest.upstream_hashes`.
- Dependency flow updated: `depot-server -> depot-service -> depot-core` added alongside existing paths.
