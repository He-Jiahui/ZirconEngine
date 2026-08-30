---
handoff_kind: fixed
status: fixed
created_at: 2026-08-29
summary_slug: codex-hook-installer-schema-compatibility
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/install-codex-session-hook.ps1
  - tools/tests/codex-session-hook.Tests.ps1
tests:
  - powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/tests/codex-session-hook.Tests.ps1
resolved_at: 2026-08-30
---

# Coordinator01: Codex Hook installer binds compatibility to DB schema 28

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P1-042` in `docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the public Hook-to-daemon compatibility contract.

## 失败现象与复现证据

`Test-DaemonCompatible` in `tools/install-codex-session-hook.ps1` requires `runtime.schema_version -eq 28`. The live Coordinator runtime descriptor is version 2, advertises control API v1, and currently uses internal database schema 69, so installer `Query` reports `daemonCompatible=false` for the healthy supported daemon.

The acceptance fixture reproduces the defect with descriptor v2, exact repository identity, loopback host, and control API v1 across schema 68, 69, and 70. All three are protocol-compatible, but the current implementation rejects every one because none equals the obsolete internal schema 28.

## 最低共享层根因

The installer treats the Coordinator database migration version as a public Hook protocol compatibility version. Internal schema migration is not part of the Hook-to-daemon control contract; the runtime descriptor version and advertised control API versions are.

## 架构修复验收

- Determine compatibility from the supported runtime descriptor version window and required control API capability.
- Preserve exact loopback host and repository identity checks.
- Do not require a particular internal database schema version.
- Accept current, immediately preceding, and immediately following internal schemas when the public descriptor/API contract is unchanged.
- Reject unsupported old/future descriptor versions, missing control API v1, malformed descriptors, and foreign repository identity.

## 禁止临时方案

- Do not update the literal from 28 to the current schema number.
- Do not accept arbitrary runtime descriptor versions or omit repository identity validation.
- Do not probe private database state from the installer.
- Do not weaken Hook installation or removal ownership behavior.

## 修复结果与回传

- 根因：The installer compared the internal database migration schema to obsolete public hook compatibility version 28.
- 架构修复：Validate the supported runtime descriptor version window, loopback host, repository identity, and required control API capability; ignore internal schema migration.
- 验证：codex-session-hook.Tests.ps1 passed; schema 68/69/70 compatibility and descriptor/API/repository rejection cases passed.
- 回传：Codex Hook installer compatibility fixed and returned; internal schema is no longer a public protocol gate.
