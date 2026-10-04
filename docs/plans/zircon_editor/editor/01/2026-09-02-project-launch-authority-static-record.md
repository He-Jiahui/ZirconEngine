---
related_code:
  - zircon_editor/src/ui/host/editor_manager_startup.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_manager_project_session.rs
  - zircon_editor/src/core/project/authority/preflight.rs
  - zircon_editor/src/core/project/project_launch_preflight.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
plan_sources:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/mvp/index.md
  - docs/plans/engine-code-structure-convention.md
status: source_implemented_static_validation_pending
---

# Editor01 项目启动 authority 接线静态收敛记录

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M2 | ProjectLaunchPreflight authority 单一路径守卫与性能测量计划 | source_implemented_static_validation_pending | 2026-09-02 | 当前 checkout 已包含并经复核的 `ProjectAuthority::preflight_project_launch`、`ProjectLaunchPreflight`、`EditorManager::execute_project_launch_preflight`、创建/打开 session admission 接线；更新 `src/tests/ui/boundary/host_cutover.rs`，断言 startup wrapper 仅转发 intent/preflight，项目政策留在 `ProjectAuthority`，旧 `ProjectPreflightCompositionProfile`/engine compatibility/FS preflight 逻辑不得回流。新增 `docs/plans/optimize/zircon_editor/01/2026-09-02-project-launch-authority-measurement-plan.md`，状态为 `measurement_pending`，定义空工程/1k/10k 规模和 Windows E:/D:/F: 性能采集边界。已完成 `git diff --check`（本 scope）；未执行 managed Cargo、真实 Editor 产品启动、ETW/WPR、CPU/RSS/GPU 或 WeCom/里程碑提交，因此不宣称 M2 accepted 或 MVP/F0/F1/F4 晋级。 |

## 当前未完成

- 需要在 Windows coordinator managed lane 对精确 source manifest 执行 Editor project/preflight focused tests，并确认当前大量外部 dirty 输入不会污染结果。
- 需要真实产品运行验证创建/打开、session root 一致性、资产权威加载和首帧 handoff；静态守卫不能替代 MVP 产品闭环。
- 性能测量计划仍为 `measurement_pending`，在取得 current-source profile 前禁止修改启动/扫描热路径。

## 后续项目

1. 先完成 MVP `00` 的 current-source compile baseline，再按 F0/F1 顺序进行产品验证。
2. 对本切片执行受管 Windows validation；若失败，按最低 owner 建立 failure，不在 UI 层增加 fallback。
3. 取得阶段计时、I/O、RSS、首帧和 p95 后，再决定是否需要 authority generation 或增量扫描等结构性算法变更。
