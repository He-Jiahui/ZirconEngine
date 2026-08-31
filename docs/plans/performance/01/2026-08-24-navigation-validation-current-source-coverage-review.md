---
title: Navigation Validation Current-Source Coverage Review
date: 2026-08-24
scope:
  - zircon_plugins/navigation/editor/src/tests.rs
  - zircon_plugins/navigation/editor/src/tests
  - zircon_plugins/navigation/native/src/tests
  - zircon_plugins/navigation/native/tests
  - zircon_plugins/navigation/runtime/src/manager/traversal/tests.rs
  - zircon_plugins/navigation/runtime/src/tests
status: static_complete_execution_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
---

# Navigation Validation Current-Source Coverage Review

## 1. Coverage

Post-M0 validation scope static review is **29/29 Rust files**, **5,811 physical / 5,303 non-empty lines**, **192,046 bytes** and **137 tests**. Ordered fingerprint: `360f4489f5d203e34f79b07bb9ff6f56d6cb00803214da812bc870cb4d2f75a0`.

Together with the three production records, Navigation is **104/104 Rust files**, **16,245 physical / 14,965 non-empty lines**, **548,059 bytes** and **147 tests**. Composite ordered fingerprint: `b4dd30c3ddbd50f037595ba9b7ab9feef890611dd909e3901b89e2629ff79d7f`.

| Validation folder | Files | Static result |
|---|---:|---|
| `editor/src/tests*` | 4 | Registration, retained selection, mock operations and overlay behavior reviewed. |
| `native/src/tests` and `native/tests` | 11 | Bake/query/crowd/TileCache ABI behavior and helpers reviewed. |
| `runtime/src/manager/traversal/tests.rs` | 1 | Transform/event ordering reviewed. |
| `runtime/src/tests` | 13 | Bake, crowd, query, obstacle, operation, overlay, registration and task-currentness contracts reviewed. |

## 2. Coverage gaps that block performance acceptance

1. Editor operation mocks complete during `submit_operation`; no test covers a genuinely pending Bake across frames, cancellation, close/session loss, queue delay or UI-thread wait.
2. `tiled_bake_does_not_block_main_thread` only checks that submission returns within 50 ms. It does not measure cloned World size, task count, worker CPU, queue growth, cancellation or owner-thread harvest cost. Wait helpers busy-yield for up to five seconds.
3. Dirty-bake tests check output tile identities, but do not assert input triangles, full-World scans, copied bytes or preserved-asset reconstruction; the current full prepare can pass them.
4. Native correctness fixtures contain a few flat polygons. They do not cover large tiled/layered worlds, >512 corridors, area IDs 16-63 flag separation, partial paths, vertical TileCache layers or native-failure/fallback parity.
5. Crowd behavior tops out at 20 agents. The only 4,096-capacity state-read performance test is ignored and compares one active agent; it does not measure full crowd tick or result-vector allocation.
6. Overlay performance tests are ignored and only prove subscriber-off versus unconditional construction and Arc snapshot versus deep clone. They do not qualify the enabled default path, serialization, editor mirror lock, line generation, upload or power.
7. No test asserts stable-frame zero scans/JSON parses/query compiles, world/session generation isolation, unload, cancellation/join, scheduler budgets or movement-authority ownership.
8. `runtime/src/tests/operation.rs` expects a Bake operation to succeed, while the current runtime handler rejects Bake preparation because it lacks a pure prepare backend. This source-level contract conflict requires managed test execution and one canonical job design; it must not be normalized by weakening the test.

## 3. Required validation matrix

Add deterministic structural counters first: scanned nodes/components, parsed DTOs, admitted chunks/triangles, query compiles, native queries, dirty/full input triangles, tile jobs, cloned bytes, cancelled/joined jobs, overlay serialized bytes/primitives and transform/desired-velocity writes.

Then add release benchmarks and current-source executable scenarios for polygon/triangle/agent/obstacle/tile scale. Report raw samples plus p50/p95/p99 and assert complexity/budget relationships, not one wall-clock threshold. Include native-disabled/failed modes and verify typed failure rather than fallback equivalence.

## 4. Execution status

- Static validation-file review: **29/29 complete**.
- M0 behavior coverage: one test added for all-visible, mesh-only, link-only and hidden overlay categories; targeted formatting and diff checks passed.
- Test execution: **pending**; the managed Windows Cargo validation session is unavailable and raw Cargo was not used.
- Current-source WPR/ETW/RenderDoc/power: **pending**; no launchable current-source executable exists.
- This record does not promote protected `review.md`/`pending.md`, create a milestone commit or send WeCom completion.
