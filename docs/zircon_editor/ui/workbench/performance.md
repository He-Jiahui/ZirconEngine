---
related_code:
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/model/build/workbench_view_model_build.rs
  - zircon_editor/src/ui/workbench/reflection/model_build.rs
  - zircon_editor/src/ui/workbench/reflection/transient_ui_state.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/workbench/layout/manager/focus.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
implementation_files:
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/workbench/layout/manager/focus.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/reflection/route_registration/activity_routes.rs
  - zircon_editor/src/ui/workbench/reflection/transient_ui_state.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/workbench/window_registry/editor_window_registry.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-07-17-editor-workbench-static-review.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
tests:
  - hierarchy_depth_projection_memoizes_parent_chains
  - dual_asset_surfaces_share_one_projection_build
  - repeated_layout_commands_report_unchanged
  - canonical_activity_windows_are_borrowed
  - route_registration_does_not_clone_the_activity_projection
  - transient_projection_borrows_paths_and_reuses_properties
  - render_submission_borrows_the_viewport_controller
doc_type: module-detail
status: in_progress
---

# Workbench performance contract

Workbench 的性能边界是从 editor authority 取得一个稳定 generation，再派生 snapshot、view model、reflection 与 retained UI。锁只用于取得稳定 snapshot；模型构建、route materialization、JSON/TOML 投影、文件 I/O 与 publish 不得持有 `WorkbenchShellState` 或 command authority 锁。

当前直接优化保证：scene hierarchy 深度一次索引计算；同一 editor snapshot 的两个资产表面共享一次 catalog projection；canonical layout 查询和 registry/descriptor index 借用既有数据；重复 layout/focus 命令不发布伪变更；transient/route 更新不复制整行；render submission 不复制 viewport controller。

这些优化不替代最终 generation 架构。完整 `EditorDataSnapshot`、`WorkbenchViewModel` 与 `EditorWorkbenchReflectionModel` 仍是 owned trees；pointer、typing、selection 与 layout burst 必须先合并 dirty domains，并在每帧最多为受影响 domain 构建一次。资产 catalog、layout/descriptor、command/menu、extension、project/preset 与 template surface 各自拥有 immutable generation，consumer 只能组合这些 generation，不能复制出第二份 authority。

动态验收必须同时记录 build count、visited rows、clone bytes、锁等待/持有和 interaction p95。正常 idle 的未变 domain build count 为 0；1k event storm 的每帧/domain build count 不超过 1；focus/no-op layout 只访问受影响路径；render controller clone bytes 为 0。当前源码 Cargo、WPR 与 RenderDoc 证据完成前，本契约状态保持 `in_progress`。
