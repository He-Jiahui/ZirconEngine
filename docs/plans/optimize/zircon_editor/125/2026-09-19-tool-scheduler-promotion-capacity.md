---
title: Editor125 Tool Scheduler Promotion Capacity
category: zircon_editor
report_id: Editor125-tool-scheduler-promotion-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260915
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor125 · Tool scheduler promotion capacity

## Scope

The interactive-tool scheduler promotes queued multi-resource and single-resource
claims after a lease is released or withdrawn. Both promotion helpers returned a
fresh zero-capacity vector even though their queue/resource maps provide a bounded
upper limit. This slice reserves those bounds while preserving FIFO promotion,
resource arbitration, lifecycle-event order, and the no-promotion empty result.

## Implementation

- `promote_available_sets` lazily reserves `set_queue.len()` only when the first
  set claim is actually promoted.
- `promote_waiting_singles` lazily reserves `resources.len()` only when the first
  single-resource claim is actually promoted, so an empty/no-op release stays
  allocation-free.
- No claim state, request position, event payload, or owner-generation semantics
  changed; the capacity hints only remove geometric growth on promotion bursts.
- The lower regression covers a blocked set plus blocked single waiter and proves
  the set is promoted first, followed by the single waiter after the set releases.
  The ignored benchmark emits `EDITOR827_TOOL_SCHEDULER_PROMOTION_BENCH_V1`.

## Local evidence

- TDD source contract is green (`3/3`).
- The lower Rust module is wired under `scheduler.rs` and Rustfmt-clean.
- The merged non-tooling Runtime/Editor source-contract batch loads `622` modules
  and passes `2221/2221` tests with zero failures, errors, or skips in one
  process; its receipt is recorded in the async admission log and aggregate
  completion list.
- Managed Windows Cargo/Release and product p50/p95/p99 evidence remain pending
  behind the external dirty `E:\Git\zr_vm` gate. Tooling production changes remain
  deferred for the later Rust migration.

The latest shared non-tooling Runtime/Editor batch supersedes the earlier local
receipt: one process now loads `624` modules and passes `2227/2227` tests with
zero failures, errors, or skips after Editor828; the six-slice focused loader
passes `21/21`. Managed Cargo/Release and product percentile evidence remain
pending.

The 2026-09-21 Editor875 follow-up preserves this activated-output reservation
while replacing the complete resource-key snapshot with an ordered projection
of promotable request IDs. The related scheduler source-contract batch passes
`20/20`; managed lower/Release/allocator/product gates remain pending.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/tools/scheduler.rs` | `2A57A3CDEC261C654C257B77579A342DD0A6E90E7D0CF8DEDCC6F1DECA72FDDC` (shared current-worktree hash after Editor876) |
| `zircon_editor/src/core/tools/optimization_batch_editor827_tool_scheduler_promotion_tests.rs` | `A4D0129588A7A31B3731B1B559429ACFE0B171BED319700F6539FBFF3917D99A` |
| `tools/tests/test_editor_tool_scheduler_promotion_capacity_performance_contract.py` | `2F7FA4AFD95EF8AE8C725606C14BB34018D4D2C8CECE31283F4834421247C6D7` |
