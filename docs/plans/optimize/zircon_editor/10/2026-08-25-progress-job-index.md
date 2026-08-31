---
title: Editor10 Progress Job Binding Index
category: zircon_editor
report_id: Editor10-progress-job-index-2026-08-25
date: 2026-08-25
session_id: root-editor10-notification-projection-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor10 Progress Job Binding Index

## Scope

This slice removes the bounded full-map scan used to detect and retire progress notifications by
`JobId`. It preserves the existing one-notification-per-job contract and automatic-to-manual
producer replacement policy. It does not claim the parent plan's immutable generation snapshots,
typed deltas, windowed projection, durable history, or progress aggregation milestones.

## Implementation

The progress center now owns one mutex-protected state containing the authoritative
`NotificationId -> ProgressNotification` map and a `JobId -> NotificationId` index. Publish,
automatic binding replacement, terminal retirement, full synchronization, and captured
synchronization update both maps under the same lock.

`publish` uses the job index instead of scanning all live notification values. `retire_job` removes
the indexed notification directly instead of retaining over the full map. Captured synchronization
still preserves a notification ID rebound to a new job after capture. At the configured maximum,
the additional bounded metadata is 64 `JobId`/shared-notification-ID index entries.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 100,000 duplicate-job lookups with 64 live entries | 6,400,000 candidate checks | 100,000 logical job-index probes; <= 3 s | 98.4375% probe reduction |
| Retire one job at capacity | scan up to 64 notifications | one job-index removal plus one exact identity removal | O(n) retain replaced by O(log n) lookups |
| Automatic-to-manual replacement | scan values for the bound job | indexed owner lookup | replacement semantics unchanged |

The ignored Windows-native release evidence prints `EDITOR_PROGRESS_JOB_INDEX_BENCH_V1` with live
entry count, lookup count, legacy candidate checks, indexed probes, reduction basis points,
elapsed nanoseconds, and the elapsed-time ceiling. Exact elapsed time remains pending coordinator
terminal evidence.

## Validation

- Exact Rustfmt and scoped `git diff --check`: passed.
- Existing projection, capacity, duplicate-job, automatic replacement, concurrent capture, and ID
  reuse coverage is retained; an explicit replacement-retirement regression and ignored release
  evidence are prepared for one shared Editor10 coordinator batch.
- No local Cargo lane was launched and no compilation is being monitored in real time.
- Final validation ticket, terminal marker values, integration commit, and WeCom delivery remain
  pending.

## Remaining Parent-plan Work

Progress snapshots still clone rows and job snapshots, stable ticks still acquire locks, and text
changes can still advance wider center projection state. Immutable generation snapshots, typed
deltas/cursors, row revisions, windowed projection, scope aggregation, and product CPU/allocation
budgets remain open.
