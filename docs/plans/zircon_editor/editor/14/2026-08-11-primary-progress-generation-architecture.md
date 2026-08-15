# Primary Progress Generation Architecture

## Scope

The retained editor host currently asks `EditorJobProgressSource` for a cloned
primary snapshot on every tick. It then creates task strings and invokes the
status setter even when neither the visible primary job nor its progress has
changed. This document defines the cross-module hard cut required to make a
stable progress frame a read-only, allocation-free bypass.

This is an architectural contract for the current M3 source candidate. It is
not an accepted output record and does not close the open Editor14 JobPump
failure.

## Measured Source Path

- `zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs` calls
  `sync_editor_job_progress()` on every retained-host tick.
- `zircon_editor/src/ui/retained_host/app/job_progress.rs` obtains
  `primary_job_progress_snapshot()`, builds a `StatusTaskProgress` with
  `format!` and `to_string`, then calls the status setter.
- `zircon_editor/src/core/jobs/progress.rs` now keeps the primary generation and
  active `BTreeMap` under one mutex. Equal-generation reads clone no snapshot,
  but the current implementation still acquires that mutex on every retained
  tick, so UI stability can contend with admission, progress and completion
  writers even when no presentation fact changed.
- `zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects/status.rs`
  clones the current status value to compare it before assigning and
  invalidating.

The retained tick projection review already identifies this path as the
remaining source of stable-frame snapshot, String, formatting, setter, and
invalidation work:
`docs/plans/performance/01/2026-07-31-editor-retained-tick-projection-adapters-current-review.md`.

## Reference Boundary

Unreal's `FAsyncTaskNotification` keeps task-state transitions at the task
notification implementation boundary, while
`CoreAsyncTaskNotificationImpl.cpp` compares authoritative notification state
before scheduling an update. Slate keeps the UI ticker and presentation work in
`SlateAsyncTaskNotificationImpl.cpp` rather than transferring job-state
ownership into the widget layer.

Zircon's closest proven generation pattern is
`zircon_runtime/src/text/font/shared.rs`: a cheap atomic generation probe guards
the steady-state hot path, while replacement state and generation publication
share one lock boundary so a snapshot cannot pair new state with an old epoch.

Zircon should retain the same division, without copying Unreal APIs: the job
source publishes an authoritative visible-primary generation, while the
retained UI owns its observed generation and performs presentation work only
when that fact changes.

## Chosen Contract

`EditorJobProgressSource` exposes one generation-aware read operation:

```rust
pub fn primary_snapshot_if_changed(
    &self,
    observed_generation: Option<u64>,
) -> Option<EditorJobPrimaryProgressSnapshot>
```

`EditorJobPrimaryProgressSnapshot` contains the monotonically advancing
visible-primary generation and an optional primary snapshot. The API has four
required outcomes:

| Input and source state | Result | Required work |
|---|---|---|
| first observation (`None`) | `Some` including the current optional primary | initialize or clear UI once |
| equal published generation | `None` | no state-mutex acquisition, snapshot clone, string construction, setter, or invalidation |
| changed generation | `Some` including the new optional primary | rebuild the presentation once |
| future/unknown generation | authoritative locked comparison | never suppress the current source snapshot from a mismatched cursor |

`ProgressState::primary_generation` remains the only authoritative generation.
The active jobs, primary selection and generation still change under the same
mutex; the atomic value is only a published mirror for a stable-read negative
check, never a second snapshot or mutation owner.

Every visible-primary change first computes and checks the next generation
without changing the projection. It then applies the projection mutation,
assigns that exact generation to `ProgressState`, and stores the same value to
the atomic mirror with `Release` ordering before the state lock is released.
It must not use an independent `fetch_add`, because that could let the mirror
diverge from the state it describes.

A retained reader loads the mirror with `Acquire`. Only an exact equality may
return `None` without the mutex. `None`, stale, changed and future generations
must acquire the state mutex, compare again against the authoritative value,
and clone only when that locked comparison still differs. This second compare
prevents a publication race from manufacturing a redundant snapshot.

The generation changes only when the visible primary projection changes:

