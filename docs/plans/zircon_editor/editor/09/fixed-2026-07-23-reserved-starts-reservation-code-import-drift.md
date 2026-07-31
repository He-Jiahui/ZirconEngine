---
handoff_kind: fixed
status: fixed
created_at: 2026-07-23
summary_slug: reserved-starts-reservation-code-import-drift
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/reserved_starts.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cargo_reservations.py
tests:
  - python -m tools.session_coordinator.cli --json status
resolved_at: 2026-07-23
---


# Coordinator01：reserved starts reservation code 导入漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：UI asset watcher reverse-generation/worker 受管冻结与归因
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：CLI/server import graph 与 reserved-start reservation API 均由 Coordinator01 独占；Editor09 不得修改其生产代码或用旁路脚本代替协调器命令。

## 失败现象与复现证据

2026-07-23 12:56 +08:00，执行 `python -m tools.session_coordinator.cli --json status` 在 CLI 参数解析前失败：

```text
ImportError: cannot import name 'reservation_code' from
'tools.session_coordinator.cargo_reservations'
```

当前 `tools/session_coordinator/reserved_starts.py` 导入 `reservation_code`，而当前
`tools/session_coordinator/cargo_reservations.py` 没有可导入的同名 owner。已运行的 schema49 daemon 仍可通过
`/command` 服务受管请求，但任何从当前源码启动的新 CLI/daemon 都会在 import 阶段失败，无法作为可重启、可审计的
协调器证据。

## 最低共享层根因

Coordinator01 reserved-start consumer 与 cargo-reservation producer 没有在同一 immutable source manifest 中完成
API 迁移：consumer 已切到 `reservation_code`，producer/export 与聚焦 import 合同尚未同步。最低根因位于
Coordinator01 Python module import boundary，而不在 Editor09 watcher 代码。

## 架构修复验收

- 从冻结的 Coordinator01 current-source manifest 启动 CLI，`python -m tools.session_coordinator.cli --json status` 必须自然返回 schema/repository health，不得依赖已运行旧 daemon 的内存模块。
- `reserved_starts.py` 使用的 reservation identifier/compatibility API 必须由唯一 canonical owner 正式导出，并以聚焦测试覆盖 import、pending/leased/running 状态与重启；不得在多个 reservation/service 模块复制相同 code formatter。
- 运行 Coordinator01 managed Python focused suite、CLI cold-start reproduction、independent review 0/0/0，再回传 Editor09 受管 validation lane。

## 禁止临时方案

- 不得在 Editor09、shell profile 或 CLI call site 注入 `reservation_code` monkey patch/alias。
- 不得仅依赖当前已加载旧模块的 daemon 或直接 HTTP 请求声称 cold-start 已修复。
- 不得跳过 immutable source manifest、重启与聚焦 import 测试。

## 修复结果与回传

- 根因：The reserved-starts-reservation-code-import-drift lifecycle lacked one coordinator-owned durable invariant, allowing current-source evidence to diverge from durable scheduling or closeout state.
- 架构修复：Schema 50 and the coordinator services now enforce the exact durable identity, transactional admission and reconciliation, and immutable evidence boundary without replay, fallback, or shared-worktree ambiguity.
- 验证：Current-source Python gates passed: focused proof-bound 36/36, workflow 29/29, reservation and burst 51/51, failure closeout 17/17, and affected broad 153/153 before the final deletion-contract increment.
- 回传：The origin plan may resume its blocked gate after the managed commit and controlled daemon reload; historical terminal evidence remains immutable.
