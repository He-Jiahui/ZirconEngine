---
related_code:
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/work_continuations.py
  - tools/session_coordinator/web/src/api/contracts.ts
  - tools/session_coordinator/web/src/api/validation.ts
  - tools/session_coordinator/web/src/components/validation/ValidationLaneTable.tsx
  - tools/session_coordinator/web/src/pages/OverviewPage.tsx
  - tools/session_coordinator/web/src/pages/ValidationPage.tsx
implementation_files:
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/work_continuations.py
  - tools/session_coordinator/web/src/api/contracts.ts
  - tools/session_coordinator/web/src/api/validation.ts
  - tools/session_coordinator/web/src/components/validation/ValidationLaneTable.tsx
  - tools/session_coordinator/web/src/pages/OverviewPage.tsx
  - tools/session_coordinator/web/src/pages/ValidationPage.tsx
plan_sources:
  - docs/superpowers/plans/2026-07-17-coordinator-validation-flow-health.md
  - docs/superpowers/plans/2026-07-17-coordinator-adaptive-cpu-burst-lanes.md
  - user: 2026-07-17 optimize coordinator visibility and Session validation efficiency without global admission blocking
tests:
  - tools/session_coordinator/tests/test_control_snapshot.py
  - tools/session_coordinator/web/src/__tests__/contracts.test.ts
  - tools/session_coordinator/web/src/__tests__/components.test.tsx
doc_type: module-detail
---

# Validation lane queue visibility

## Purpose

The coordinator separates Session admission from exclusive validation resources.
An occupied CPU or GPU Cargo lane may delay a managed validation command, but it
must never imply that Session registration, file work, Failure handling, or the
work board has been drained.

## Projection boundary

`ControlSnapshotService` projects at most 20 active `cargo_lane_reservations`.
Each row contains only a reservation identifier, owner Session identifier, lane
scope, execution mode, declared burst eligibility, lifecycle state, FIFO
position inside that mode, creation time, and expiry. Commands, target
directories, compatibility profiles, environment, instantaneous CPU or memory
samples, and other internal scheduling data remain outside the browser
snapshot.

CPU warm-cache, CPU isolated-burst, and GPU queues are numbered independently.
A running or leased reservation remains position one; pending reservations
appear after it in stable `created_at`/reservation-id order. Terminal rows are
omitted, so the page answers the operator's immediate question—who is using a
lane and who is next—without turning historical scheduling noise into a work
blocker.

The snapshot also has one bounded `cpuBurst` object: capacity is fixed at one,
`active` is zero or one, and `eligiblePending` counts only warm pending CPU
reservations declared as eligible. Resource admission happens only when a
reservation is consumed. A page refresh never samples the machine, changes a
queue, or delays Session work. During a rolling upgrade, an older daemon's
missing burst fields are rendered as warm-only with `0/1` burst WIP.

Both `validation.cargoJobs` and `validation.currentCargoTargets` are separate
safe lane projections. A row contains only the job and owner Session IDs, lane
kind, lifecycle status, creation/start/finish/release timestamps, artifact
cleanup policy/status, and an enum process-observation conclusion. For running
work the conclusion is `observed`, `awaiting_observation`, or `reconciling`;
terminal work is `not_applicable`. The former is bounded job history; the latter
remains one existing latest row per target for live artifact-lifecycle counting.
Target directory existence and cleanup aggregation stay server-side. Managed
commands, absolute target paths, compatibility/reuse payloads, PIDs, raw
process-tree timestamps, exit codes, and cleanup error text never enter either
browser array.

## Browser behavior

The Validation page renders the queue above current Cargo targets. It groups
warm CPU, one optional isolated CPU burst, and GPU into separate WIP summaries;
an eligible warm pending check is visibly marked `可隔离检查`. Pending and leased
rows show their reservation expiry. A running row instead shows whether its
process tree has been observed, is awaiting observation, or is being reconciled;
an observed job explicitly states that a slow heartbeat does not terminate it.
Its original reservation expiry does not mean the running validation has
expired. The Overview page repeats warm-cache WIP,
burst capacity, the queue head, and oldest waiting age as compact flow-health
information. Ages are informational: the browser never predicts an ETA from
them, and an invalid timestamp becomes an unknown age.

Its fixed copy states that the queue only orders validation. A full or
unavailable burst WIP is capacity information, not a request to stop, drain,
or force-stop work. This distinction is intentional: the UI must not confuse a
resource wait with global Session admission.

When a Session is waiting on validation or a lease, the coordinator may project
one unchecked implementation/documentation slice from that Session's own
numbered plan as a continuation. This is code-first advice, not a scheduler
mutation: the queued validation keeps its FIFO position, the worker must still
claim a concrete non-conflicting scope, and it completes one code slice at a
time before selecting the next one. Validation becomes the next work item only
when no such code slice remains or it naturally reaches a terminal result.

The runtime contract permits an absent `cargoReservations` field during a
rolling daemon upgrade by supplying an empty list. For legacy reservation rows,
it supplies the warm mode and false eligibility; an absent `cpuBurst` becomes
`{ capacity: 1, active: 0, eligiblePending: 0 }`. New snapshots enforce a
20-row bound, exact field names, allowed lane/mode/state enums, and one-based
positions before rendering.

## Test coverage

`test_control_snapshot.py` verifies mode-local FIFO order, single burst WIP,
exact safe-lane keys, and the observed/reconciling process conclusion for both
browser arrays. The web contract test rejects an invalid zero position while
covering legacy defaults, and the component test verifies that unexpected path,
command, PID, and cleanup-error fields are never rendered. These tests are run
in the current validation stage; no Cargo build is required for this
Python/TypeScript projection work.
