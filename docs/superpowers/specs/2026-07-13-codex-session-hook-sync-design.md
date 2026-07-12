# Codex Session Hook Synchronization Design

Date: 2026-07-13
Status: approved
Owner: `zircon_tooling/session_coordinator`
Repository: `E:\Git\ZirconEngine`

## Purpose

Synchronize every Codex Session whose canonical working directory belongs to ZirconEngine into the local Session Coordinator. The coordinator must expose those Sessions in the existing web control center, keep the synchronization non-blocking for Codex, preserve the user's existing Codex notification command, and never treat Codex's private local files as the coordinator's writable state database.

## Evidence and constraints

- The installed Codex configuration exposes a project/global `notify` command. Codex appends one JSON notification argument when a turn completes; no reliable public `SessionStart`/`SessionEnd` event bus is available in the current installation.
- Codex rollout files live below `$CODEX_HOME/sessions/YYYY/MM/DD/rollout-*.jsonl`; archived threads live below `$CODEX_HOME/archived_sessions/`. Their first `session_meta` record contains a thread/session ID and cwd.
- Rollout JSON may contain prompts, model instructions, tool data, environment values, and other private material. None of that raw content may be copied to the coordinator database, logs, Git, or hook queue.
- ZirconEngine currently develops in one shared `main` checkout. Hook synchronization must not create worktrees, branches, Git commits, file leases, or business Session status transitions.
- The coordinator may temporarily be offline, slow, on an older descriptor/schema, or represented by a stale runtime descriptor. A hook must not guess which process owns the repository.

## Considered approaches

### 1. Synchronize only the current notification payload

This is small and immediate, but it misses Sessions that have not just completed a turn, cannot reliably derive archived/active state, and depends too heavily on one notify payload shape. Rejected.

### 2. Read or watch Codex's private app database

This could expose richer titles and UI state, but it binds ZirconEngine to an undocumented schema and risks lock contention with Codex Desktop. Rejected.

### 3. Notify-triggered incremental reconciliation with periodic recovery

The hook writes a bounded, sanitized trigger; the coordinator then scans only changed rollout files and performs an idempotent reconciliation. Startup and periodic reconciliation recover missed notifications. This covers all ZirconEngine Sessions while keeping Codex files read-only and the hook fast. Selected.

## Architecture

```mermaid
flowchart LR
    C["Codex notify"] --> H["Project hook multiplexer"]
    H --> Q["Sanitized trigger spool"]
    H --> N["Existing global notifier"]
    Q --> W["Coordinator Codex sync worker"]
    S["Startup / periodic scan"] --> W
    W --> R["Read-only rollout reconciler"]
    R --> D["codex_sessions projection"]
    D --> A["Control snapshot / SSE"]
    A --> U["Sessions web page"]
```

### Hook multiplexer

The repository project config declares one notify command backed by a small Python entry point under `tools/`. The entry point:

1. accepts arbitrary notify arguments and finds the final JSON object when one exists;
2. reduces it to `eventType`, `threadId`, `turnId`, normalized cwd, and timestamp;
3. accepts the event only when cwd resolves inside the ZirconEngine repository;
4. atomically writes a trigger no larger than 4 KiB below `%LOCALAPPDATA%/Zircon Session Coordinator/codex-hook/<repository-key>/pending/`;
5. best-effort signals the authenticated local coordinator with a short timeout;
6. loads the existing global `notify` array from `$CODEX_HOME/config.toml`, rejects recursion, and forwards the original Codex arguments;
7. returns without blocking Codex on reconciliation, daemon startup, or network I/O.

The hook never writes SQLite and never serializes raw notification JSON. Queue filenames use random IDs rather than prompt-derived data. The queue is capped at 1,024 pending triggers; overflow removes the oldest trigger and emits only a sanitized counter when the daemon next reconciles.

### Rollout discovery and parsing

`CodexSessionDiscovery` reads only `$CODEX_HOME/sessions` and `$CODEX_HOME/archived_sessions`. It resolves every `session_meta.payload.cwd` and retains records inside the configured repository root using case-insensitive Windows path semantics and realpath containment.

The parser reads the first valid `session_meta` line plus a bounded tail window for the latest lifecycle event. It does not deserialize or retain base instructions, messages, tool calls, assistant output, environment context, attachments, goals, or token accounting. A malformed or concurrently appended final line is ignored until the next pass; the last known-good projection remains visible with a sanitized diagnostic.

Files are incrementally keyed by canonical path, size, and nanosecond mtime. A periodic full directory membership scan detects new, moved, archived, or removed rollout files without hashing their complete contents.

### Coordinator projection

Schema v27 adds `codex_sessions` and `codex_sync_runs`.

`codex_sessions` contains only:

- `thread_id` primary key;
- canonical rollout path and source location enum (`active`, `archived`, `missing`);
- state enum (`active`, `idle`, `archived`, `unavailable`);
- canonical cwd, originator, CLI version, thread source, and safe last-event enum;
- optional last turn ID;
- first-seen, last-activity, last-synced, source mtime, and source size;
- optional `bound_session_id` foreign key;
- sanitized diagnostic code, never exception text.

