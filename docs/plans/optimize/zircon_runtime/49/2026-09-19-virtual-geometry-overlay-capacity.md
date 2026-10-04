---
title: Runtime virtual geometry overlay capacity
category: zircon_runtime
report_id: Runtime835-virtual-geometry-overlay-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime835 · virtual geometry overlay capacity

## Scope

When virtual-geometry diagnostics are enabled, the Runtime frame submission
path builds BVH and visbuffer gizmos from bounded debug snapshots. The four
temporary line/gizmo collectors still started at zero capacity even though
their output bounds are known from the source shape.

## Implementation

- Reserve the input-count upper bound before BVH and visbuffer gizmo
  projections, while retaining iterator filtering and source order.
- Reserve at most 13 line segments per BVH node (12 box edges plus one valid
  parent connector).
- Reserve the fixed 16-segment visbuffer marker shape (one stem, three cross
  segments, and one 12-edge box).
- Add the lower source regression and ignored
  `RUNTIME835_VIRTUAL_GEOMETRY_OVERLAY_CAPACITY_BENCH_V1` marker.

No overlay kind, owner, filtering, parent lookup, color, geometry, or empty
fallback semantics changed. Tooling production remains out of scope.

## Deterministic work model

For 4,096 BVH nodes, the old zero-capacity line collector models 15 geometric
growth events while the 53,248-entry bound starts full (`15→0`). The outer
4,096-entry gizmo collector and fixed 16-segment marker model 11→0 and 3→0
events respectively. This is allocation-shape evidence only; it is not a
claim about allocator, CPU, GPU, RSS, or product p50/p95/p99 performance.

## TDD and local evidence

- The source/model contract was intentionally RED against the zero-capacity
  collectors and became GREEN after the bounded reservations and direct
  extensions (`3/3`).
- The lower Rust source regression and ignored Release marker are wired in
  `build_runtime_frame.rs`.
- The ignored marker's allocation model now compares zero-capacity and exact
  bounded-capacity starts for all three collectors; the focused three-slice
  Python contract was rerun as `9/9` after this marker-only refinement.
- Exact-file Rustfmt and Python compilation pass. The merged Runtime/Editor
  capacity/projection batch covers `171` files and passes `632/632` tests in
  `4.660s`; the broader non-tooling `test_*contract.py` batch covers `825`
  files and passes `3349/3349` in `69.899s`, with zero failures, errors, load
  errors, or skips. Managed Cargo/Windows Release and overlay product
  percentile evidence remain pending.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs` | `477C750E0B44B40F478091DD002D9113AFD512935ACEC5D6310CC890C19D0585` |
| `tools/tests/test_runtime_virtual_geometry_overlay_capacity_performance_contract.py` | `A51EBA15A83534CD67819ED4AA035B70F186B5E74965D07929C6BE2D48ABA9E4` |

## Managed acceptance gate

Keep this record at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves current-source compilation, overlay output
parity, allocation behavior, and the declared Runtime overlay p50/p95/p99
gates.
