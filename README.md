# Depot

Self-hosted, armored universal package registry.

Depot speaks native registry protocols (PyPI, npm, Cargo, Hex) and acts as a pull-through cache between package manager clients and upstream registries. Artifacts are stored with blake3 integrity verification, policy enforcement, and optional at-rest encryption.

## Registry Support

| Protocol | Spec | Status |
|----------|------|--------|
| PyPI | PEP 503/691 Simple Repository API | In progress |
| npm | Registry API | In progress |
| Cargo | Sparse Index (RFC 2789) | In progress |
| Hex | Repository API | In progress |

## Requirements

- Rust (edition 2024 — requires Rust 1.85+)
- [Task](https://taskfile.dev/) (optional, for dev workflow commands)

## Getting Started

```bash
# First-time setup (installs hooks, generates AI config)
task setup

# Build
cargo build --workspace

# Run the server
cargo run -p depot-cli -- serve

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace
```

## Architecture

Depot uses a hexagonal architecture with Tower middleware. See the [Architecture Overview](docs/architecture.md) for Mermaid diagrams and detailed component descriptions.

### ADRs

- [0001 — Hexagonal Architecture](docs/adr/0001-hexagonal-architecture.md)
- [0002 — Tower Middleware](docs/adr/0002-tower-middleware.md)
- [0003 — OpenDAL Storage](docs/adr/0003-opendal-storage.md)
- [0004 — Blake3 & Lock File](docs/adr/0004-blake3-lockfile.md)
- [0005 — Protocol Adapters](docs/adr/0005-protocol-adapters.md)
- [0006 — Feature Flags](docs/adr/0006-feature-flags.md)
- [0007 — JSON Schema Validation](docs/adr/0007-json-schema-validation.md)

### Schemas

Canonical JSON Schemas for all registry protocols and depot's own formats are in [`schemas/`](schemas/):

- [`schemas/registries/`](schemas/registries/) — PyPI, npm, Cargo, Hex response schemas
- [`schemas/depot/`](schemas/depot/) — config and lockfile schemas

## License

[BUSL-1.1](LICENSE)
