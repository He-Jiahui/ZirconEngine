---
title: Runtime857 animation drain output capacity
category: zircon_runtime
report_id: Runtime857-animation-drain-output-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime857 - animation drain output capacity

## Scope

`LevelSystem::drain_animation_clip_events` enforces a bounded
`AnimationClipEventSamplingLimits::max_events` budget, but its aggregate event
vector previously started at zero capacity. A frame with several pending sample
batches therefore paid geometric growth before reaching the already-known
bound.

## Optimization

- Read the pending sample count first, keep the no-pending path at zero capacity,
  and reserve `limits.max_events` for non-empty aggregate drains before sampling.
- Preserve the remaining-event budget, event-byte budget, batch order, cursor
  requeue behavior, unavailable-asset handling, and empty-drain semantics.
- Keep the bounded sampler's existing `Runtime640` inner-batch reservation
  outside this slice; this change covers only the cross-batch drain collector.

## TDD and deterministic evidence

The Python source/model contract was run RED before the reservation and lower
owner existed (two structural/wiring failures), then GREEN at `4/4`. The lower
Rust owner checks ordered aggregation and empty batches and carries the ignored
`RUNTIME857_ANIMATION_DRAIN_OUTPUT_CAPACITY_BENCH_V1` marker. A 4,096-event
bounded model changes the aggregate collector's modeled geometric growth events
from `12` to `0`.

## Local validation

- Standalone `rustc --edition 2021 --test` execution of the lower owner passes
  `2/2` non-ignored tests; the single benchmark marker remains intentionally
  ignored for the managed Windows Release lane.
- `tools/tests/test_runtime_animation_drain_output_capacity_performance_contract.py`:
  `4/4`.
- A follow-up RED assertion caught the empty-drain reservation mismatch; after
  moving the pending-count read ahead of allocation, the contract is GREEN and
  the lower empty model explicitly asserts `capacity() == 0`.
- The combined Runtime/Editor focused batch covering the preceding V2/render/
  material slices plus Runtime857, Runtime858, and Editor857 passes `38/38`
  tests in `0.021s`, with zero failures, errors, or skips.
- After Runtime859 and Editor858 joined the batch, the current combined focused
  invocation passes `46/46` tests in `0.020s`, with zero failures, errors, or
  skips; the lower Runtime857 owner was rerun after the lazy empty-path repair.
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
  Allocator and animation-event product p50/p95/p99 evidence remain pending
  behind the shared external worktree gate; isolated contract fixtures are not
  a managed acceptance result.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/scene/level_system/animation_runtime.rs` | `45088DF65F39E5AEC4F4721E2E4D4B8B5380E0F9AA22DCFC2C030510F42A7948` |
| `zircon_runtime/src/scene/level_system/animation_runtime/drain_output_capacity_tests.rs` | `3BAB65A6A73CE115CF56B9D3EB507CA600B6C56152F3C916FB3C8759707F8BA1` |
| `tools/tests/test_runtime_animation_drain_output_capacity_performance_contract.py` | `A0BC35DADEC363391604D485486CF240BA307BE7CA9030069AC8B4E87B5A7944` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Runtime tree,
executes the lower regression and ignored marker, and supplies allocator plus
animation-event product p50/p95/p99 measurements. Tooling production remains
deferred for the later Rust migration; coordinator status is not polled here.
