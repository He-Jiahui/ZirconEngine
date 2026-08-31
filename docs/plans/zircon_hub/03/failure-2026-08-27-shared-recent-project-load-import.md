---
handoff_kind: failure
status: open
created_at: 2026-08-27
summary_slug: shared-recent-project-load-import
origin_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
fixing_plan: docs/plans/zircon_hub/03-project-lifecycle-robustness.md
origin_child_dir: docs/plans/optimize/zircon_tooling/06
fixing_child_dir: docs/plans/zircon_hub/03
plan_link_mode: child_record_only
related_code:
  - zircon_hub/src/tauri_app/runtime_state/tests.rs
tests:
  - cargo test -p zircon_hub --lib focus_refresh_reconciles_pending_hub_recents_with_editor_registry --locked
---

# Hub03: shared recent-project loader test import and identity drift

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 来源执行切片：Tooling06 managed Cargo process-tree lifecycle current-source Hub library replay
- 修复责任计划：`docs/plans/zircon_hub/03-project-lifecycle-robustness.md`
- 交接原因：受管 Hub library gate 已越过 Coordinator process-tree lifecycle 与 lockfile admission，最低错误位于 Hub03 runtime-state inline test 的 projects API import 边界。

## 失败现象与复现证据

2026-08-27 受管命令 `cargo test -p zircon_hub --locked --verbose --lib` 真实启动 Cargo，并在编译 `zircon_hub` lib tests 时以 exit 101 终止。`zircon_hub/src/tauri_app/runtime_state/tests.rs:158` 调用 `load_shared_recent_projects`，但文件顶部的 `crate::projects` use 组只导入 `project_metadata_key`、`reconcile_shared_recent_projects` 与 `RecentProject`，因此 rustc 报 E0425。补齐 import 后，受管 focused test 已实际启动并暴露第二层 RED：测试用完整 `RecentProject` 相等性检查成员，而 reconcile 会按共享 registry 顺序 authority 推进 Hub 新记录的 `last_opened_unix_ms`。panic fixture 中 `hub.toml` 与 `recent_projects.json` 均同时含 HubGame 与 EditorGame，失败不是条目丢失。

## 最低共享层根因

Hub03 的 focus-refresh regression 使用公开 projects re-export 回读共享 recent-project registry，但测试模块未把该函数导入局部作用域；导入修复后，测试又把可由 registry 单调推进的时间戳误当成项目身份。生产身份 authority 已是 `project_metadata_key(path)`，测试应按同一 authority 验证 Hub/Editor 两个路径都存在，而不是锁定完整 DTO 的瞬时时间戳。生产 API、reconcile 行为和文件格式均不需要改变。

## 架构修复验收

- 测试模块从 `crate::projects` 的既有公开 re-export 导入 `load_shared_recent_projects`，不新增旁路 helper 或重复文件解析。
- 内存 config 与磁盘 shared registry 均使用既有 `project_metadata_key` 断言 HubGame/EditorGame 路径身份存在；不得删除任一侧断言，也不得锁死 registry 可推进的 recent timestamp。
- 受管 focused lib test 实际执行并通过，随后 Hub library gate 至少越过该 E0425；若出现新的外部错误，单独路由且不吸收到本节点。
- scoped rustfmt 与 `git diff --check` 通过。

## 禁止临时方案

- 不得删除或跳过 shared registry 回读断言。
- 不得在测试内重复实现 JSON loader、扩大生产可见性或改变 recent-project 持久化格式。
- 不得把 Tooling06 进程树修复或其他 Hub failure 合并进本单文件修复。

## 修复结果与回传

Open state: `managed compile RED and focused assertion RED captured; import plus identity-authority repair implemented; focused managed validation and fixed return pending`.
