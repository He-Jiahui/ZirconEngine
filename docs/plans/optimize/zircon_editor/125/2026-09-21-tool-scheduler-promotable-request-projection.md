---
title: Editor Tool Scheduler Promotable Request Projection
category: zircon_editor
report_id: Editor875-tool-scheduler-promotable-request-projection-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor875 Tool Scheduler Promotable Request Projection

## Finding

`ToolScheduler::promote_waiting_singles` cloned every registered
`ToolResourceKey` into a temporary vector before discovering whether any single
request could be promoted. Held resources, resources blocked by the set-queue
head, empty queues, and queues containing only set requests all paid that clone
and allocation cost. The mutation phase only needs each selected request ID.

## Optimization

- Scan the ordered resource map immutably and retain only the first promotable
  single-request ID for each eligible resource.
- Lazily reserve the request-ID projection on its first match, preserving a
  zero-capacity no-candidate path.
- Perform request detach, activation, and lifecycle publication in a second
  phase over those IDs, keeping BTree resource order and stale-request guards.
- Preserve set-head priority, one-single-per-resource promotion, activated
  lease/event order, resource arbitration, and the Editor827 activated-output
  capacity behavior.

## TDD and deterministic evidence

The Editor875 source/model contract was observed RED with four failures (the
fifth deterministic model check was already true), then GREEN at `5/5`. A lower
regression releases a two-resource holder with waiters queued in reverse order
and verifies promotion follows canonical resource order.

For 4,096 registered resources with no promotable single request, the retired
path clones 4,096 resource keys and allocates a 4,096-slot key vector. The new
path clones zero resource keys and leaves the request-ID projection at zero
capacity. Dense promotion still clones zero resource keys and stores only
copyable request IDs. The ignored 101-pair Release marker
`EDITOR875_TOOL_SCHEDULER_PROMOTABLE_REQUEST_PROJECTION_BENCH_V1` reports
alternating p50/p95/p99 samples and requires improved no-candidate p95.

## Local validation boundary

- Exact-file Rustfmt passes for the scheduler and lower regression.
- Editor875 plus the existing Editor08/827/828 scheduler contracts passes
  `20/20` in `0.011s`, with zero failures or errors.
- Editor875 was submitted with Editor876–878 in the asynchronous v8 multi-task
  current-source batch; no per-task lane was started.
- Local source/model evidence does not establish Windows compilation, actual
  allocator behavior, or interactive-tool product p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/tools/scheduler.rs` | `2A57A3CDEC261C654C257B77579A342DD0A6E90E7D0CF8DEDCC6F1DECA72FDDC` (shared current-worktree hash after Editor876; the Editor875 promotion remains covered by its unchanged focused files and `17/17` adjacent scheduler batch) |
| `zircon_editor/src/core/tools/optimization_batch_editor875_tool_scheduler_promotable_request_projection_tests.rs` | `31C3B9A8574CEF628F1A53291BDCB0315C97BFCC92CDD4A430BC811C7AD01E8C` |
| `tools/tests/test_editor875_tool_scheduler_promotable_request_projection_performance_contract.py` | `CFB5DB120653427891C1316467DA6984A267DCD7C2E38979B201B64217BF5172` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus interactive-
tool p50/p95/p99 evidence. The clone model is not product acceptance.
