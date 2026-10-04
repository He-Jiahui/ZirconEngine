---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime531-hybrid-gi-sideband-binary-lookup.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
---

# Runtime923 Runtime531 hybrid-GI sideband lookup

Prepared sorted sidebands now validate canonical ordering once and use binary
lookup for resident probes; reordered or duplicate sidebands keep the previous
first-match fallback.

Marker `RUNTIME531_HYBRID_GI_SIDEBAND_BINARY_LOOKUP_BENCH_V1` reports the
canonical comparison model. Managed Release validation remains pending after
the historical snapshot-stale handoff was repaired.
