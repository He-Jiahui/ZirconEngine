---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-editor554-scaled-image-x-sample-cache.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/raster/scaled.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/raster/scaled.rs
---

# Editor970 Editor554 scaled-image X sample cache

Scaled-image rasterization now computes X-axis source samples once per draw and
reuses them across destination rows, while preserving Y sampling and bilinear
write order. Marker `EDITOR554_X_AXIS_SAMPLE_CACHE_BENCH_V1` reports the
cached-versus-direct sample benchmark; the focused equivalence test keeps the
coordinates bit-identical. The current-source Editor library binary was run as
one grouped `editor554/editor555` selector: the two Editor554 behavior/source
contracts passed, and its debug marker reported `legacy=9.5132ms` versus
`optimized=1.5092ms`. The marker is a local debug receipt, not managed Release
evidence; managed Editor Release validation remains pending.
