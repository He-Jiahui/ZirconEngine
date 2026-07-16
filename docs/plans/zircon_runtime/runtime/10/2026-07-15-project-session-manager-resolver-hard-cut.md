---
related_code:
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/handle.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/handle.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service.rs
  - docs/zircon_runtime/dynamic_api/session.md
implementation_files:
  - zircon_runtime/src/dynamic_api/session/project.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
tests:
  - managed Windows cargo test -p zircon_runtime --locked (job 8f545c28290d42f791e00940e764c659; removed resolver E0425 absent, compilation advanced to an unrelated UI test-consumer E0308)
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/session/project.rs
  - git diff --check -- zircon_runtime/src/dynamic_api/session/project.rs
  - source invariant scan for zero resolve_asset_manager calls and bounded handle-based resolutions
doc_type: milestone-detail
---

# Runtime10 Project Session Manager Resolver Hard Cut

Plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
Milestone: M1
Status: compile_unblock_verified
Date: 2026-07-15
Files: ["docs/plans/zircon_runtime/runtime/10/2026-07-15-project-session-manager-resolver-hard-cut.md", "zircon_runtime/src/dynamic_api/session/project.rs"]

## 状态与完成项目

| 切片 | 状态 | 完成证据 |
|---|---|---|
| dynamic project-session manager resolver hard cut | `compile_unblock_verified` | 删除后的 `resolve_asset_manager` 调用已清零；asset、project-asset 与 navigation 三条 session use point 均通过 versioned handle 和 `resolve_manager_service` 在操作边界解析。受管默认特性 Runtime 测试已越过原 E0425，并在后续独立 UI test consumer 处停止。 |

## 根因与修复

Asset manager owner 硬切后，`RuntimeProjectConfig::open_project_assets` 仍调用已经删除的
`crate::asset::pipeline::manager::resolve_asset_manager`，并继续依赖旧 wrapper 的
`.shared()` 转换。该调用点属于 dynamic project-session consumer，不应恢复旧 resolver 或
增加 compatibility façade。

当前实现直接取得 `asset_manager_handle(core)`，并在 `open_project` 的 bounded operation
边界调用 `resolve_manager_service(core, handle)`。同一 owner 文件中的 project asset reload
queue 与 navmesh load 也收敛为 `project_asset_manager_handle` / `navigation_manager_handle` 加
共享 service resolver，移除了 named concrete-manager lookup 和旧 navigation helper。

没有保留 `pub use` shim、Arc-holder adapter、重复 resolver、静默 fallback 或旧模块路径。
模块文档 `docs/zircon_runtime/dynamic_api/session.md` 已记录这三条 handle-based use-point
解析及生命周期边界。

## 验证

- 红态：Shader04 fresh 默认特性 Runtime 编译在 `session/project.rs` 报 E0425，指出删除后的
  `resolve_asset_manager`。
- 绿态深度证据：managed job `8f545c28290d42f791e00940e764c659` 不再报告该 E0425，继续
  编译到独立的 `runtime_ui_text_render_contract` E0308。
- scoped rustfmt、diff check 与 source invariant scan 通过。
- 本记录只确认 resolver compile unblock；不宣称 Runtime10 总计划或全 workspace 完成。

## 剩余范围

Runtime10 仍为 `in_progress`。ABI 清册、session 失败路径、UI 镜像契约和 cdylib 重载收尾
继续服从原 M0-M3 拓扑，本切片不改变其完成状态。
