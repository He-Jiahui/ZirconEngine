---
handoff_kind: fixed
status: fixed
created_at: 2026-08-29
summary_slug: codex-spool-directory-durability
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/codex_sync/durability.py
  - tools/session_coordinator/codex_sync/spool.py
  - tools/session_coordinator/tests/test_codex_durability.py
  - tools/session_coordinator/tests/test_codex_spool.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_codex_durability -v
  - python -m unittest tools.session_coordinator.tests.test_codex_spool -v
resolved_at: 2026-08-30
---

# Coordinator01: Codex spool replace does not flush directory metadata

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P1-040` in `docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the Codex spool namespace durability contract.

## 失败现象与复现证据

`CodexTriggerSpool` fsyncs each temporary file and atomically replaces it into `pending`, `quarantine`, or a repository marker path, but it never flushes the containing directory. A successful Hook therefore proves file-content flush and namespace atomicity, not persistence of the new directory entry across power loss. Acknowledgement and overflow rejection unlink entries with the same gap.

Focused tests patch the directory durability boundary. Current enqueue, quarantine, and acknowledgement paths never call it, and no portable Windows/POSIX implementation exists.

## 最低共享层根因

File-content durability and namespace durability were treated as the same operation. `fsync(file)` does not make a later `replace` or `unlink` directory entry durable. Windows additionally requires a write-capable directory handle opened with `FILE_FLAG_BACKUP_SEMANTICS` before `FlushFileBuffers` can flush directory metadata.

## 架构修复验收

- Provide one fail-closed directory flush primitive for Windows and POSIX.
- On Windows, use an exact existing directory handle with write access, full sharing, `OPEN_EXISTING`, and `FILE_FLAG_BACKUP_SEMANTICS`, then require `FlushFileBuffers` success.
- On POSIX, open the directory itself and require `fsync` success.
- Flush the affected directory after successful enqueue/marker replacement, quarantine moves, acknowledgement unlink, and overflow rejection cleanup.
- Reject regular files and propagate unsupported/failed durability rather than claiming file-only persistence.

## 禁止临时方案

- Do not treat close, rename, file fsync, or `Move-Item` as directory durability.
- Do not swallow a failed directory flush on accepted enqueue/marker paths.
- Do not add sleeps, retries, global volume flushes, or raw shell commands.
- Do not weaken spool privacy, capacity, acknowledgement, or quarantine rules.

## 修复结果与回传

- 根因：The spool fsynced temporary file contents but never flushed directory metadata after replace or unlink operations.
- 架构修复：A fail-closed Windows/POSIX directory flush primitive now durably closes every accepted spool namespace mutation, including pending and marker replaces, quarantine moves, overflow cleanup, and acknowledgement unlink.
- 验证：RED proved the helper and integration calls absent; GREEN Windows/spool 13/13, Codex regression 34/34, consumers 2/2, py_compile and diff check passed.
- 回传：Codex spool atomic file writes now include fail-closed directory metadata durability for accepted namespace mutations on Windows and POSIX.
