---
title: Editor Tool Scheduler Shutdown Owned Drain
category: zircon_editor
report_id: Editor876-tool-scheduler-shutdown-owned-drain-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor876 Tool Scheduler Shutdown Owned Drain

## Finding

`ToolScheduler::shutdown` cloned every queued `ToolRequestHandle` into a
temporary `(handle, position)` vector and materialized every owned lease into a
second handle vector before clearing their authoritative maps. Shutdown already
owns `&mut self`, so the clone and both full-handle scratch collections were not
required to publish the same terminal events.

## Optimization

- Project only each queued request's `usize` position while the queue indexes
  are still available.
- Record request/lease counts for exact event capacity, then take ownership of
  both ordered maps.
- Iterate `BTreeMap::into_values()` directly, zipping owned requests with the
  ordered position projection.
- Preserve capture events first, lease deactivation next, request withdrawal
  last, plus request-ID/lease-ID order, previous positions, outcome counts, and
  the fully cleared scheduler state.

## TDD and deterministic evidence

The Editor876 source/model contract was observed RED with five failures (the
deterministic model was already true), then GREEN at `6/6`. The lower regression
queues two requests behind one holder and verifies ordered withdrawal positions
`1, 2` after an owned shutdown drain.

For 4,096 queued requests and 4,096 active leases, the retired path performs
4,096 `ToolRequestHandle` clones and stores 8,192 complete handles across its
two scratch vectors. The new path performs zero request-handle clones and stores
zero handles in scratch collections; its only queued-request projection stores
4,096 `usize` values. The ignored 101-pair Release marker
`EDITOR876_TOOL_SCHEDULER_SHUTDOWN_OWNED_DRAIN_BENCH_V1` emits alternating
p50/p95/p99 samples and requires improved model p95.

## Local validation boundary

- Exact-file Rustfmt passes for the scheduler and lower regression.
- Editor827/828/875/876 scheduler source/model contracts pass `17/17` in one
  batch, with zero failures or errors.
- Editor875–878 were submitted together in the asynchronous v8 multi-task
  current-source batch; no per-task lane was started.
- Local source/model evidence does not establish Windows compilation, actual
  allocator behavior, or interactive-tool shutdown p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/tools/scheduler.rs` | `2A57A3CDEC261C654C257B77579A342DD0A6E90E7D0CF8DEDCC6F1DECA72FDDC` |
| `zircon_editor/src/core/tools/optimization_batch_editor876_tool_scheduler_shutdown_owned_drain_tests.rs` | `D36ED30CEB78B382F39927898E0964449C09588B56AC49701F5331AEDCA0456C` |
| `tools/tests/test_editor876_tool_scheduler_shutdown_owned_drain_performance_contract.py` | `F92A07867729725AF5C2337FC2272011936DD58C1055C775DED142E65201BCEC` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus interactive-
tool shutdown p50/p95/p99 evidence. The clone model is not product acceptance.