`codex_sync_runs` stores aggregate counts, duration, source revision, trigger kind, terminal status, and a sanitized error code. It does not store per-line JSON or filesystem exception messages.

Codex Session state is derived deterministically:

- latest unmatched task-start event: `active`;
- latest terminal/turn-complete event while the rollout is active: `idle`;
- rollout under archived storage: `archived`;
- previously observed rollout absent for two completed membership scans: `unavailable`.

The reconciler auto-binds only when an existing business `sessions.session_id` exactly equals the Codex thread ID. It performs no fuzzy title, goal, plan-path, or message matching. Codex projections never create leases, patches, Cargo jobs, workflow runs, or business Session status changes.

### Daemon scheduling and identity

The coordinator runs one bounded Codex sync worker per repository. Startup performs an initial reconcile. A 30-second lightweight membership tick schedules incremental reconciliation, and a 15-minute full pass repairs missed filesystem events. Hook signals only set the worker wake event.

Before signaling, the hook validates runtime descriptor version, repository key, process ID plus creation time, localhost address, and supported control API. A stale descriptor, schema mismatch, identity mismatch, read-only daemon, or multiple candidate daemon chains causes fail-closed queueing. The hook never launches a second daemon.

The worker uses the coordinator's existing single-daemon mutex and database transaction boundaries. One run may be pending while another executes, but duplicate triggers coalesce into one follow-up pass.

## Control API and web visualization

The control snapshot gains a bounded `codexSessions` collection and `codexSync` summary. Each Codex Session row shows state text, thread ID, source location, last activity, last sync, CLI/origin, exact business binding, and diagnostic code. The Sessions page keeps business Sessions and Codex Sessions in separate panels so source presence is never confused with lease/workflow authority.

SSE publishes only `codex.session.discovered`, `codex.session.state_changed`, `codex.session.archived`, `codex.session.unavailable`, and `codex.sync.completed` summaries. Payloads contain IDs, enums, timestamps, and counts only.

No browser action is required for normal operation. A maintainer-only controlled `codex.sessions.reconcile` action may request a new pass through the existing preview/confirm/audit protocol; it cannot choose arbitrary paths or read raw rollout contents.

## Installation and removal

`tools/install-codex-session-hook.ps1` supports `Query`, `Install`, `Update`, `Remove`, and `DryRun`.

- Install validates the trusted ZirconEngine project config, records no webhook or credential, and adds the repository notify command idempotently.
- The hook reads the unchanged global notify command at runtime and forwards it. It does not copy that command into Git.
- Query reports configured, forwardable, daemon-compatible, and queue-health states without printing command secrets or runtime tokens.
- Remove deletes only the exact managed project hook entry and the repository-scoped external spool; it does not alter the user's global notify command.
- Repeated install/update/remove operations are idempotent and preserve unrelated TOML keys and comments.

## Failure behavior

- Hook parse failure: enqueue a generic `payload_invalid` trigger if the cwd can still be proven; otherwise exit successfully without synchronization.
- Coordinator unavailable or slow: retain the trigger and return promptly.
- Rollout append race: keep the last known-good row and retry.
- Rollout malformed: set a sanitized diagnostic code and continue other Sessions.
- Queue corruption: quarantine the one invalid file below the managed root and continue.
- Multiple daemon identities or stale descriptor: do not signal or start a process; wait for periodic/startup recovery.
- Database migration failure: preserve v26 and fail daemon startup closed.
- Existing notifier failure: synchronization still remains queued; the hook returns the original notifier's exit code only when Codex's notify contract requires it, otherwise records no raw error output.

## Security and privacy invariants

- Never persist raw Codex notification payloads or rollout lines.
- Never persist prompts, assistant messages, goals, tool arguments, environment values, attachments, model instructions, runtime tokens, webhook URLs, or global notify commands.
- Accept only canonical paths inside the configured Codex roots and ZirconEngine repository.
- Limit file count, file size metadata, tail bytes, trigger bytes, queue depth, snapshot rows, and SSE payload size.
- Render all projected strings as text and validate every enum at the Python and TypeScript boundaries.
- Use authenticated localhost signaling and existing coordinator process-identity verification.

## Verification

The milestone must prove:

- notify argument reduction rejects secrets and non-Zircon cwd values;
- the existing global notifier receives the original argument vector exactly once;
- hook latency stays bounded with daemon online, offline, stale, and slow;
- active, idle, archived, missing, malformed, append-racing, and restored rollout fixtures reconcile idempotently;
- an initial full scan and repeated incremental scans produce the same projection;
- duplicate hook events coalesce and concurrent workers never overlap;
- v26 to v27 migration is atomic and idempotent;
- snapshots, SSE, runtime contracts, and Sessions UI remain bounded and text-only;
- install/query/update/remove preserve unrelated project/global config;
- full Python, Web, Windows Tray, controlled-action smoke, independent Critical/Important review, and a new source-frozen 24-hour soak all pass before final acceptance.

## Scope exclusions

- No Codex private app database access.
- No remote/cloud Session discovery.
- No prompt or conversation search in the control center.
- No automatic inference of plan ownership from user text.
- No hook-driven Git commits, leases, patches, validation jobs, or lifecycle actions.
- No worktrees or feature branches.
