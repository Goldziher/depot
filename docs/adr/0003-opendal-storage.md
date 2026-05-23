# ADR-0003: OpenDAL as Storage Abstraction

## Status

Accepted

## Context

Depot must store package artifacts on user-chosen backends: local filesystem for small deployments, S3-compatible storage for production, GCS for Google Cloud users. Writing and maintaining separate implementations for each backend is costly and error-prone.

## Decision

We use [Apache OpenDAL](https://opendal.apache.org/) as the storage abstraction layer. OpenDAL provides a unified `Operator` API across 30+ storage services. Our `StoragePort` trait wraps an OpenDAL `Operator`, translating between depot's domain types and OpenDAL's API.

Storage backends are configured dynamically with an OpenDAL service name and
string options:

```toml
[storage]
backend = "fs"

[storage.options]
root = "./depot-data"
```

Available services are still controlled by feature flags:

- `backend-fs` (default) — local filesystem
- `backend-s3` — S3-compatible (AWS, MinIO, R2)
- `backend-gcs` — Google Cloud Storage
- `backend-memory` — in-memory (for testing)

## Consequences

- Adding a new storage backend is typically a feature flag and documentation addition — OpenDAL already supports the runtime option map.
- We depend on a large external crate, but it's well-maintained (Apache project) and the feature-flag gating keeps binary size manageable.
- The `StoragePort` trait keeps our core decoupled from OpenDAL, so swapping it out (unlikely) would only affect `depot-storage`.
