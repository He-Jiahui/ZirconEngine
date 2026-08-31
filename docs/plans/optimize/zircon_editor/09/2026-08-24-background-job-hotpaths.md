---
title: Editor09 Background Job Hot-path Optimization
category: zircon_editor
report_id: Editor09-hotpaths-2026-08-24
date: 2026-08-24
session_id: root-editor09-keyed-cancel-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor09 Background Job Hot-path Optimization

## Scope

This slice closes the keyed cancellation-authority defect and six boundedness or scaling gaps in the
shared Editor job core:

- keyed pending merge refreshes the progress authority's cancellation token and presentation data;
- job events use a bounded count/byte/age journal with progress coalescing and typed lifecycle-gap
  delivery;
- a backpressured progress record cannot replace the coalescing index of a newer concurrent update;
- primary progress selection uses a maintained priority index and snapshots share label/message
  storage;
- fair admission skips priorities with no ready jobs;
- progress observer callbacks receive transferred batches rather than one dispatch-lock round trip
  per event;
- admission keys and mutex-group identities reject oversized retained strings with non-retaining
  typed errors;
- job tickets support deadline-bounded waits without consuming a timed-out receiver or holding the
  ticket lock for the whole wait;
- dependency builder deduplication uses ordered binary search, making equivalent dependency sets
  independent of builder order.

## Implementation

The event journal defaults to 4,096 records, 16 MiB retained bytes, and five minutes maximum age.
Progress updates coalesce per `JobId`; dropped lifecycle ranges become one lossless
`JobJournalGap`, allowing consumers to resynchronize instead of silently accepting an incomplete
lifecycle. Pump backpressure restores the record to the journal. If a newer progress update arrived
while the old record was outside the queue, restoration coalesces the stale record and preserves the
newer sequence index.

`EditorJobProgressSource` stores labels and progress messages as `Arc<str>` and maintains
`BTreeSet<(priority_rank, JobId)>` for primary selection. Pending admission maintains ready counts
per priority. Observer dispatch transfers the queued `VecDeque` under one lock and invokes callbacks
outside the lock. Admission identity budgets are 256 UTF-8 bytes for keyed admission and 128 UTF-8
bytes for mutex groups.

`JobTicket::wait_until` temporarily takes exclusive receiver ownership, waits outside the mutex, and
restores the receiver after a timeout so callers can retry. `EditorJobSpec::after` maintains sorted
unique `JobId` values with binary search instead of rescanning every previously added dependency.

## Performance Evidence

| Evidence | Before | After / bound | Structural change |
| --- | ---: | ---: | ---: |
| Paused consumer, 32,768 lifecycle events | 32,768 retained, unbounded growth | <= 4,096 records and <= 16 MiB | >= 87.5% retained-entry reduction |
| Primary projection, 9,999 non-primary updates over 10,000 jobs | 99,990,000 candidate scans | 9,999 maintained-index reads | 99.99% lookup-work reduction |
| Shared snapshot strings | repeated copied label/message bytes | 0 copied string bytes after registration/update | 100% repeated string-copy reduction |
| Empty weighted priority slots, 4,096 selections | 196,608 category probes | 32,768 probes | 83.3333% probe reduction |
| Observer batch, 1,024 events | 1,026 dispatch-lock acquisitions | 2 acquisitions | 99.8051% lock-acquisition reduction |
| Oversized admission/mutex identity, 1 MiB input each | 1 MiB retained in each error path | 0 input bytes retained | 100% retained-input reduction |
| Dependency deduplication, 4,096 unique IDs | 8,386,560 linear comparisons | <= 45,057 binary-search comparisons | >= 99.4627% comparison reduction |

Actual elapsed time, throughput, high-water bytes, and final retained depth are accepted only from
the Windows-native release evidence ticket.

## Validation

- Exact `rustfmt --check` and scoped `git diff --check`: passed.
- Full `core::jobs` regression batch plus seven release evidence tests: pending coordinator terminal
  evidence.
- Ticket deadline/receiver retry tests and ordered dependency-set tests are submitted together in a
  second focused batch; neither is accepted from a single isolated test run.
- Initial corrected-ownership ticket `b5a1dcd1a1724f5a8b668f5c8df7a8ca` failed before Cargo because
  `.cargo` was requested as a dependency root but did not exist in pinned HEAD `858350a...`.
- Core ticket `d79991c986e84551837f2ab8642f3d06` and ticket/dependency ticket
  `0b8cc9f7a8d949ada04e863c84b26d6e` each use one direct release Cargo invocation for their source
  slice. This allows the coordinator to discover and pin sibling `zr_vm` while compiling all
  `core::jobs` regressions and ignored evidence once per independently changing slice. The job-core
  slice includes the backpressure restore regression found during self-review. Terminal evidence
  remains pending and is not inferred from either queued receipt.

## Shutdown Boundary

The top-level shutdown coordinator has a separate correctness ticket. Its `app.rs` source currently
contains unrelated active edits, so it is intentionally excluded from this job-core integration
candidate even if its validation passes. This record does not claim the Editor09 product shutdown
P0 is integrated.

## Remaining Parent-plan Work

The parent plan's durable history, retry/persistence, dependency failure policy, plugin lifecycle,
full product shutdown barrier, GPU/file-process ownership, and long-session soak requirements remain
open. This record accepts only the job-core slice listed above.
