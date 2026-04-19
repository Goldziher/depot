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

## Consequences

- Each adapter is self-contained: protocol-specific types, handlers, and upstream client in one module.
- Adding a new protocol requires no changes to existing code — only a new module and router registration.
- Feature flags gate each adapter, so unused protocols are not compiled.
- Adapters share no protocol-specific logic with each other; all shared behavior goes through `PackageService`.
