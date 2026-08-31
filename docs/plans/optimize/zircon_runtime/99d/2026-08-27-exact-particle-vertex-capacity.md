---
title: Runtime99D Exact Particle Vertex Capacity
category: zircon_runtime
report_id: Runtime99D-exact-particle-vertex-capacity-2026-08-27
date: 2026-08-27
session_id: root-runtime99d-two-task-particle-performance-batch-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99D Exact Particle Vertex Capacity

## Scope

`build_particle_vertices` previously appended six `ParticleVertex` values per accepted sprite to
an empty `Vec`. A representative 65,536-sprite frame grew that output allocation 16 times and
copied the already-built vertex prefix on every reallocation.

The builder now defines one renderability predicate for camera layer, depth pass, size, and alpha.
It uses that same predicate to count the exact accepted sprites and to build the output. The vector
is allocated once with `accepted_sprites * 6` capacity. Sprite traversal order, depth/overlay
partitioning, layer filtering, generated vertices, colors, and empty-output behavior are unchanged.

This is a bounded current-path improvement. It does not close the parent plan's persistent instance
buffer, ring allocation, GPU upload, or indirect batching work.

## Performance Evidence

The isolated release model mirrors the production eligibility scan and six-vertex expansion with a
48-byte vertex payload. It runs 31 alternating sample pairs, 16 rounds per sample, over 65,536
sprites and 142,524 output vertices. The model was compiled with `rustc -O` on Windows.

| Metric | Growing `Vec` | Exact capacity | Change |
|---|---:|---:|---:|
| Allocator calls per build | 16 | 1 | -93.75% |
| Cumulative requested bytes per build | 18,874,080 | 6,841,152 | -63.75% |
| P50 for 16 rounds | 203,295,100 ns | 81,124,500 ns | -60.095% |
| P95 for 16 rounds | 433,281,600 ns | 352,154,700 ns | -18.724% |

Model source:
`.codex/state/session-coordinator/runtime99d-exact-particle-vertex-capacity-model.rs`.

## Contracts And Validation

- `tools/tests/test_runtime99d_exact_particle_vertex_capacity_performance_contract.py` locks the
  shared eligibility predicate, exact capacity calculation, and absence of the growing output Vec.
- The existing Rust depth/overlay behavior test now requires `capacity == len` for both passes.
- Local source-contract result: 3 tests passed.
- Local `rustfmt --edition 2021 --check` passed for the production Rust file.
- The corrected nearest-rank local preflight passed its allocation, requested-byte, and minimum
  15% P50/P95 reduction gates. These timings validate the batch parser only; terminal acceptance
  and WeCom publication must use the coordinator run.
- Cargo compilation and focused Rust behavior tests remain pending in the managed asynchronous
  coordinator batch; no direct Cargo command was run.

The managed Runtime99d batch runs the two production behavior tests under the shared
`runtime99d_batch_` filter, the three Python source contracts, and the Windows release model with
nearest-rank P50/P95 extraction. It shares one ticket with the particle identity hash-index task
and emits two exact performance rows before record acceptance.

## Remaining Parent-Plan Work

Runtime99D still owns persistent particle instance storage, frame/ring upload allocation, material
and texture consumption, GPU sorting and batching, device-generation retirement, scalability, and
product-scale CPU/GPU qualification. This slice only removes repeated CPU vertex-vector growth on
the existing billboard path.
