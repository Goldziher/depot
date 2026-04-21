# ADR-0004: Blake3 Integrity and Depot Lock File Format

## Status

Accepted

## Context

Each package ecosystem uses different hash algorithms (SHA256 for PyPI, SHA1/SHA512 for npm, SHA256 for Cargo). We need a consistent integrity guarantee across all ecosystems, and a way to pin exact artifact versions with verified hashes.

## Decision

### Blake3 for integrity

All artifacts stored in depot are hashed with [BLAKE3](https://github.com/BLAKE3-team/BLAKE3). Blake3 is:

- Faster than SHA-256 (especially on large artifacts)
- Cryptographically secure
- Consistent across ecosystems (we compute our own hash regardless of what upstream uses)

Upstream-provided hashes (SHA-256, etc.) are verified on fetch, then depot's blake3 hash becomes the canonical integrity check.

### Current Implementation

Blake3 integrity is implemented in `CachingPackageService` (`depot-service`):

- On first artifact fetch, the blake3 hash is computed and stored as a `.blake3` sidecar file alongside the artifact in storage (e.g., `pypi/requests/2.31.0/requests-2.31.0.tar.gz.blake3`).
- On every subsequent cache read, the sidecar hash is loaded and verified against the artifact data.
- Upstream-provided hashes are preserved in `ArtifactDigest.upstream_hashes` for ecosystems that provide them.

### Depot lock file

We define our own TOML-based lock file format (`depot-lock.toml`) that is ecosystem-agnostic:

```toml
[metadata]
schema_version = 1
generated_at = "2026-04-19T10:30:00Z"
depot_version = "0.1.0"

[[packages]]
ecosystem = "pypi"
name = "requests"
version = "2.31.0"
artifacts = [
  { filename = "requests-2.31.0.tar.gz", blake3 = "d1e2f3...", size = 110293 }
]
resolved_from = "https://pypi.org"
pinned = true
```

The lock file records:

- Which ecosystem and version was resolved
- Blake3 hash and size of every artifact
- Which upstream it was fetched from
- Whether the pin is explicit or auto-resolved

## Consequences

- A single lock file captures the full dependency state across all ecosystems.
- Integrity verification is uniform — one algorithm, one code path.
- The lock file is human-readable (TOML) and diff-friendly for version control.
- We do not replace ecosystem-specific lock files (package-lock.json, Cargo.lock, etc.) — depot's lock file tracks what's in the registry, not what's in a project.
