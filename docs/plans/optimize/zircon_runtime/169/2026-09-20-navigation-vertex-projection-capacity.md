---
title: Runtime navigation polygon vertex projection capacity
category: zircon_runtime
report_id: Runtime862-navigation-vertex-projection-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime862 Navigation Polygon Vertex Projection Capacity

## Finding

`BakedPolygon::from_asset` projected the validated polygon index slice through
`filter_map(...).collect::<Vec<_>>()`. Because invalid vertex indices make the
iterator's lower bound zero, the vertex output started at zero capacity even
when the bounded index slice was dense and valid.

## Optimization

The projection now uses `polygon_vertices`, which reserves the index-slice
length once and extends the filtered, ordered vertex stream into that buffer.
Invalid indices are still ignored, duplicate vertices remain available for the
existing sort/dedup pass, and bounds/center/edge-key semantics are unchanged.
The reservation is conservative when indices are invalid but never changes
the projected result.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the helper
and lower owner existed, then GREEN at `4/4`. Lower tests cover retained
capacity for a dense index slice and invalid-index filtering/order; the ignored
managed marker is
`RUNTIME862_NAVIGATION_VERTEX_PROJECTION_CAPACITY_BENCH_V1`. Its simple
4,096-index model changes the zero-capacity collector from `11` modeled growth
events to `0` reserved growth events.

## Local validation boundary

- Exact-file Rustfmt and Python compilation pass for the changed Rust and
  source-contract files.
- The combined navigation source/model batch covering Runtime861, Runtime862,
  navigation projection/dispatch ownership, tree focus, and UI navigation
  index passes `35/35` tests in `0.062s`, with zero failures, errors, or skips.
- The existing Runtime08d borrowed-index contract was updated to recognize the
  extracted `polygon_vertices` owner while still asserting a borrowed
  `index_set` and no index copy. The expanded seven-contract navigation batch
  passes `38/38` tests in `0.139s`, with zero failures, errors, or skips.
- The refreshed one-process explicit performance-or-contract loader covers
  `970` non-tooling files and passes `4104/4104` tests in `228.769s`, with zero
  load errors, failures, errors, or skips. Shader-prewarm Cargo command lines
  printed by fixtures are not managed Windows Release/Cargo acceptance.
- Local source/model receipts do not establish Rust compilation, allocator
  behavior, or product navigation latency percentiles.
- Tooling production remains deferred for the later Rust migration.

On 2026-09-21, the same one-process non-tooling loader was rerun after the
record-integrity and document-structure audits and again passed `4104/4104`
across `970` files in `274.515s`, with zero failures, errors, load errors, or
skips. Fixture-emitted Cargo command lines remain non-acceptance output.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/navigation/runtime/baked_mesh.rs` | `0ECB4B874765D5CE51C88283126A54A497B81D63D423DBD5213F6D680493E558` |
| `tools/tests/test_runtime862_navigation_vertex_projection_capacity_performance_contract.py` | `175115A2729E03E86026ADF90844EA71B2561AF4F1ED6748EF70BBE8CCDC3D16` |
| `tools/tests/test_runtime08d_borrowed_polygon_indices_performance_contract.py` | `2D76031EF2A617942501FFF2B45C1512897AEE4E7E4F80C40D4E8F7A9F3490EC` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed managed Windows Release lane compiles the current tree,
executes the lower regression and ignored marker, and supplies allocator plus
navigation product p50/p95/p99 evidence. Do not infer product acceptance from
the local source/model receipts.
