---
handoff_kind: fixed
status: fixed
created_at: 2026-08-03
summary_slug: validation-ticket-large-manifest-cli-transport
origin_plan: docs/plans/mvp/00-current-source-baseline-recovery.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/mvp/00
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/zircon-session.ps1
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/validation_tickets.py
  - tools/session_coordinator/tests/test_validation_tickets.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_validation_tickets
resolved_at: 2026-08-04
---


# Coordinator01: validation ticket large-manifest CLI transport

## 来源执行者

- 来源计划：`docs/plans/mvp/00-current-source-baseline-recovery.md`
- 来源执行切片：M0.1 Runtime15 receipt-tree hard-delete managed validation preparation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：最低共享根因是 Coordinator CLI transport；Runtime15 和 MVP00 不应绕过受管命令入口。

## 失败现象与复现证据

初始复现中，Runtime15 的 current-source hard cut 包含 1,849 个已删除 Rust 文件。validation
ticket 已支持 JSON `null` 删除墓碑，但 `zircon-session validation submit` 只接受单个
`--source-manifest-json` 参数；该 manifest 超过 Windows 32K command-line 上限，无法通过
canonical wrapper 提交。

当前源码已提供互斥的 `--source-manifest-stdin` 路径，wrapper 以 UTF-8 管道传输大 manifest，
并保持 JSON stdout 的单文档语义。该前向修复尚未取得受管 terminal receipt，不能作为
Runtime15 lib-test 或 MVP00 M0.1 的 accepted evidence。

## 最低共享层根因

Coordinator service 的 JSON request body 能承载大 manifest，CLI 却只有 inline JSON argument，
缺少 stdin 等不受进程命令行长度限制的结构化输入路径。

## 架构修复验收

- `validation submit` 提供与 inline JSON 互斥的 stdin manifest 输入，并继续使用 `json.loads`
  与 `ValidationTicketService._manifest` 做唯一结构/路径/散列校验。
- 大于 32K、包含至少 1,849 个 `null` tombstone 的 manifest 可由标准
  `tools/zircon-session.ps1 ... -Json` 提交并返回单一 JSON receipt。
- inline 小 manifest 行为保持当前主线语义；非法 JSON、非 object 和非法值仍返回现有 typed
  coordinator error，不增加宽松截断或 fallback parser。

## 禁止临时方案

- 不允许 Runtime15 省略删除路径、恢复旧文件或直接写 coordinator 数据库。
- 不允许把 manifest 写入受版本控制的临时文件，或由业务 Session 绕过 CLI 直接调用内部服务。

## 修复结果与回传

- 根因：The coordinator service accepted structured source manifests, but validation submit exposed them only as one inline JSON command-line argument, so Windows could not transport the 1,849-entry deletion manifest beyond its command-line length limit.
- 架构修复：Add a mutually exclusive --source-manifest-stdin transport that feeds the same strict JSON and manifest normalization path as inline input. The PowerShell wrapper forwards stdin as UTF-8 without BOM ambiguity, restores caller encoding state, and preserves one-document JSON stdout; service boundaries reject non-standard JSON values before mutation.
- 验证：Managed ticket 69670cb657e8458495d33aa2f47d1517 passed 30/30 tests in validation copy job e38a4a76ec28438fb318708f6bfede16 (exit 0, 19.277s). A real MVP00 wrapper replay transported a 90,602-byte manifest with 1,849 null tombstones and returned exactly one JSON receipt: request d96b8decbc834f1b978e81578ca1b8bd, ticket 4b7aa3ce0a404f5d8c2c027ec17514a7. The synthetic ticket then failed at the independent expected validation_copy_overlay_not_owned gate.
- 回传：Coordinator01 returned large-manifest stdin transport to MVP00 with managed 30-test evidence and a real 1,849-tombstone canonical wrapper receipt.
