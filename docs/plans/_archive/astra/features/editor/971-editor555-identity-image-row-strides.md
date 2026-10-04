---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-editor555-identity-image-row-strides.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/raster/identity.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/raster/identity.rs
---

# Editor971 Editor555 identity-image row strides

The opaque identity-image fast path now precomputes row byte length, source and
destination strides, and initial offsets, then advances those offsets by
addition in both passes. Alpha rejection still completes before mutable frame
access. Marker `EDITOR555_IDENTITY_ROW_STRIDE_BENCH_V1` reports the modeled
row-address benchmark, with focused tests covering transparent rejection and
clipped opaque copies. The same current-source grouped library binary ran the
Editor555 source contract and marker successfully; its local debug marker
reported `legacy=1.5398ms` versus `optimized=1.1543ms`. This is local debug
evidence only, not the managed Release gate; managed Editor Release validation
remains pending.
