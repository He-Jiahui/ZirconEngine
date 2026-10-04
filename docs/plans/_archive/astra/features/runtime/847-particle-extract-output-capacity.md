---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/26/2026-09-20-particle-extract-output-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/scene/world/render_particles/capacity_tests.rs
tests:
  - tools/tests/test_runtime_particle_extract_output_capacity_performance_contract.py
---

# Runtime847 · particle extract output capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime particle extraction | Reserve the sorted dynamic-component owner bound for `emitters` and `bounds` while keeping unbounded sprite fanout lazy; preserve filtering, global sprite order, bounds values, and GPU frame aggregation. | TDD source/model contract `4/4`; lower reservation/empty regression and ignored `RUNTIME847_PARTICLE_EXTRACT_OUTPUT_CAPACITY_BENCH_V1` marker are wired; dense 4,096-owner model changes `11→0` growth events per bounded collector. The focused six-contract Runtime/Editor batch passes `24/24` in `0.013s`; the one-process broad non-tooling loader passes `2382/2382` across `651` modules in `5.334s`, with zero load errors/failures/errors/skips. Managed Cargo/Release, allocator, and particle product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only frame-output vector allocation shape. Runtime26's
per-emitter scratch reuse, sprite sorting, dynamic-component authority, and
GPU-frame aggregation remain unchanged; tooling production is out of scope.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/scene/world/render_particles.rs` | `E57733F5FF2AB9D00F174D99B54EBD896497C4135BDBEBF97EFBE4A3E97EE149` |
| `zircon_runtime/src/scene/world/render_particles/capacity_tests.rs` | `732D1A19580B5765783C75BA08B0FECA358E3479BFE588B79866D6A08323A8C3` |
| `tools/tests/test_runtime_particle_extract_output_capacity_performance_contract.py` | `6CEEF2361132ABA80D6E6EDA020BA7D0E6C41721A77C0FD60A87B898A1169058` |

## Managed gate

No standalone Cargo process is started locally and coordinator status is not
polled. Keep this entry `implemented_pending_validation` until the combined
owner-attributed Windows Release lane proves current-source compilation,
lower-test reachability, allocator behavior, and particle-extraction product
p50/p95/p99 evidence.
