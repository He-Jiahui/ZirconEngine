# Coordinator Terminal Index Snapshot Retention Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent terminal Git-finalization recovery snapshots from indefinitely consuming coordinator storage, then reclaim historical terminal snapshots through an auditable, non-admission-blocking maintenance path.

**Architecture:** `finalize_requests.index_snapshot` is needed only while a request can be recovered after an interrupted index mutation. `GitFinalizeService` clears it atomically when the request becomes terminal; schema 47 applies the same terminal-only rule to historical rows before it records the migration marker, then checkpoints and vacuums an existing database. The migration runs before daemon admission and does not add a drain, registration gate, or Cargo dependency.

**Tech Stack:** Python 3.14, SQLite WAL, existing `Database` transactions, `unittest`, coordinator CLI maintenance.

---

## File and responsibility map

- `tools/session_coordinator/git_finalize.py` — retains an index snapshot only during `finalizing`; clears it on normal, reconciled, and recovered terminal transitions.
- `tools/session_coordinator/tests/test_git_finalize.py` — proves terminal finalization no longer retains recovery bytes while interrupted recovery still restores the index.
- `tools/session_coordinator/tests/test_database.py` — proves historical terminal BLOB cleanup, live-finalizing preservation, and physical SQLite compaction during a v46-to-v47 upgrade.
- `docs/tools/session_coordinator/finalize.md` — describes the recovery-lifetime invariant and the bounded, non-draining cleanup behavior.
- `docs/cli-and-tooling/local-session-coordinator.md` — links operator-facing retention semantics to the maintenance command.

## Milestone M1 — terminal snapshot lifetime and legacy retention

**Goal:** Make index recovery bytes live only for active finalization and reclaim already terminal historical rows at a safe daemon startup without touching baseline manifests, Cargo targets, or Session admission.

**Dependencies:** Existing `finalize_requests` schema and the migration runner's established pre-marker compaction pattern. The concurrent baseline-head-advance work owns `tools/session_coordinator/baselines.py` and is explicitly out of scope.

### Implementation slices

- [ ] Clear `index_snapshot` in every `GitFinalizeService` update that transitions a request to `committed` or `failed`, including normal finalization, milestone commit, forward reconciliation, and stale-mutex recovery. Keep the snapshot until that terminal update so a process crash before the transition remains recoverable.
- [ ] Add schema 47 that clears only non-null `index_snapshot` BLOBs on `committed` and `failed` rows. It must never select `finalizing` rows and must not change baseline, Cargo, Session, or Failure tables.
- [ ] Extend the migration runner with the existing pre-marker safety pattern: update terminal BLOBs in a transaction, checkpoint/vacuum an existing database, then write version 47. If compaction fails, leave the version marker absent so a later startup retries.
- [ ] Add concise recovery-lifetime comments beside the terminal updates; do not add a compatibility column, direct SQL operator path, global drain, or daemon lifecycle action.

### Testing stage

- [ ] Run `python -m unittest tools.session_coordinator.tests.test_git_finalize tools.session_coordinator.tests.test_database` through a coordinator-managed Windows Cargo-free Python validation scope.
- [ ] Add and verify tests for: a normal finalize clears `index_snapshot`; stale-mutex failed recovery restores the index before clearing the BLOB; historical committed/failed BLOBs clear; a live `finalizing` row survives; and the upgraded SQLite file shrinks.
- [ ] Run `git diff --check` for all touched source and docs, inspect the schema marker retry path, and capture before/after count and byte totals from a read-only database query.

### Exit evidence

- [ ] Focused tests pass with no unhandled exception.
- [ ] Schema 47 clears historical terminal BLOBs while preserving a live finalization record.
- [ ] Clearing leaves `Session` admission `read_write`; no drain or maintenance hold is invoked. The normal daemon start boundary owns the brief migration work and offline replay remains the restart recovery path.

## Milestone M2 — zero-downtime compaction design (separate acceptance)

**Goal:** Design, but do not yet enable, a copy-verify-swap SQLite compaction path for a future case where a maintenance window cannot use the safe startup vacuum while preserving offline queue replay and current writes.

**Dependencies:** M1 accepted; an explicit lifecycle design proving that live WAL writes cannot be lost during a file swap.

### Implementation slices

- [ ] Measure post-M1 `page_count`, `freelist_count`, and active database byte size to establish whether startup vacuum already resolves the pressure.
- [ ] Write a child-plan design before code naming the snapshot boundary, queue replay order, integrity checks, backup retention, atomic replacement conditions, and rollback trigger.
- [ ] Reject implementation if it needs a global Session drain, suppresses registration, or bypasses the offline queue requirement.

### Testing stage

- [ ] No implementation validation occurs in M2; it is accepted only as a reviewed design artifact with an explicit later implementation owner.

### Exit evidence

- [ ] A concrete recovery-safe design exists or M2 remains intentionally open with measured evidence; it must not claim reclaimed disk space.

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
