---
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/cargo_reservations.py
  - tools/session_coordinator/cargo_runner.py
implementation_files:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/cli.py
plan_sources:
  - user: 2026-07-17 preserve declared Render01 → Render05 → Shader06 CPU scheduling order without reintroducing global drain
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-17-cpu-reservation-start-priority-overtake.md
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-17-source-manifest-build-config-cap.md
tests:
  - tools/session_coordinator/tests/test_cargo_reservations.py
doc_type: module-detail
---

# Session Coordinator Cargo Job Arbitration

## Responsibility

`cargo_jobs.py` is the sole authority for managed Cargo target selection, lease
lifecycle, exact-command binding, lane reservations, process identity recording,
and terminal cleanup handoff. A Session never creates an unrecorded target or
starts a command outside a coordinator job.

## CPU Priority Reservation Invariant

CPU reservations express a declared, exact command. Their durable FIFO states
are `pending`, `leased`, `running`, and `finished`; only release or expiry
removes an entry from arbitration. Each Session may keep one pending CPU entry;
distinct Sessions may queue in creation order behind the active lane, so a
declared task cannot lose its place merely because another Session receives the
single immediate-successor slot first.

The coordinator applies the invariant at both boundaries:

1. `acquire()` rejects a new unreserved CPU job when a CPU reservation already
   owns the FIFO head.
2. `start()` rejects a previously acquired, unreserved CPU job when a CPU
   reservation has appeared before that job starts. It first expires invalid
   pending entries and reconciles terminal entries in that same transaction, so
   an old reservation cannot become a permanent false blocker.
3. `consume-cpu-reservation` admits only the FIFO head. A later pending entry
   remains durable but cannot bind a job until every earlier reservation is
   terminally released or expired.

The second check closes the pre-lease race: a generic task may obtain a lease
while no reservation exists, but it cannot later leap over a Render01/Render05/
Shader06-style declared priority reservation. The rejection is
`cargo_cpu_lane_reserved` and includes only the reservation, owning Session,
and status needed for safe routing.

This is not a global drain or an admission shutdown. When no CPU reservation is
active, ordinary managed CPU jobs remain startable. Bound jobs continue to
require the exact command fingerprint and canonical compatibility payload that
were recorded when their reservation was created. A Session may correct an
unstarted pending command only when its compatibility payload and target are
unchanged; the reservation ID and FIFO creation timestamp remain unchanged and
the correction is appended to the coordinator audit stream.

### Source-manifest binding for shared-main validation

An exact reservation may carry `source_manifest` as a first-class field in its
canonical compatibility JSON. It maps repository-relative source paths to
uppercase SHA-256 digests and is persisted separately from the small textual
`build_config` profile. This is for validations whose result is meaningful only
for a particular current-source snapshot, such as a narrowly scoped visibility
or render regression test in the shared `main` worktree.

The manifest has an explicit 4096-entry and 256 KiB serialized-byte limit, and a deterministic
SHA-256 fingerprint: repository path keys and digests are case-folded, sorted
ordinally as `path=sha256`, joined with LF and no trailing LF. The reservation
result returns both the normalized `sourceManifest` and
`sourceManifestFingerprint`, making the exact current-source boundary
auditable without forcing a large file table through the 4096-character
`build_config` limit. The legacy
`build_config.source_manifest` form remains readable for historical
reservations, but new source-sensitive gates use the first-class field.

Directory leases are not source files. A caller must recursively expand an
owned source directory into its explicit Rust file entries before reserving;
the coordinator freezes and verifies every submitted entry. Render01's
compiled-pipeline gate has 68 entries because the
`compiled_render_pipeline` directory lease contributes four Rust children. No
partial-directory manifest is an accepted substitute.

The coordinator verifies the normalized manifest twice: before it persists the
pending reservation, and again in `cargo run-reserved` immediately before it
starts the child process. If any file is missing or its bytes no longer match,
start is rejected with `cargo_<lane>_reservation_source_manifest_stale`,
including the expected and observed digest. The job is left unstarted for an
audited owner release/replacement; it cannot produce accidental green evidence
for a later overwrite.

## Target and Process Safety

Target roots are selected only from approved drive-root pools. Reusable pools
are keyed by canonical compatibility; overlapping targets and live process trees
block conflicting work. A job's PID, process creation identity, observed
descendants, and terminal result are persisted so a failed supervisor cannot be
mistaken for permission to reuse or release a still-live process tree.

`start()` also records an initial managed descendant snapshot in the same
database transition that marks a job `running`. This keeps a real Cargo tree
visible while a large workspace scan is pending; a later watcher observation
may refine it, but an empty initial projection is never used as evidence that a
just-started managed job is safe to restart, release, or reuse.

### Explicit CPU target binding

`cargo reserve-cpu --target-dir <approved-root>` may name a pre-existing,
coordinator-managed warm pool when the next exact command must retain its build
outputs. The service validates the path with `TargetPathPolicy` before it writes
the reservation. `consume-cpu-reservation` then passes that persisted target
back into `acquire()`; callers cannot substitute a target between reservation
and start.

The explicit target is not a compatibility shortcut. The reservation still
stores the directional command's own canonical compatibility payload and exact
command fingerprint. Consumption continues to reject a live or overlapping
target, so a successor can reuse a pool only after its current job has reached
a real terminal release.

## Verification

`CargoReservationTests.test_unreserved_cpu_lease_cannot_start_ahead_of_consumed_priority_reservation`
reproduces the ordering defect with an existing generic lease followed by a
bound priority reservation. It expects the generic start to fail with
`cargo_cpu_lane_reserved`; the exact bound job remains eligible to start with
its approved command. The paired
`test_unreserved_cpu_lease_can_start_after_stale_priority_reservation_expires`
proves expired priority state is reconciled before the start decision.
`test_cpu_reservation_preserves_explicit_approved_target_when_consumed` proves
an approved CPU target survives reservation and binds the resulting leased job.
`test_cpu_reservations_queue_multiple_exact_successors_in_fifo_order` proves a
later Session cannot consume its reservation before the head, but receives its
exact lease after the head terminally releases.
`test_start_persists_initial_managed_process_tree_observation` proves a managed
supervisor's first Cargo descendants are persisted at the start transition,
before a background workspace scan can run.
`test_cpu_reservation_rejects_start_when_bound_source_manifest_drifts` proves
that a source-sensitive CPU reservation records its manifest and then refuses
to start after the same owned source file is changed.
`test_cpu_reservation_supports_first_class_large_source_manifest_and_rechecks_all_entries`
proves that a 1275-file expanded hard-cutover manifest larger than 4096 characters
persists independently from `build_config`, reports its deterministic
fingerprint, and still rejects a start after the final entry drifts.
`test_source_manifest_rejects_entries_past_explicit_hard_cutover_limit` proves
that the expanded limit remains bounded at 4096 entries.
