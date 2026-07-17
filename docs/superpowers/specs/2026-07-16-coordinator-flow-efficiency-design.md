# Coordinator Flow Efficiency Design

## Problem observed

The coordinator is protecting too many transitions as if they were irreversible. In the 48 hours ending 2026-07-16T15:35Z, the live database recorded 6,492 periodic Codex synchronizations, 46,891 `codex.session.updated` events, and 2,454 business-Session status changes. Almost every periodic scan reported source metadata changes, so the event stream and SQLite writer are busy even when no operator decision is required. The same period also contains 918 `active`/`stale` reversals and 402 controlled-action failures, most caused by workflow bookkeeping or preflight conditions rather than a failed build or conflicting write.

The resulting UI reports counts, but does not answer the operator questions that determine whether a Session can continue: what resource is actually scarce, who owns it, how long it has been waiting, whether work is blocked or merely advisory, and what safe next action exists.

## Product direction

The coordinator is a local collaboration accelerator, not a general approval system. It must preserve only three hard boundaries:

1. One writer owns a concrete overlapping file path at a time.
2. One managed job owns a concrete exclusive Cargo/GPU resource at a time.
3. An irreversible Git commit verifies its own scope at commit time.

Everything else is advisory, observable, and retryable. Session registration, heartbeats, evidence collection, health reporting, audit writes, and independent validation admission never wait on another Session's unrelated work.

## Reference patterns

- Redmine-style saved views motivate task rows with filters, selected columns and a direct next action, rather than one undifferentiated event feed. [Redmine issue list](https://www.redmine.org/projects/redmine/wiki/RedmineIssueList)
- Jenkins separates queueing, execution and pipeline stages, and uses stages to present progress rather than treating every transition as a global lock. [Jenkins Pipeline](https://www.jenkins.io/doc/book/pipeline/)
- Kanban WIP limits apply to the constrained work column, not to all incoming work. The coordinator will therefore limit only concrete resource lanes and surface wait age/owner. [Atlassian Kanban WIP limits](https://www.atlassian.com/agile/kanban/wip-limits)

## Architecture

### 1. Quiet synchronization and stable Session state

Codex discovery remains periodic, but a scan writes a Session event only for user-visible lifecycle changes: discovery, source location, state, binding, diagnostic code, last event, or last turn. Source revision and synchronization timestamp stay as metadata without becoming timeline events. Completed sync runs with zero visible changes are retained as compact run telemetry rather than appended to the operator audit stream.

Business Session status changes are similarly classified. A stale heartbeat is a health signal, not a workflow transition: the UI shows it as `attention` with an age and recovery action. A real terminal/archive transition remains durable. This removes active/stale flapping from the work board while preserving the audit trail.

### 2. Friction budget for workflows

The coordinator exposes `hard block`, `waiting resource`, `attention`, and `ready` as distinct states. A missing topology record, missing lease, stale evidence, or manifest mismatch becomes a structured preflight diagnostic that tells the Session exactly what to refresh; it does not prevent unrelated Session work, heartbeat, or independent validation.

Commit is the only workflow action that requires the full attribution, Failure and content gates. The WeCom delivery remains an idempotent post-commit side effect keyed by commit SHA; delivery failure is visible but never rolls back the commit or blocks Session progress.

### 3. Operator work board

The Overview becomes a Kanban-style work board built from the existing control snapshot:

| Lane | Meaning | Cards |
| --- | --- | --- |
| Ready | Session can continue now | active Session with no exclusive wait |
| Waiting resource | A specific Cargo/GPU/lease owner must finish | owner, resource, age, queue position |
| Attention | Recoverable preflight or stale heartbeat | reason, age, suggested CLI/action |
| Needs intervention | True failed validation or open Failure | failure code, linked path, owner |

The board also shows a compact 24-hour experience strip: sync volume, no-op suppression, validation pass/fail/retry counts, orphan recovery count, and current SQLite/event pressure. Existing Sessions, Validation and Audit pages retain their detailed tables; the board links to filtered rows rather than duplicating authority.

### 4. Durable, bounded telemetry

Snapshot projections add an `experience` section computed from persisted history. It contains fixed-size rolling buckets and current blockers, never raw command output, secrets, or unbounded event payloads. The event stream filters operational noise at the producer; the audit page can opt into compact sync-run telemetry when detailed diagnosis is needed.

## Delivery milestones

1. **M1 — Event and state stabilization:** suppress no-op Codex timeline events, preserve compact sync telemetry, classify Session freshness without status flapping, and add experience projection contracts.
2. **M2 — Flow simplification:** make preflight failures actionable/advisory where safe, restrict hard blocking to path/resource/commit boundaries, and make post-commit WeCom behavior explicit and non-blocking.
3. **M3 — Work board:** add the Overview board, resource queue cards, experience strip, links to detailed views, and focused React contract tests.
4. **M4 — Experience validation:** compare the 24-hour metrics before/after M1–M3, run focused coordinator/web suites, and verify that unrelated Session admission continues during a waiting Cargo lane.

## Acceptance criteria

- A stable source scan does not emit `codex.session.updated` or a user-visible audit event.
- The service exposes enough data to identify the exact resource owner, wait age, queue position and next safe action for every waiting Session.
- A heartbeat/staleness observation never closes general Session admission or hides a runnable Session behind a terminal-looking state.
- Validation and workflow preflight diagnostics clearly distinguish recoverable operator work from hard blocks.
- WeCom delivery occurs after a durable commit and cannot undo or delay that commit.
- The board makes current flow, waiting work, recoverable attention and true failures visible without reading raw SQLite or logs.
