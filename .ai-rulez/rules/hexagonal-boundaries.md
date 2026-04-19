---
priority: critical
---

# Hexagonal Architecture Boundaries

- `depot-core` must NEVER depend on axum, tower, opendal, reqwest, or any framework crate. All I/O goes through port traits.
- Protocol adapters must NEVER access storage directly — always go through `PackageService`.
- Adapters must NOT share protocol-specific logic with each other. Shared behavior belongs in `PackageService`.
- New dependencies in `depot-core` require justification — keep it framework-free.
