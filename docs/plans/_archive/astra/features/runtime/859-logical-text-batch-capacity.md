---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11c/2026-09-20-logical-text-batch-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout/logical_batch_capacity_tests.rs
tests:
  - tools/tests/test_runtime_logical_text_batch_capacity_performance_contract.py
---

# Runtime859 - logical text batch capacity

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Screen-space logical text batch projection | Reserve the resolved layout line bound before artifact and visual-fallback batch emission. | TDD source/model contract `4/4`; lower order/empty regression and ignored `RUNTIME859_LOGICAL_TEXT_BATCH_CAPACITY_BENCH_V1` marker are wired. A 4,096-line model changes modeled growth `12→0`; focused Runtime/Editor batch is `46/46`; managed Cargo/Release, allocator, and text-render product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Latest local refresh

The lower owner passes `2/2` non-ignored tests under standalone Rust test
compilation; its one performance marker remains ignored for managed Release.
The current five-owner lower batch passes `10/10` non-ignored tests, and the
post-Runtime859/Editor858 expanded source-contract receipt covers `967` files
and passes `4092/4092` tests in `375.582s`; fixture Cargo command lines are not
managed Windows Release/Cargo acceptance. The receipt is also recorded in the
linked optimize record and aggregate ledgers.

## Complexity boundary

This slice changes only the `logical_text_batches` output reservation. It does
not alter glyph artifact ownership, stale/missing/incomplete rejection, source
isomorphic fallback, line order, text payloads, or tooling production.

## Managed gate

No managed Windows Cargo/Release command is started locally and coordinator
status is not polled. Keep this entry `implemented_pending_validation` until the
combined owner-attributed Windows Release lane proves current-source
compilation, lower-test reachability, allocator behavior, and text-render
product p50/p95/p99 evidence.
