---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/581/2026-08-31-texture-probe-selection-short-circuit.md
related_records:
  - docs/plans/astra/features/runtime/888-runtime581-visible-spatial-hash-index.md
  - docs/plans/astra/features/editor/945-editor581-badge-overlay-clip-early-exit.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/selection.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/selection.rs
---

# Runtime889 Runtime581 Texture-Probe Selection Short Circuit

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Texture-target probe selection | Texture-target planar-reflection selection returns on the first valid GPU-parameter probe; non-texture selection keeps its deterministic minimum-ID policy and absent-probe behavior. | Behavior contract covers first, later, and absent valid candidates; source test preserves the texture-only short-circuit boundary. |
| 性能门禁 | 2,048 candidates with the first valid match avoid the former full scan and minimum-ID reduction across 8,192 selections. | ignored marker `RUNTIME581_TEXTURE_PROBE_SELECTION_BENCH_V1` requires optimized P95 ≤50% of the legacy scan; managed Runtime Release receipt remains pending. |

- `selection.rs` contains the Runtime581 behavior contract and release marker.
- No tooling changes; the source plan's combined Runtime581/Editor581 handoff remains the managed gate.

### Grouped validation submission (2026-09-25)

This slice is included in the exact `optimization_batch_gz` replacement wave:
Runtime development PTY `81972`, Editor development PTY `21257`, Runtime02
Release PTY `20043`, and Editor Release PTY `50976`. All four wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.
