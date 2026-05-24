# Depot

Self-hosted, armored universal package registry.

Depot speaks native registry protocols and acts as a pull-through cache between package manager clients and upstream registries. Artifacts are stored with blake3 integrity verification and policy enforcement. Depot also supports local hosted publishing through native upload routes and an explicit operator publish command. At-rest encryption, rate limiting, full sync, lockfile update workflows, and upstream publish forwarding are deferred production-hardening work.

## Registry Support

| Protocol | Spec | Status |
|----------|------|--------|
| PyPI | PEP 503/691 Simple Repository API | Working (`pip install` verified) |
| npm | Registry API | Working (`npm install` verified) |
| Cargo | Sparse Index (RFC 2789) | Working (`cargo fetch` verified) |
| Hex | Repository API | Working (`mix hex.package fetch` verified) |
| Maven | Maven Central-compatible artifact layout | MVP pull-through + local publishing adapter |
| RubyGems | Bundler Compact Index | MVP pull-through + local publishing adapter |
| NuGet | V3 restore API | MVP pull-through + local publishing adapter |
| pub.dev | Hosted Pub Repository v2 | MVP pull-through + local publishing adapter |

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

# Write a minimal config
cargo run -p depot-cli -- config init

# Run without a config file and inspect registries
cargo run -p depot-cli -- --no-config --storage-backend memory registry status

# Fetch one artifact through Depot's cache path
cargo run -p depot-cli -- package fetch pypi six 1.16.0 six-1.16.0.tar.gz

# Publish one explicit local artifact when publishing is enabled in config
cargo run -p depot-cli -- package publish pypi ./dist/example-0.1.0.tar.gz \
  --name example --package-version 0.1.0 --license MIT

# Start the stdio MCP server for agent integrations
cargo run -p depot-cli -- --no-config --storage-backend memory mcp serve

# Run unit and offline conformance tests
cargo test --workspace

# Run live native-client E2E tests (requires network and package-manager CLIs)
task test:e2e

# Lint
cargo clippy --workspace
```

## Architecture

Depot uses a hexagonal architecture with Tower middleware. The crate structure is:

| Crate | Role |
|-------|------|
| `depot-core` | Domain types, port traits, policy engine, lock file, config |
| `depot-service` | Application service layer (`CachingPackageService`): pull-through caching, blake3 integrity, policy enforcement |
| `depot-ops` | Shared local operator API used by CLI and MCP |
| `depot-storage` | OpenDAL-backed `StoragePort` (feature-gated: fs, S3, GCS, memory) |
| `depot-adapters` | Protocol adapters (axum routers) + upstream clients (feature-gated per ecosystem) |
| `depot-server` | Axum app assembly, Tower middleware, shared `AppState` |
| `depot-cli` | Binary crate, Clap CLI, stdio MCP server |

See the [Architecture Overview](docs/architecture.md) for Mermaid diagrams and detailed component descriptions.

### ADRs

- [0001 — Hexagonal Architecture](docs/adr/0001-hexagonal-architecture.md)
- [0002 — Tower Middleware](docs/adr/0002-tower-middleware.md)
- [0003 — OpenDAL Storage](docs/adr/0003-opendal-storage.md)
- [0004 — Blake3 & Lock File](docs/adr/0004-blake3-lockfile.md)
- [0005 — Protocol Adapters](docs/adr/0005-protocol-adapters.md)
- [0006 — Feature Flags](docs/adr/0006-feature-flags.md)
- [0007 — JSON Schema Validation](docs/adr/0007-json-schema-validation.md)
- [0008 — Registry Expansion](docs/adr/0008-registry-expansion.md)
- [0009 — Publishing and Upload Workflows](docs/adr/0009-publishing-upload-workflows.md)
- [0010 — CLI and MCP Operations](docs/adr/0010-cli-mcp-operations.md)

### Schemas

Schema provenance, fetched upstream artifacts, Depot-derived JSON Schemas, and grammar fixtures are in [`schemas/`](schemas/):

- [`schemas/registries/`](schemas/registries/) — derived registry schemas where the protocol is JSON-like
- [`schemas/depot/`](schemas/depot/) — config and lockfile schemas
- [`schemas/README.md`](schemas/README.md) — source links and registry-by-registry derivation notes

## CLI And MCP

The CLI and MCP server share the same local operations layer. Both can run without `depot.toml`
using built-in defaults plus explicit flags.

Common CLI operations:

```bash
depot config show
depot config validate
depot registry status
depot package list pypi
depot package versions npm is-odd
depot package metadata cargo once_cell 1.19.0
depot package fetch npm is-odd 3.0.1 is-odd-3.0.1.tgz --output ./is-odd.tgz
depot cache delete-artifact npm is-odd 3.0.1 is-odd-3.0.1.tgz --yes
```

Use `--output json` for machine-readable output. MCP runs over stdio:

```bash
depot mcp serve
depot mcp serve --allow-writes
```

MCP read tools are always available. Mutating tools, including publish, yank, unyank, and cache
delete, require `--allow-writes`.

## License

[BUSL-1.1](LICENSE)
