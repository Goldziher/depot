# ADR-0007: JSON Schema Validation with schemars + jsonschema

## Status

Accepted

## Context

Depot proxies four different registry protocols, each with its own response format. We need to:

1. Define Rust types that match the official registry schemas
2. Validate upstream responses at runtime to catch malformed data early
3. Provide canonical JSON Schema files for external tooling and documentation

## Decision

We use two complementary crates:

- **`schemars`** — derive `JsonSchema` on Rust types to generate JSON Schema definitions. The hand-written Rust types are the source of truth; schemas are derived.
- **`jsonschema`** — validate incoming JSON (from upstream registries) against our schemas at runtime before deserialization.

Canonical JSON Schema files are stored at `schemas/` in the repo root, organized as:

```text
schemas/
├── registries/    # Official registry protocol schemas
│   ├── pypi.schema.json
│   ├── npm.schema.json
│   ├── cargo.schema.json
│   └── hex.schema.json
└── depot/         # Depot's own formats
    ├── config.schema.json
    └── lockfile.schema.json
```

Registry types live in `depot-core/src/registry/` with one module per ecosystem. These types use `std::collections::HashMap` (not `AHashMap`) since `schemars` requires `JsonSchema` on all fields, and these are serialization types, not hot-path internal data structures.

## Consequences

- Upstream response validation catches malformed data before it enters our domain logic.
- JSON Schema files serve as machine-readable documentation of every protocol we support.
- `HashMap` in registry types is acceptable — these are used at I/O boundaries, not in tight loops.
- Schema files can be used by external tools (editors, CI, documentation generators).
