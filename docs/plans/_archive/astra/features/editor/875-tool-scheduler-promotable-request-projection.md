---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/125/2026-09-21-tool-scheduler-promotable-request-projection.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/876-tool-scheduler-shutdown-owned-drain.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/tools/scheduler.rs
tests:
  - zircon_editor/src/core/tools/optimization_batch_editor875_tool_scheduler_promotable_request_projection_tests.rs
  - tools/tests/test_editor875_tool_scheduler_promotable_request_projection_performance_contract.py
---

# Editor875 Tool Scheduler Promotable Request Projection

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor125 single-request promotion projection | Scan resources immutably and collect only promotable request IDs before mutation, eliminating the complete resource-key clone snapshot while preserving set priority, BTree resource order, stale guards, activation, and event ordering. | Intentional RED `4/5` → GREEN `5/5`; lower reverse-queue/canonical-resource-order regression and ignored `EDITOR875_TOOL_SCHEDULER_PROMOTABLE_REQUEST_PROJECTION_BENCH_V1` marker are wired. The 4,096-resource no-candidate model changes resource-key clones `4096→0` and projection capacity `4096→0`; the related scheduler batch passes `20/20`. Managed Cargo/Release, allocator, and interactive-tool product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Scope boundary

This slice changes only the temporary promotion projection. It does not change
queue limits, claim identity, set-head priority, resource ownership, input
capture, lifecycle events, extension retirement, or tooling production.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/tools/scheduler.rs` | `2A57A3CDEC261C654C257B77579A342DD0A6E90E7D0CF8DEDCC6F1DECA72FDDC` (shared current-worktree hash after Editor876) |
| `zircon_editor/src/core/tools/optimization_batch_editor875_tool_scheduler_promotable_request_projection_tests.rs` | `31C3B9A8574CEF628F1A53291BDCB0315C97BFCC92CDD4A430BC811C7AD01E8C` |
| `tools/tests/test_editor875_tool_scheduler_promotable_request_projection_performance_contract.py` | `CFB5DB120653427891C1316467DA6984A267DCD7C2E38979B201B64217BF5172` |

## Managed gate

Editor875 was submitted with Editor876–878 in asynchronous v8 and was not
submitted alone. Keep this entry pending until that current-source lane provides Editor
compilation, lower/ignored Release execution, allocator evidence, and
interactive-tool product percentiles.
