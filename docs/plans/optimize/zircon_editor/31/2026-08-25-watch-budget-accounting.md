---
title: Editor31 Script Watch Budget Accounting
category: zircon_editor
report_id: Editor31-script-watch-budget-accounting-2026-08-25
date: 2026-08-25
session_id: root-editor31-watch-budget-accounting-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor31 Script Watch Budget Accounting

## Scope

This slice advances Editor31 `G07` for the existing script-build orchestrator. It optimizes pending
watch-path admission and queued-request path merging without changing watcher ownership, debounce
deadlines, trigger promotion, the 20-path/64-KiB limits, or the empty-path full-rebuild sentinel.

## Reference Evidence

- Bevy's `dev/bevy/crates/bevy_asset/src/io/file/file_watcher.rs` debounces filesystem events and
  suppresses repeated adjacent asset events before publishing them.
- Bevy's `dev/bevy/crates/bevy_asset/src/processor/tests.rs` exercises changed-asset notification
  through the real processing boundary.
- Fyrox's `dev/Fyrox/fyrox-core/src/watcher.rs` keeps filesystem receipt non-blocking through
  `try_recv`; it does not make compilation or resource reload part of the watcher itself.

Zircon deliberately retains its stricter bounded coalescing and full-rebuild fallback. The change
only removes repeated accounting work inside that existing contract.

## Implementation

`ScriptBuildOrchestrator` now maintains the byte total beside its deduplicated pending path set.
Only a newly inserted path updates the total, so duplicate watcher events no longer traverse every
retained path to recompute the same budget. Overflow, count excess, or byte excess still clears the
incremental set and requests a full rebuild; dispatch and failed-build cleanup reset the tracked
total with the set.

Queued-request merging reuses the same checked insertion rule and accounts bytes while building the
deduplicated ordered set. It no longer performs a second full traversal after the merge.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 1,000,000 duplicate watcher events | 1,000,000 retained-path budget visits | 1 byte-accounting update | 99.9999% |
| 15-path deduplicated queued merge | 15 post-build budget visits | 0 post-build visits | 100% |
| Release evidence wall time | not gated | <= 3 s | pending terminal evidence |

The ignored Windows-native release test emits
`EDITOR31_WATCH_BUDGET_ACCOUNTING_BENCH_V1` with event count, unique paths, legacy visits,
optimized updates, reduction parts per million, queued-merge visits, elapsed nanoseconds, and the
elapsed-time ceiling. Exact timing is accepted only from coordinator terminal evidence.

## Validation

- Behavior coverage includes duplicate accounting, full-rebuild reset, sorted/deduplicated queued
  merging, existing debounce/count/byte/full-rebuild behavior, and the ignored release marker.
- Exact-file Rustfmt and scoped diff checks pass locally; no local Cargo lane is launched.
- Managed test results, terminal marker values, integration commit, and automatic WeCom delivery
  remain pending.

## Remaining Parent-plan Work

This slice does not wire a production watcher, shared job executor, compiler adapter, artifact
receipt, runtime install generation, Play waiter, commandlet, LSP, debugger, or script authoring UI.
Editor31 remains partial until those product layers and its complete performance gates converge.
