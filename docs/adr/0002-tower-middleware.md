# ADR-0002: Tower Middleware for Cross-Cutting Concerns

## Status

Accepted

## Context

Cross-cutting concerns — authentication, rate limiting, request tracing, compression, integrity headers — must apply uniformly across all protocol adapters without duplicating logic in each adapter.

## Decision

We use Tower's `Layer`/`Service` abstraction to compose middleware. The stack is assembled in `depot-server/src/app.rs` and wraps all adapter routes.

MVP middleware stack (outermost first):

1. **TraceLayer** — structured request/response logging via `tracing`
2. **CorsLayer** — required for npm web clients and browser-based tooling
3. **Auth** — optional bearer token validation when `auth.enabled = true`
4. **CompressionLayer** — response compression (gzip, brotli, zstd)

Rate limiting and integrity response headers are deferred production-hardening
features. They are not part of the MVP middleware stack.

Protocol adapters are mounted as nested axum routers under path prefixes (`/pypi`, `/npm`, `/cargo`, `/hex`).

## Consequences

- All cross-cutting logic is defined once and applies to every adapter.
- Middleware ordering is explicit and documented.
- Individual adapters remain focused on protocol translation.
- Tower's `Service` trait composes well with axum's router, avoiding framework lock-in for the middleware implementations themselves.
