---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime536-renderer-feature-dedup-owned-insert.md
implementation_files:
  - zircon_runtime/src/graphics/pipeline/validation/validate_renderer_asset.rs
tests:
  - zircon_runtime/src/graphics/pipeline/validation/validate_renderer_asset.rs
---

# Runtime928 Runtime536 renderer-feature dedup owned insert

Renderer asset validation now owns each generated feature name once while
deduplicating through the existing set, eliminating the duplicate owned-string
projection. Validation order and diagnostics remain unchanged.

Marker `RUNTIME536_RENDERER_FEATURE_DEDUP_OWNED_INSERT_BENCH_V1` reports the
clone/allocation reduction model. Managed Release validation remains pending.
