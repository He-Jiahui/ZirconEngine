---
title: Runtime particle extract output capacity
category: zircon_runtime
report_id: Runtime847-particle-extract-output-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime847 · particle extract output capacity

## Scope

Runtime26's frame-owned particle extraction already reuses the sprite output
and fixed GPU-bound scratch per emitter. The sorted dynamic-component owner
list also provides a strict upper bound for the frame's `emitters` and
`bounds` outputs, but both vectors previously started at zero capacity.

## Optimization

- Sort the dynamic-component owner list exactly as before, then capture its
  length as the output upper bound.
- Reserve that bound for `emitters` and `bounds` before entering the owner loop.
- Keep `sprites` lazy because authored sprite/HUD fanout is not bounded by one
  item per owner; preserve all existing extraction, filtering, sort, and GPU
  frame aggregation behavior.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the
reservations and lower module existed, then GREEN after both were wired
(`4/4`). The folder-backed lower module checks reservation order and empty
zero-capacity behavior and emits the ignored
`RUNTIME847_PARTICLE_EXTRACT_OUTPUT_CAPACITY_BENCH_V1` marker. A dense 4,096
owner model changes each zero-capacity output collector's geometric growth
from `11→0` events.

## Local validation

- `tools/tests/test_runtime_particle_extract_output_capacity_performance_contract.py`:
  `4/4`.
- Exact-file Rustfmt and Python compilation pass. The focused six-contract
  Runtime/Editor batch passes `24/24` tests in `0.013s`; the one-process broad
  non-tooling Runtime/Editor performance/pressure loader covers `651` modules
  and passes `2382/2382` tests in `5.334s`, with zero load errors, failures,
  errors, or skips. Managed Windows Cargo/Release, allocator, and particle
  product p50/p95/p99 evidence remain pending behind the external worktree
  gate.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/scene/world/render_particles.rs` | `E57733F5FF2AB9D00F174D99B54EBD896497C4135BDBEBF97EFBE4A3E97EE149` |
| `zircon_runtime/src/scene/world/render_particles/capacity_tests.rs` | `732D1A19580B5765783C75BA08B0FECA358E3479BFE588B79866D6A08323A8C3` |
| `tools/tests/test_runtime_particle_extract_output_capacity_performance_contract.py` | `6CEEF2361132ABA80D6E6EDA020BA7D0E6C41721A77C0FD60A87B898A1169058` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed managed Windows Release lane compiles the current tree,
executes the lower regression and ignored marker, and supplies allocator plus
particle-extraction product p50/p95/p99 evidence. Tooling production remains
deferred for the later Rust migration.
