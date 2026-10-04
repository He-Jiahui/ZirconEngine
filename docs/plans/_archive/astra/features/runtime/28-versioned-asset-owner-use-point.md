---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09d/2026-09-10-versioned-asset-owner-use-point.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
---

# Versioned Asset Owner Use-Point Resolution

The render-asset semantic executor no longer stores a long-lived concrete
`ProjectAssetManager`. It keeps the weak/versioned `ProjectAssetManagerAccess` handle,
resolves it when admitting or maintaining work, and closes on resolution failure before
returning a typed error.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime09D/M5 | Remove concrete asset-manager ownership from the semantic executor owner and guard manager replacement/teardown at use points | implemented_pending_validation | Frameworks01/05 plus UI access-boundary contract `31/31`; bounded Runtime/Editor source-contract batch `70/70`; scoped Rustfmt and diff checks; managed Runtime/Editor Cargo and release percentile evidence remain pending |
