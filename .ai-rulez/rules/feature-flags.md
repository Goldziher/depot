---
priority: high
---

# Feature Flags

- All protocol adapters are gated behind feature flags in `depot-adapters`.
- All storage backends are gated behind feature flags in `depot-storage`.
- Feature flags are additive — combining features must never break builds.
- Use `#[cfg(feature = "...")]` on modules, not on individual functions.
- When adding a new adapter or backend, add corresponding feature flags and update `depot-cli`'s `full` feature.
