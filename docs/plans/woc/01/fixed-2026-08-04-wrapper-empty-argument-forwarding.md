---
handoff_kind: fixed
status: fixed
created_at: 2026-07-22
summary_slug: wrapper-empty-argument-forwarding
origin_plan: docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/woc/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/zircon-session.ps1
  - tools/session_coordinator
tests:
  - powershell -NoProfile -Command "& .\tools\zircon-session.ps1 -Json -Command status"
  - powershell -NoProfile -Command "& .\tools\zircon-session.ps1 -Json -Command lease -Arguments @('claim','--session-id','<session>','<path>')"
resolved_at: 2026-08-04
---


# Session Coordinator 01: wrapper forwards an empty argument and loses terminal lease evidence

## 来源执行者

- 来源计划：`docs/plans/woc/01-woc-zrvm-one-to-one-replication.md`
- 来源执行切片：M7 source-first Delve admission contract / shared-file lease.
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：最低共享原因位于所有 Zircon sessions 共用的
  `tools/zircon-session.ps1` 参数转发与终端结果契约，不属于 WOC 源码。

## 失败现象与复现证据

在 `E:\Git\ZirconEngine`，下列只读命令稳定返回 CLI 参数错误，而最近协调上下文脚本同时通过服务 health endpoint
报告 `status=ok`：

```powershell
& .\tools\zircon-session.ps1 -Json -Command status
```

终端结果：

```json
{"status":"error","error":{"code":"cli_arguments_invalid","message":"unrecognized arguments: ","details":{}}}
```

包装器把 `ValueFromRemainingArguments` 的空值附加到 Python CLI。`status` 没有位置参数，因而不能读取服务状态。相同会话中的
`lease claim`/`lease release` 也间歇地只输出 wrapper 启动前缀 `ready` 或 `starting`，没有 acquire/release/reject terminal
JSON；重试同一精确 claim 后才可能返回 `{"lease":{"acquired":true,...}}`。这使调用方无法分辨请求未提交、已提交但未回执，
或已被 coordinator 拒绝，破坏了共享 checkout 的唯一写排他证据。

## 最低共享层根因

`tools/zircon-session.ps1` 无条件将 `$Arguments` 拼接到 Python module argv。未传递剩余参数时，该值仍以空 CLI token
到达 coordinator parser；此外 wrapper 对启动前缀与实际命令 terminal envelope 没有单一可观察结果契约。WOC 不得依据
`ready` 推断 lease 已获得，也不得以未确认写绕过 coordinator。

## 架构修复验收

- 无剩余参数的 `status` 调用只向 coordinator 传递 `status`，返回正常 JSON status envelope，不能附加空 token。
- 任何 `lease claim`、`lease release`、failure 或 Cargo 命令都返回恰好一个 terminal JSON envelope；启动状态只能作为
  内部连接信息，不能替代 command result。
- 对启动竞争、服务重连和 command timeout，terminal envelope 必须显式区分 `not_submitted`、`accepted_pending`、
  `completed` 与 `rejected`，并带稳定 request/operation identity，调用方无需重放可能已执行的写请求。
- 添加 PowerShell wrapper 回归测试：无参数 status、带参数 lease claim/release、daemon startup race 与 timeout/reconnect。
- 重新执行本 handoff 的 WOC lease reproduction；WOC 才能恢复对 admission/source 文件的安全协调写入。

## 禁止临时方案

- 不得让 WOC 或任一项目在没有已确认 lease 的情况下写共享文件。
- 不得通过直接调用 Python coordinator、OS 文件锁、忽略 terminal result 或额外重放写请求绕过 wrapper。
- 不得把 `ready`/`starting` 文本当作成功的 lease 或 Cargo 命令结果。

## 修复结果与回传

- 根因：PowerShell ValueFromRemainingArguments forwarded an empty token and JSON mode exposed coordinator startup prefixes instead of one terminal command envelope.
- 架构修复：The wrapper now filters empty remaining arguments, preserves literal argument arrays, and suppresses ready/starting text in JSON mode; request lifecycle handling provides stable terminal reconciliation.
- 验证：Fresh Python real-subprocess regression passed 1/1 across PowerShell 7 and Windows PowerShell 5.1. Live WOC01 replay completed PS7 claim, PS5 release/claim, and PS7 release with one successful JSON envelope per command.
- 回传：Coordinator wrapper empty-argument and terminal-envelope contract is fixed and live WOC01 lease coordination replay passed.
