---
title: Runtime859 logical text batch capacity
category: zircon_runtime
report_id: Runtime859-logical-text-batch-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime859 - logical text batch capacity

## Scope

`logical_text_batches` validates a resolved text layout and then emits one
`ResolvedLayoutTextBatch` per authored layout line across artifact and visual
fallback routes. The line count is already available before either loop, but the
batch output previously started at zero capacity for every screen-space text
command.

## Optimization

- Reserve `layout.lines.len()` before artifact and visual-fallback projection.
- Keep the stale/incomplete/missing artifact rejection branches unchanged.
- Preserve line order, source ranges, glyph-artifact ownership, fallback route,
  and empty-layout zero-capacity behavior.

## TDD and deterministic evidence

The Python source/model contract was run RED before the reservation and lower
owner existed (structural and wiring failures), then GREEN at `4/4`. The lower
Rust owner checks output order and empty layouts and carries the ignored
`RUNTIME859_LOGICAL_TEXT_BATCH_CAPACITY_BENCH_V1` marker. A 4,096-line model
changes modeled geometric growth from `12` to `0`.

## Local validation

- Standalone `rustc --edition 2021 --test` execution of the lower owner passes
  `2/2` non-ignored tests; its single benchmark marker remains intentionally
  ignored for the managed Windows Release lane.
- `tools/tests/test_runtime_logical_text_batch_capacity_performance_contract.py`:
  `4/4`.
- The combined focused Runtime/Editor source-contract batch, including the
  preceding V2/render/material slices plus Runtime857/858, Editor857, Runtime859,
  and Editor858, passes `46/46` tests with zero failures, errors, or skips.
- Exact-file `rustfmt --edition 2021 --check` passes for the production and
  lower Rust owners; `python -m py_compile` passes for both new contracts.
- The refreshed expanded non-tooling source-contract loader covers `967` files
  and passes `4092/4092` tests in `375.582s` under the explicit
  performance-or-contract filename filter (tooling, export, and coordinator
  files excluded), with zero load errors, failures, errors, or skips. The run
  also printed two existing shader-prewarm fixture Cargo command lines inside
  tests; it was not a managed Windows Release/Cargo acceptance run.
- No managed Windows Cargo/Release validation command was started locally.
  Allocator and text-render product p50/p95/p99 evidence remain pending behind
  the shared external worktree gate; isolated contract fixtures are not managed
  acceptance evidence.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout.rs` | `1E760097678AF6F42E74C0C381E790F14F6834B114A2B60FB9892502FF332E7F` |
| `zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout/logical_batch_capacity_tests.rs` | `0FB015AA6C6709169DAB8C657F593B57292EEC723FDC7A928958B11BC95C2C13` |
| `tools/tests/test_runtime_logical_text_batch_capacity_performance_contract.py` | `39BE41EE933A34A01B3A2A36C001EF44DB3F164CF8C1DCF1DD4D17AF7C633441` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Runtime tree,
executes the lower regression and ignored marker, and supplies allocator plus
text-render product p50/p95/p99 measurements. Tooling production remains
deferred for the later Rust migration; coordinator status is not polled here.
