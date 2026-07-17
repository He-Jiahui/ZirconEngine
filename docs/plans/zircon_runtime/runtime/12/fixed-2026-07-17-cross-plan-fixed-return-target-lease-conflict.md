---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: cross-plan-fixed-return-target-lease-conflict
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/runtime/12
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/server.py
tests:
  - tools/session_coordinator/tests/test_server.py::ServerTests::test_scoped_failure_return_allows_origin_plan_destination_lease
resolved_at: 2026-07-17
---


# Coordinator01：跨计划 fixed return 与 origin 目录 lease 冲突

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`
- 来源执行者：`runtime12-input-event-bounds-closeout-r2-20260717`
- 来源执行切片：M5 input event bounds failure→fixed return 与 immutable managed commit
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：fixed artifact 的跨 child-plan 移动由协调器控制面原子执行；Runtime12 不能也不应手工释放 Plugins02 的目录 lease。

## 失败现象与复现证据

- Runtime12 已将 Performance01 的 `input-event-growth-and-frequency` 成功原子回传。
- 同一 M5 的 Sound02 `input-event-buffer-visibility` return 被拒绝为 `failure_return_lease_missing`：目标 `docs/plans/zircon_plugins/02/fixed-2026-07-17-input-event-buffer-visibility.md` 位于 active Plugins02 目录 lease `docs/plans/zircon_plugins/02` 下，fixing Session 无法合法 claim 该精确目标。
- 当前 server 要求 fixing Session 同时拥有 source、receipt 和 origin destination 三项 lease；目录层级排他使这一要求与“fixed 文件必须回到 origin 子计划”的规则互相矛盾。

## 最低共享层根因

`CoordinatorApplication._require_scoped_failure_return_leases` 将 origin destination 当成普通 fixing Session 写入。对于 active origin directory lease，这会把正确的跨计划交接误判为租约缺失；绕过检查、手工释放 origin lease 或直接写文件都会破坏 shared-main 排他。

## 架构修复验收

- fixing Session 必须继续持有 failure source 和 fixing-child receipt 的 live lease。
- 对唯一的 `fixed-*` origin destination，协调器只在其被 **active origin-plan Session** 的 live lease 覆盖时授予一次性 lifecycle transfer；不把 destination lease 转给 fixer，也不释放 origin lease。
- origin lease owner 的 `plan_path` 必须等于 failure node 的 `origin_plan`；不相关 Session、失效 owner、非 origin 目录/文件 lease 仍以 `failure_return_lease_missing` 拒绝。
- return 在单一 coordinator action 中继续原子写 destination/receipt、删除 source、刷新 graph，并记录 delegated origin-destination audit evidence。
- Runtime12 可回传 Sound02 failure 后以仅其 immutable M5 manifest 完成 managed commit；Render01 仍在该 SHA 前保持无 reservation。

## 禁止临时方案

- 不得手工释放、转移、缩小或覆盖 Plugins02 的目录 lease。
- 不得调用不带 Session 的 failure return 来跳过 scoped lease 校验。
- 不得复制 failure 文件、留下 open/fixed 双份，或让 Runtime12 手工 Git stage/commit。

## 修复结果与回传

- 根因：Scoped failure return required the fixer to own the origin fixed-artifact destination, which conflicts with a live origin-plan directory lease.
- 架构修复：The coordinator now grants the single fixed destination write only when a live covering lease belongs to an active Session whose plan_path exactly matches origin_plan; the fixer must still own the failure source and fixing-child receipt.
- 验证：Current-source focused regressions passed 4 tests in 9.087s, including allowed and rejected cross-plan returns plus exact-child directory ownership and milestone manifest binding; daemon instance 9c58203b34bb47ac9afd746a3a4290ac loaded schema 48 before this live return.
- 回传：Loaded and proved delegated origin-destination lease handling under the current daemon.