1. registration changes the lowest visible non-terminal `JobId`;
2. a progress payload changes that visible primary;
3. terminal marking or completion removes that visible primary and exposes the
   next visible primary or no primary.

All four mutation routes call one source-owned reserve-and-publish helper while
the state guard is live. Readers that race a writer may temporarily retain the
old observation. A later call that acquires the newly published mirror value
then observes the authoritative generation; Acquire/Release does not promise a
fixed UI-tick bound, so convergence timing belongs to the dynamic profile gate.
No reader can return a snapshot paired with a different epoch.

Changes to non-primary jobs do not advance this generation. Job identifiers
remain sorted by the existing `BTreeMap` order; production issuance is
monotonic, but the contract must also preserve correct primary selection for
unordered ids used by tests.

## Ownership And Hard Cut

- **Editor14** owns `EditorJobProgressSource`, the visible-primary generation,
  the atomic published-generation probe, the locked snapshot operation, and
  source-level behavior tests. The current unaccepted M3 fragment implements
  this boundary; managed behavior and performance validation remain open.
- **EditorUI08** owns the retained-host observed-generation cursor, status
  task materialization, and the no-change bypass. That owner updates
  `app.rs`, `app/job_progress.rs`, the status dispatch path, and the host
  runtime accessor as one UI manifest.
- The final Editor14 M3 source manifest removes the legacy unconditional
  `primary_snapshot()` API and its runtime accessor after EditorUI08 consumes
  the generation-aware operation. No alias, facade, dual-path fallback, or
  compatibility shim remains after the migration.

The source and UI changes must land as ordered exact manifests because a
source-only removal would break the retained host, while a permanent bridge
would preserve the old per-tick clone path.

## Behavior Matrix

Editor14 source tests must cover all of the following before the source
manifest can be accepted:

| Scenario | Generation result | Snapshot result |
|---|---|---|
| initial empty read | returned once | `None` primary |
| repeated stable read | unchanged | no returned payload |
| stable read while the state mutex is held elsewhere | unchanged | returns `None` without waiting for the mutex |
| future observed generation | locked source generation | returns the authoritative snapshot instead of suppressing it |
| primary progress value changes | increments once | updated primary |
| non-primary progress changes | unchanged | no returned payload |
| primary terminals | increments once | next primary or empty |
| terminal followed by completion | no duplicate increment | no duplicate presentation |
| arbitrary insertion order | first visible `JobId` remains primary | stable ordering |

## Managed Performance Gate

EditorUI08 must additionally instrument 100,000 stable retained ticks and
assert zero progress-state mutex acquisitions, source snapshot clones,
task-string formatting, status setter calls, and presentation invalidations
after the initial synchronization. A changed primary and an empty transition
must each acquire and update exactly once.

Capture uses the coordinator-built Windows profiling editor through
`tools/ui-profile-capture.ps1 -SkipBuild -UseWpr`, with `CARGO_TARGET_DIR`
under the approved `D:`, `E:` or `F:` managed roots and output under
`E:\zircon-profiles` or another approved non-`C:` root. The source manifest,
Perfetto timeline, hotspot counters and `system.etl` remain linked evidence.

The report records stable/mismatch call counts, mutex attempts and wait time,
snapshot and string allocation bytes, frame p50/p95, UI-thread CPU samples and
context switches. On hardware exposing package-energy or residency counters,
record before/after energy per 100,000 ticks under the same idle scene; otherwise
state that power evidence is unavailable rather than inferring it from elapsed
time. Comparison is same-machine before/after plus the documented Unreal
change-driven design, not an unsupported cross-machine absolute watt claim.

This gate remains open until those artifacts exist. Static source tests and a
wall-clock-only microbenchmark cannot establish runtime contention or power.

## Relation To Open Failure

The open JobPump failure remains focused on bounded lifecycle delivery,
lossless Editor02 producer admission, controlled pump budgets, scale evidence,
and Windows WPR. Generation-aware primary progress removes a separate
retained-tick read amplification. It may be validated as M3 only with its own
current-source manifest and must not be presented as a substitute for the
JobPump failure return.
