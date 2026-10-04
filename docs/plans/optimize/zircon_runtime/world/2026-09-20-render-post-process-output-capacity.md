---
title: Runtime858 render post-process output capacity
category: zircon_runtime
report_id: Runtime858-render-post-process-output-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime858 - render post-process output capacity

## Scope

`World::collect_post_process_volumes` scans the registered
`PostProcessVolumeComponent` table and emits two independently filtered output
vectors: post-process extracts and local fog volumes. The component count is
already available before the scan, but both vectors previously grew from zero
capacity on every view collection.

## Optimization

- Read the registered component count once and reserve that count for both
  output vectors.
- Use a zero bound when the component is not registered, preserving the
  no-component no-allocation path.
- Preserve layer filtering, global/local shape handling, priority/entity sort
  order, tuple-to-output projection, and empty-world behavior.

## TDD and deterministic evidence

The Python source/model contract was run RED before the reservation and lower
owner existed (one structural failure and one wiring failure), then GREEN at
`4/4`. The lower Rust owner checks output order and empty input and carries the
ignored `RUNTIME858_RENDER_POST_PROCESS_OUTPUT_CAPACITY_BENCH_V1` marker. A
4,096-component model changes both output collectors' modeled geometric growth
events from `24` to `0`.

## Local validation

- Standalone `rustc --edition 2021 --test` execution of the lower owner passes
  `2/2` non-ignored tests; the single benchmark marker remains intentionally
  ignored for the managed Windows Release lane.
- `tools/tests/test_runtime_render_post_process_output_capacity_performance_contract.py`:
  `4/4`.
- The combined Runtime/Editor focused batch covering the preceding V2/render/
  material slices plus Runtime857, Runtime858, and Editor857 passes `38/38`
  tests in `0.021s`, with zero failures, errors, or skips.
- After Runtime859 and Editor858 joined the batch, the current combined focused
  invocation passes `46/46` tests in `0.020s`, with zero failures, errors, or
  skips.
- Exact-file `rustfmt --edition 2021 --check` passes for the production and
  lower Rust owners; `python -m py_compile` passes for all three new contracts.
- The expanded non-tooling source-contract loader covers `965` files and passes
  `4084/4084` tests in `137.924s` under the explicit performance-or-contract
  filename filter (tooling, export, and coordinator files excluded), with zero
  load errors, failures, errors, or skips.
- The post-Runtime859/Editor858 expanded receipt covers `967` files and passes
  `4092/4092` tests in `375.582s`; two shader-prewarm Cargo command lines were
  printed by fixture tests and are not managed Windows Release/Cargo acceptance.
- No managed Windows Cargo/Release validation command was started locally.
  Allocator and render post-process product p50/p95/p99 evidence remain pending
  behind the shared external worktree gate; isolated contract fixtures are not a
  managed acceptance result.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/scene/world/render_post_process.rs` | `781BBEFA7071F2CA4E204C5C7C5953ED831FCA444F80D11F389A95457EF3DD3B` |
| `zircon_runtime/src/scene/world/render_post_process/output_capacity_tests.rs` | `C37EA92E87B17D25BA227370F72DFB38B000E0BFB5EC2A77EE7EE7CA4DA59335` |
| `tools/tests/test_runtime_render_post_process_output_capacity_performance_contract.py` | `3B238E69E9EA9655115E4EE2C5BB836565301BA3F58C51A0859D2146EE946E5F` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Runtime tree,
executes the lower regression and ignored marker, and supplies allocator plus
render post-process product p50/p95/p99 measurements. Tooling production remains
deferred for the later Rust migration; coordinator status is not polled here.
