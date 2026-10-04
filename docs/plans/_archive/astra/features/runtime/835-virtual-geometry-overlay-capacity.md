---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/49-runtime-debug-gizmo-command-buffer-retained-extract-filter-budget-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/49/2026-09-19-virtual-geometry-overlay-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
tests:
  - tools/tests/test_runtime_virtual_geometry_overlay_capacity_performance_contract.py
---

# Runtime835 · virtual geometry overlay capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime49 virtual-geometry debug overlays | Reserve bounded BVH/visbuffer gizmo, per-node line, and fixed marker-line capacities while preserving filtering, order, parent connectors, colors, and empty fallbacks. | TDD source/model contract `3/3`; lower Rust source regression and ignored `RUNTIME835_VIRTUAL_GEOMETRY_OVERLAY_CAPACITY_BENCH_V1` marker are wired; deterministic dense model removes `15`, `11`, and `3` geometric growth events; focused batch `632/632` across `171` files and broad non-tooling contract batch `3349/3349` across `825` files both pass. Managed Cargo/Release and overlay product percentile evidence remain pending. | implemented_pending_validation |

The ignored marker was then tightened to model both zero-capacity and exact
bounded-capacity starts for each collector. The three-slice source/model
contracts were rerun together (`9/9`), and the exact Runtime835 Rustfmt check
still passes; this remains local evidence only.

## Complexity boundary

This slice changes only temporary collector capacity in the Runtime debug
overlay projection. It does not change the virtual-geometry snapshot owner,
cluster lookup, gizmo kind, line geometry, parent resolution, visibility,
coloring, or frame publication semantics.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs` | `477C750E0B44B40F478091DD002D9113AFD512935ACEC5D6310CC890C19D0585` |
| `tools/tests/test_runtime_virtual_geometry_overlay_capacity_performance_contract.py` | `A51EBA15A83534CD67819ED4AA035B70F186B5E74965D07929C6BE2D48ABA9E4` |

## Managed gate

No Cargo process is started locally and the coordinator is not polled. Keep
this entry `implemented_pending_validation` until the owner-attributed batched
Windows Release lane proves current-source compilation, overlay parity,
allocation behavior, and Runtime overlay product p50/p95/p99 evidence.
