---
title: Editor06 Swap-remove Failed Lifecycle Stage
category: zircon_editor
report_id: Editor06-swap-remove-failed-lifecycle-stage-2026-08-26
date: 2026-08-26
session_id: root-editor06-plugin-admission-borrows-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Swap-remove Failed Lifecycle Stage

## Scope

Successful lifecycle callbacks remove their stage from a private failed-stage membership list.
The list is queried for membership only; no public contract observes its order. The removal can
therefore avoid the full-vector compaction performed by `retain`.

## Implementation

`remove_failed_lifecycle_stage` finds the matching stage once and removes it with `swap_remove`.
Repeated success of an already-cleared stage is a no-op, and failed-stage membership remains
unchanged for every other stage. The regression covers both removal and repeated no-op behavior.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Failed-stage entries moved when removing the first of 10 | 9 | 1 |
| Failed-stage membership | exact | unchanged |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `EDITOR06_FAILED_LIFECYCLE_STAGE_SWAP_REMOVE_BENCH_V1` with both p95
durations, sample/iteration/stage counts, and the deterministic move reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and functional membership tests are prepared. The
release benchmark is batched with the event-consumer sort benchmark; commit integration, terminal
p95 values, and WeCom delivery remain coordinator-owned.
