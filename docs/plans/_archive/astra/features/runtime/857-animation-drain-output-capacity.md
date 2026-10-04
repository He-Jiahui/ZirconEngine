---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/level_system/2026-09-20-animation-drain-output-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/scene/level_system/animation_runtime.rs
  - zircon_runtime/src/scene/level_system/animation_runtime/drain_output_capacity_tests.rs
tests:
  - tools/tests/test_runtime_animation_drain_output_capacity_performance_contract.py
---

# Runtime857 - animation drain output capacity

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Aggregate animation event drain | Read the pending count first, keep an empty drain at zero capacity, and reserve the bounded `max_events` output for non-empty sampling across multiple batches, preserving event/byte budgets, cursor requeue, order, and empty-drain behavior. | TDD source/model contract `4/4` after a RED run with one semantic/source-shape failure; lower order/empty regression and ignored `RUNTIME857_ANIMATION_DRAIN_OUTPUT_CAPACITY_BENCH_V1` marker are wired. The 4,096-event model changes modeled growth `12→0`; the current combined Runtime/Editor focused batch passes `46/46`; managed Cargo/Release, allocator, and animation-event product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Latest local refresh

The lower owner also passes `2/2` non-ignored tests under standalone Rust test
compilation; its one performance marker remains ignored for managed Release.
The post-Runtime859/Editor858 expanded loader is `4092/4092` across `967`
files in `375.582s`; two shader-prewarm Cargo command lines printed by fixture
tests are not managed Windows Release/Cargo acceptance.

## Complexity boundary

This slice changes only the aggregate `Vec<AnimationClipEvent>` reservation in
`LevelSystem::drain_animation_clip_events`. It does not alter the bounded inner
sampler (`Runtime640`), queue admission, cursor semantics, event budget policy,
runtime ABI, or tooling production.

## Managed gate

No managed Windows Cargo/Release command is started locally and coordinator
status is not polled. Keep this entry `implemented_pending_validation` until the
combined owner-attributed Windows Release lane proves current-source
compilation, lower-test reachability, allocator behavior, and animation-event
product p50/p95/p99 evidence.
