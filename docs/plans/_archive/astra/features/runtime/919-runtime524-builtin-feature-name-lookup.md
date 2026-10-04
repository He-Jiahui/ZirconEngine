---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime524-builtin-feature-name-direct-lookup.md
implementation_files:
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
tests:
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
---

# Runtime919 Runtime524 built-in render-feature name lookup

Authoring-name parsing now uses an exhaustive direct match rather than
recomputing names while scanning all 41 variants. Round-trip and unknown-name
behavior remain covered.

Marker `RUNTIME524_BUILTIN_FEATURE_NAME_LOOKUP_BENCH_V1` reports direct lookup
calls against the legacy candidate-check model. Managed Release compile,
focused tests, and the performance receipt remain pending.
