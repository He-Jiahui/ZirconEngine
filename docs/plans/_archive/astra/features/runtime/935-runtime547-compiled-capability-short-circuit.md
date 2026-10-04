---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime547-compiled-capability-short-circuit.md
implementation_files:
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/runtime_feature_flags.rs
tests:
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/runtime_feature_flags.rs
---

# Runtime935 Runtime547 compiled capability short circuit

Hybrid-GI and virtual-geometry capability resolution now short-circuits
independently after the first match while preserving late and absent capability
scans. The focused behavior test covers first, late, absent, and repeated
declarations.

Marker `RUNTIME547_COMPILED_CAPABILITY_SHORT_CIRCUIT_BENCH_V1` reports the
131,072-to-2 resolution model. Managed Release validation remains pending.
