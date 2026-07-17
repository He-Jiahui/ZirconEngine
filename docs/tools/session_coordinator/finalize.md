---
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/database.py
implementation_files:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/migrations.py
plan_sources:
  - user: 2026-07-17 remove global coordinator blocking while retaining scoped finalization safety
  - user: 2026-07-17 optimize the coordinator, preserve nonblocking admission, and recover local work after service restart
  - docs/superpowers/plans/2026-07-17-coordinator-terminal-index-snapshot-retention.md
tests:
  - tools/session_coordinator/tests/test_git_finalize.py
  - tools/session_coordinator/tests/test_database.py
doc_type: module-detail
---

# Git finalization recovery lifetime

## Purpose

`GitFinalizeService` serializes a scoped Git index mutation so a shared-main
commit can be recovered after a process interruption without reverting
unrelated worktree files. The persisted `finalize_requests.index_snapshot` is
therefore recovery material, not an audit artifact and not a permanent copy of
the repository index.

## Behavior model

The service stores the previous index bytes only after the request enters
`finalizing`. Until the request reaches a terminal state, startup can use the
recorded pre-transaction HEAD, index-existence bit, and snapshot to decide
whether it must restore the index or reconcile a commit that already advanced
`HEAD`.

Every transition to `committed` or `failed` clears `index_snapshot` in the
same SQLite update that records the terminal outcome. This covers normal
finalize, milestone commit, forward reconciliation, and stale-mutex recovery.
It deliberately does not clear a `finalizing` request: the daemon may still
need those bytes after an unexpected restart.

## Scoped baseline policy

Baseline health is an observability signal, not a global finalization gate. A
degraded workspace can contain unrelated, preserved work from another Session;
its existence must not prevent a completed Session from committing its own
exactly attributed manifest. Finalization therefore avoids a full-workspace
scan and does not reject solely because the epoch is degraded.

The scoped safety boundary remains unchanged at preview and immediately before
the ref update: the baseline `HEAD` must still match, every requested byte must
be attributed to the requesting Session, its complete owned dirty scope must be
present, and index scope, live leases, queued patches, plan/Failure rules and
secret checks must all pass. Background observation continues to report the
degraded epoch until an explicit reconcile or accept operation resolves it.

## Upgrade retention and compaction

Schema 47 applies the same terminal-only rule to historical records during
startup. It never selects `finalizing` rows. For an existing database, the
migration checkpoints and vacuums only after the terminal BLOB update succeeds,
then records its schema marker. If physical compaction fails, the marker is not
written, so the next safe startup retries instead of claiming that disk debt was
removed.

This migration runs before the daemon accepts new mutations. It does not invoke
a global drain, suspend Session registration, or modify Cargo targets, baseline
manifests, plans, or Failure state. Normal offline replay continues to own the
small allowlisted registration and heartbeat requests while a service process is
unavailable.

## Constraints

- The index snapshot is never surfaced by control-plane projections or logs.
- Terminal request metadata, commit SHA, categories, validation evidence, and
  error text remain immutable audit history; only the obsolete recovery BLOB is
  removed.
- Baseline epoch manifests are a different retention concern and are not part
  of this module's migration.
- SQLite file swapping or live compaction beyond the schema-startup vacuum is
  intentionally out of scope until offline-queue replay and WAL handoff are
  designed together.

## Test coverage

`test_git_finalize.py` verifies normal terminal cleanup, failed stale-mutex
recovery after restoring the index, and an owned finalize against an unrelated
degraded baseline. `test_database.py` verifies that a simulated v46 upgrade
clears committed/failed BLOBs, preserves a live finalizing BLOB, and reduces
the physical database file. These tests are scheduled for the M1 testing stage
in the retention plan.
