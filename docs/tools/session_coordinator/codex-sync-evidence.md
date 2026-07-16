---
related_code:
  - tools/session_coordinator/codex_sync/evidence.py
  - tools/session_coordinator/codex_sync/history.py
  - tools/session_coordinator/codex_sync/worker.py
  - tools/session_coordinator/codex_sync/hook.py
  - tools/session_coordinator/failures.py
  - .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py
implementation_files:
  - tools/session_coordinator/codex_sync/evidence.py
  - tools/session_coordinator/codex_sync/history.py
  - tools/session_coordinator/codex_sync/worker.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/failures.py
  - .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py
plan_sources:
  - user: 2026-07-15 collect cross-session evidence under the local Codex sessions directory and keep scheduler state current
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
tests:
  - tools/session_coordinator/tests/test_codex_evidence_projection.py
  - tools/session_coordinator/tests/test_codex_worker.py
  - tools/session_coordinator/tests/test_codex_hook.py
  - tools/session_coordinator/tests/test_failures.py
  - .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/test_validate_plan_failure_handoffs.py
doc_type: workflow-detail
---

# Codex Sync Live Evidence Projection

## Responsibility

`CodexEvidenceProjector` creates one bounded, prompt-free realtime view at
`C:\Users\HeJiahui\.codex\sessions\YYYY\MM\zircon-engine-evidence-live-YYYY-MM-DD.md`.
It is an operational projection, not the canonical home for milestone evidence or
failure handoffs: those remain in their numbered `docs/plans` child directories.

The Codex hook only signals the repository-scoped worker. The worker discovers and
commits session state first, then atomically replaces this projection. A partial
write therefore cannot leave readers with a truncated evidence file.

## Realtime Boundary

The projection intentionally excludes historical noise:

- Codex rollout sessions are limited to active-source records with activity in the
  last four hours, at most 50 rows.
- Coordinator sessions must have a heartbeat in the same four-hour window. Among
  those, the projection includes active execution states and recently registered
  sessions, at most 100 rows. A stale or merely abandoned `active` record is not live
  work; neither are completed, cancelled, and archived sessions.
- Cargo contains only leased or running jobs.
- Controlled actions include executing actions and terminal actions completed in the
  same four-hour window; previews and expired previews are not operational evidence.
- Open `failure-*` nodes are listed independently, ordered by priority, so a routing
  blocker remains visible even when its originating Session is waiting.

This keeps the file useful for concurrent sessions while preserving the coordinator
database and numbered child-plan records as the sources of complete history.

## Incremental Historical Evidence

Every successful live projection advances the current `sessions/YYYY/MM` history
cursor; the first sync after startup and each fifteen-minute full sync additionally
renders the monthly history view. The collector reads only repository-owned
`rollout-*.jsonl` files and persists a cursor, prefix hash, pending-call metadata,
revision, and completion flag for each source. A pass has an 8 MiB aggregate budget
and a 512 KiB per-source budget, so a large active rollout advances incrementally
without preventing other sources from being discovered.

An unchanged source is skipped only after `scan_complete=1`. If a bounded read
lands exactly at its last newline, the next pass performs a zero-byte EOF probe and
persists the completion flag without reparsing records. A trailing partial JSONL
line instead keeps its prior offset and remains retryable until a terminating
newline arrives. These two states must remain distinct: EOF completion closes a
stable source, while a partial tail is active input and must never be discarded.

The collector classifies only validation, commit, failure-return, cleanup, and
task-terminal outcomes in memory, then stores a hashed source ID and hashed event
key with the sanitized result.

The durable rows live in `codex_evidence_sources` and `codex_evidence_records`.
`zircon-engine-evidence-history-YYYY-MM.md` is an atomically written, bounded
monthly projection of those rows; it renders at most 500 recent entries. The live
page gets a four-hour `最近外部会话证据` slice on every sync, so scheduler users can
compare external execution outcomes with coordinator-managed Cargo and controlled
actions without opening a rollout file.

Monthly totals and rows are filtered to that calendar month. If a collector observes
zero eligible evidence while a historical summary already exists, it leaves the
existing file intact instead of replacing useful evidence with an empty projection.

This evidence is audit-only. It may surface that an external validation or commit
occurred, but it cannot automatically accept a milestone, release a Cargo job,
clear a Failure, or create a Git commit. Those state transitions remain controlled
coordinator actions with their existing ownership and gate checks.

## Failure Classification Boundary

The evidence projection may name the Failure graph in an ordinary dated output
record, for example `2026-07-15-live-evidence-window-and-failure-chain.md`. That
does not make the record a handoff. Both the immutable-action artifact snapshot and
the handoff validator classify only canonical `failure-{date}-{summary}.md` /
`fixed-{date}-{summary}.md` files, plus legacy date-first names ending explicitly
in `-failure-handoff.md` or `-fixed-handoff.md` so that those legacy names can be
reported as invalid.

This prevents a documentation title from producing false Failure schema diagnostics
or blocking another Session's valid `failure return`. The canonical numbered-child
handoff rules, provenance checks, and graph import remain unchanged.

## Privacy Contract

The projection may expose identifiers, enum states, relative plan paths, relative
handoff paths, timing, and sanitized action outcomes. It must never emit session
prompts, command lines, raw validation logs, CWD values, absolute paths, webhook
URLs, bearer tokens, browser tickets, cookies, or maintenance credentials. Rollout
references are reduced to their basename before rendering.

The incremental collector applies the same contract to SQLite: it does not store
prompt text, shell input, tool arguments, tool output, raw call IDs, source paths,
or CWD. Command and output text exist only transiently while a bounded tail is
classified; the stored event key is a one-way hash.

## Verification

`test_codex_evidence_projection.py` verifies the output location, sanitization,
incremental history de-duplication, exact-budget EOF completion, partial-tail retry,
per-source and aggregate budget behavior, empty-projection preservation, worker
ordering, the four-hour filtering boundary, and inclusion of an open Failure chain.
The Failure classifier regressions prove that the dated evidence record is
ignored while a legacy `...-failure-handoff.md` name remains rejected. A coordinator
reload must wait for active managed Cargo and validation work to finish naturally,
then replace the daemon as one instance without entering a global draining mode or
closing unrelated task admission; no evidence format update may interrupt another
Session's verification.
