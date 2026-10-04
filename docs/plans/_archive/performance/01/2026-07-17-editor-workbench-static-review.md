---
related_code:
  - zircon_editor/src/ui/workbench
  - zircon_editor/src/ui/layouts/views/asset_browser/name_compaction.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-layout-metadata-full-recompute-and-sync-io.md
  - docs/plans/zircon_editor/editor/09/failure-2026-07-17-editor-asset-catalog-full-rebuild-and-preview-lock.md
  - docs/plans/zircon_editor/editor/10/failure-2026-07-17-project-open-repeated-manager-scan.md
reference_sources:
  - dev/slint/internal/core/properties.rs
  - dev/slint/internal/core/model/repeater.rs
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource_format_binary.cpp
tests:
  - workbench performance source/behavior guards listed below
  - current-source Windows zircon_editor focused tests pending
  - editor idle/interaction/WPR and current RenderDoc cold/warm capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Workbench 逐文件性能静态审查（2026-07-17）

## 范围与当前覆盖

当前工作树 `zircon_editor/src/ui/workbench` 共 **327** 个 Rust 文件，已逐文件阅读 **327/327**。静态覆盖已完成，但当前源码 Cargo、编辑器产品 trace 与冷/暖 RenderDoc 门禁尚未完成，因此整个范围继续留在 `pending.md`，不得进入 `review.md`。

| 模块 | 已读/总数 | 主要结论 |
|---|---:|---|
| asset_content_layout / autolayout | 45/45 | 文件名塑形与 descriptor key 分配已修；完整 geometry 仍应由 layout generation 缓存 |
| debug_reflector / reference / fixture | 32/32 | reference/fixture 主要为诊断或测试；debug reflector 全树字符串投影列为 P2 诊断成本 |
| document/page/menu tabs + root contracts | 10/10 | 主要为常量、DTO 与小规模计算；未发现独立 P0 根因 |
| event / view / window_registry | 45/45 | window instance index 与 canonical activity windows 深拷贝已修；registry generation 仍归 PERF-MVP-099 |
| layout + layout preset | 41/41 | 重复命令 changed 语义已修；typed delta、metadata index、preset persistence/I/O 仍归 EditorUI08 |
| model / reflection | 71/71 | activity clone 与 transient property allocation 已修；整份 chrome/view/reflection/routes 重建是 PERF-MVP-099 主根因 |
| preset / project | 23/23 | preset 多为启动声明；project open/save/list 与资产 projection 的 generation/I/O 问题分别移交 Editor10/09 |
| snapshot / startup / state | 60/60 | hierarchy 记忆化、双资产表面、共享 clock、draft borrow、render controller borrow 已修；完整 snapshot generation 仍待架构门禁 |

## 已直接优化

- 场景层级深度从每行沿 parent 反复 `Scene::find_node` 改为一次 parent index 与记忆化遍历；`node_records()` 只冻结一次。
- Asset Activity 与 Asset Browser 共享一次 folder/filter/sort projection，再只改表面专属 mode/tab；search query 每 snapshot 只规范化一次，locator parent 使用 `rsplit_once`。
- canonical activity windows 返回 borrowed `Cow`；window registry 与 geometry descriptor indexes 借用稳定行/字符串，避免整 map/row/key 深复制。
- `ResizeSplit`、drawer mode/extent/tab、main page 与 focus 的重复命令准确返回 `changed=false`，阻止无效 metadata/reflection 连锁刷新。
- reflected inspector submission 借用 draft map；route registration 用 `mem::take` actions；transient projection 借用 node path 并复用既有 property descriptor。
- Asset Activity 与 Asset Browser 的长文件名 compaction 将 prefix 选择由线性 runtime shaping 改为二分；welcome recent rows 共享一次系统时钟。
- render submission 直接借用 `SceneViewportController`，删除每帧复制 state/handles/pointer bridge 的 render-only facade。

对应 RED→GREEN/行为守卫：

- `hierarchy_depth_projection_memoizes_parent_chains`
- `dual_asset_surfaces_share_one_projection_build`
- `asset_snapshot_normalizes_search_once_and_streams_parent_paths`
- `canonical_activity_windows_are_borrowed`
- `window_registry_indexes_instances_without_cloning_rows`
- `geometry_indexes_descriptor_rows_by_borrowed_id`
- `repeated_layout_commands_report_unchanged`
- `reflected_inspector_updates_borrow_the_draft_map`
- `route_registration_does_not_clone_the_activity_projection`
- `transient_projection_borrows_paths_and_reuses_properties`
- `shared_content_name_compaction_does_not_linearly_shape_every_prefix`
- `runtime_file_name_compaction_uses_logarithmic_prefix_search`
- `recent_project_labels_share_the_snapshot_clock`
- `render_submission_borrows_the_viewport_controller`

## 已移交的架构热点

- PERF-MVP-099 / EditorUI08：`WorkbenchShellState` 的单 mutex 与 snapshot→view model→reflection→route 全量构建缺少 domain generation；model 仍复制 active page、drawers、tabs、payload 与 menu contributions，transient event 仍扫描全部 nodes。
- PERF-MVP-077 / EditorUI08：普通 layout/focus 之后的 host metadata 仍以全图扫描为主；geometry 每次构造多个小 map/Vec，应该绑定 layout/size/descriptor generation，而不是另建可变 authority。
- PERF-MVP-088 / Editor09：asset workspace snapshot 仍复制 catalog strings/records；本次只消除同一 snapshot 内第二遍扫描，最终需要共享 immutable catalog/projection generation。
- PERF-MVP-100 / Editor10：welcome `probe_draft`、project load/save、locator 与 layout preset 路径重复 canonical/stat/open/scan/read/parse；save 前完整 scan 不能停留在 UI 主线程事务。
- PERF-MVP-101：debug reflector/历史帧对 1k/10k nodes 的长字符串/section 全量物化只在诊断启用时发生，需单独测量并虚拟化，不计入正常 idle/frame 成本。

## 参考实现

Slint 的 `PropertyTracker::evaluate_if_dirty` 与 repeater row dirty 路径表明，稳定 workbench 应按依赖 generation 只重建受影响 projection。Godot resource loader 先用 `ResourceCache` 与 modified-time identity 复用资源，支持 Zircon 将 project/catalog/preset 投影绑定到明确 generation，而不是每个 UI snapshot 重开和重扫。

## 未通过项

尚缺当前源码 Windows `zircon_editor` 聚焦测试、1/100/1000 scene/asset/view 规模计数、1k pointer/typing/layout storm、idle/interaction WPR、锁等待/持有、clone bytes、current-source cold/warm RenderDoc capture。静态修复均不得在这些门禁完成前写入 `review.md`。
