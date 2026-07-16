# Coordinator Offline Replay Without Admission Blocking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep coordinator admission permanently nonblocking while preserving locally submitted, safe CLI commands when the daemon is offline and replaying them only after a healthy start.

**Architecture:** Add a repository-local, atomic JSON spool beneath the existing coordinator state root. The CLI and spool both enforce the same allowlist of state-convergent commands. A healthy `status` call—used by `tools/zircon-session.ps1 start`—uses a non-waiting single-consumer replay lock, processes envelopes in FIFO order, deletes an item only after daemon acknowledgement, and stops at the first retained or failed head without reordering dependent work.

**Tech Stack:** Python 3.14 standard library, existing `CoordinatorConfig`, `CoordinatorClient`, `tools/zircon-session.ps1`, `unittest`.

---

## Milestone M1: Durable offline intent handoff

**Goal:** Queue only safe local requests during an offline transport failure and deterministically replay them once a daemon is healthy, without altering supervision admission policy.

**In-scope behaviors:** Atomic queue files; exact schema and repository binding; FIFO replay; state-convergent retry; bounded queue; explicit allowlist; non-waiting single-consumer replay; `status`-triggered replay; no queues for process, Cargo, lifecycle, commit, cleanup, or interactive actions; queue status/replay CLI visibility.

**Dependencies:** Existing Codex hook spool atomic-write and quarantine conventions; current `CoordinatorClient` typed offline errors; existing PowerShell `start` invoking CLI `status` after health is published.

### Implementation slices

- [ ] **M1.1 Add the failing offline-spool contract.**
  - Create `tools/session_coordinator/tests/test_offline_command_spool.py` with a temporary `CoordinatorConfig` state root.
  - Cover exact envelope validation, atomic FIFO ordering, repository-key mismatch/unsafe-command quarantine, queue-cap rejection, acknowledgement deletion, non-waiting replay ownership, and failed-head retention.
  - First define the desired API:

    ```python
    spool = OfflineCommandSpool(config.offline_command_queue_root, repository_key="a" * 64)
    queued = spool.enqueue(command="session.heartbeat", arguments={"session_id": "session-a"})
    self.assertEqual("queued", queued.status)
    self.assertEqual((queued,), spool.validated_pending())
    ```

- [ ] **M1.2 Implement one bounded local command spool.**
  - Create `tools/session_coordinator/offline_queue.py`.
  - Reuse the Codex spool durability pattern: write an exact JSON envelope to a same-directory temporary file, `flush`/`fsync`, then `os.replace` it into `pending/` with a monotonic filename prefix.
  - Define `OfflineCommand` with `command_id`, `repository_key`, `command`, `arguments`, `created_at`, and schema version. Validate the complete key set, 64-character repository key, maximum encoded bytes, JSON-compatible arguments, and the safe command allowlist at both enqueue and read time.
  - Permit only commands whose duplicate delivery converges to the same coordinator state. A replay lock must return immediately when another local replayer owns it; after a crashed replayer, a dead-PID lock may be recovered and the retained state-convergent command may be retried.
  - Quarantine malformed, oversized, foreign-repository, or unreadable items; cap pending items at a documented bounded count without deleting valid pending work.

- [ ] **M1.3 Add CLI admission and replay tests before CLI behavior.**
  - Extend `tools/session_coordinator/tests/test_deferred_action_client.py` to prove a command that raises `CoordinatorClientError("offline", ...)` is queued exactly once only when it belongs to the explicit allowlist.
  - Extend `tools/session_coordinator/tests/test_milestone_cli.py` to prove a succeeding `status` replays queued FIFO command envelopes, stops after the first transport loss, and does not replay lifecycle/Cargo/commit/cleanup commands.
  - Use a small injected `execute_command(command, arguments)` callback so the replay tests run against the real spool rather than mocks of its persistence.

- [ ] **M1.4 Route safe CLI calls through the durable fallback.**
  - Modify `tools/session_coordinator/cli.py` to centralize command construction in one helper returning `(command_name, arguments)` before invoking `CoordinatorClient.command`.
  - Allowlist only `session.register`, `session.heartbeat`, and `lease.heartbeat`; reject offline queueing for every other command with the existing typed `offline` result.
  - On a direct offline transport error, persist the envelope and return a success-shaped `{"status": "queued", "queueId": ..., "command": ...}` response, never execute local work or start a process.
  - Add `offline-queue status` and `offline-queue replay` CLI subcommands. `status` only reports pending/quarantined counts. `replay` requires a healthy runtime descriptor, processes FIFO once, and returns acknowledged/retained/quarantined counts.
  - Make the existing `status` path call replay after `health()` succeeds and attach a concise `offlineReplay` summary. This makes the existing PowerShell `start` recover the local queue automatically, without modifying supervision state or the launcher.

- [ ] **M1.5 Document operational boundaries.**
  - Update `docs/cli-and-tooling/local-session-coordinator.md` with the allowlist, durable on-disk location, FIFO/retry semantics, status/replay command examples, and the guarantee that drain remains observation-only.
  - Update `docs/tools/session_coordinator/control-plane.md` with the invariant: offline replay is a local client handoff and cannot create jobs, reservations, lifecycle intents, commits, or a maintenance hold.

### Testing stage M1-T

```powershell
python -m unittest tools.session_coordinator.tests.test_offline_command_spool tools.session_coordinator.tests.test_deferred_action_client tools.session_coordinator.tests.test_milestone_cli -v
python -m tools.session_coordinator --repo-root E:\Git\ZirconEngine --json offline-queue status
python -m tools.session_coordinator --repo-root E:\Git\ZirconEngine --json status
git diff --check -- tools/session_coordinator docs/cli-and-tooling/local-session-coordinator.md docs/tools/session_coordinator/control-plane.md
```

Expected: all focused tests pass; queue status is read-only; the live `status` call reports replay state without blocking any work; the diff has no whitespace errors. If a test exposes a lower-level spool invariant failure, repair `offline_queue.py` first and rerun upward through the CLI path.

**Exit evidence:** Focused unit output; an offline-enqueued safe command replayed after a fresh `start`/`status`; an attempted offline Cargo or lifecycle command remains typed offline and is never queued; docs describe the strict boundary.

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
