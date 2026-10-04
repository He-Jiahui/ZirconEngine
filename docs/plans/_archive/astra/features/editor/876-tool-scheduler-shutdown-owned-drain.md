---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/125/2026-09-21-tool-scheduler-shutdown-owned-drain.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/875-tool-scheduler-promotable-request-projection.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/tools/scheduler.rs
tests:
  - zircon_editor/src/core/tools/optimization_batch_editor876_tool_scheduler_shutdown_owned_drain_tests.rs
  - tools/tests/test_editor876_tool_scheduler_shutdown_owned_drain_performance_contract.py
---

# Editor876 Tool Scheduler Shutdown Owned Drain

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor125 scheduler shutdown drain | Project ordered request positions before taking ownership of request/lease maps, then publish directly from their owned BTree iterators. This removes queued-handle clones and both complete-handle scratch vectors while preserving event phases, ID order, positions, counts, and empty terminal state. | Intentional RED `1/6` → GREEN `6/6`; lower two-waiter position regression and ignored `EDITOR876_TOOL_SCHEDULER_SHUTDOWN_OWNED_DRAIN_BENCH_V1` marker are wired. The 4,096-request/4,096-lease model changes request-handle clones `4096→0` and handle scratch slots `8192→0`; the related scheduler batch passes `17/17`. Managed Cargo/Release, allocator, and product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Scope boundary

This slice changes only shutdown projection/ownership. It does not change queue
limits, claim admission, promotion, resource arbitration, capture policy,
lifecycle event schema, extension retirement, or tooling production.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/tools/scheduler.rs` | `2A57A3CDEC261C654C257B77579A342DD0A6E90E7D0CF8DEDCC6F1DECA72FDDC` |
| `zircon_editor/src/core/tools/optimization_batch_editor876_tool_scheduler_shutdown_owned_drain_tests.rs` | `D36ED30CEB78B382F39927898E0964449C09588B56AC49701F5331AEDCA0456C` |
| `tools/tests/test_editor876_tool_scheduler_shutdown_owned_drain_performance_contract.py` | `F92A07867729725AF5C2337FC2272011936DD58C1055C775DED142E65201BCEC` |

## Managed gate

Editor875–878 were submitted together in asynchronous v8 and were not submitted
individually. Keep this entry pending until that current-source lane
provides Editor compilation, lower/ignored Release execution, allocator
evidence, and interactive-tool shutdown percentiles.
