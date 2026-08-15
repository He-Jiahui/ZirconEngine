---
related_code:
  - zircon_editor/src/ui/host/
  - zircon_editor/src/core/
  - zircon_editor/src/ui/workbench/
implementation_files:
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_capabilities.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/sync.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/service.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/commit.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/palette.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/prepare.rs
  - zircon_editor/src/ui/host/editor_asset_manager/reference_graph.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/folders.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/artifact.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native_load_state.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/run.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_projection.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/07/failure-2026-07-17-ui-asset-editor-full-projection-and-import-rehydrate.md
  - docs/plans/zircon_editor/editor/09/failure-2026-07-17-ui-asset-watcher-unbounded-refresh.md
  - docs/plans/zircon_editor/editor/09/failure-2026-07-17-editor-asset-catalog-full-rebuild-and-preview-lock.md
  - docs/plans/zircon_editor/editor/10/failure-2026-07-17-project-open-repeated-manager-scan.md
  - docs/plans/zircon_editor/editor/12/failure-2026-07-17-editor-plugin-catalog-rebuild-and-deep-copy.md
  - docs/plans/zircon_editor/editor/15/failure-2026-07-17-export-overlapping-recursive-digests.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-layout-metadata-full-recompute-and-sync-io.md
reference_sources:
  - dev/slint/internal/backends/winit/accesskit.rs
  - dev/slint/internal/core/model/repeater.rs
  - dev/godot/editor/export/editor_export_platform.cpp
tests:
  - zircon_editor/src/ui/host/editor_capabilities.rs::tests::enabled_capability_lookup_uses_the_sorted_snapshot
  - zircon_editor/src/ui/host/editor_event_runtime_access/tests.rs::keyboard_dispatch_resolves_the_current_authority_keymap
  - zircon_editor/src/ui/host/asset_editor_sessions/mod.rs::tests::zui_suffix_check_does_not_lowercase_the_whole_asset_id
  - zircon_editor/src/ui/host/asset_editor_sessions/sync.rs::tests::syncing_instance_builds_one_reflection_model
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/tests.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/palette.rs::tests::unchanged_palette_drag_does_not_rebuild_instance_projection
  - zircon_editor/src/ui/host/animation_editor_sessions/editing.rs::performance_tests::animation_mutation_skips_noops_and_does_not_reserialize_the_stable_route
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/prepare.rs::performance_tests::native_package_preparation_indexes_discovery_once
  - zircon_editor/src/ui/host/editor_asset_manager/reference_graph.rs::performance_tests::reference_graph_borrows_catalog_records_during_rebuild
  - zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/folders.rs::performance_tests::folder_sort_borrows_display_names
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/artifact.rs::tests::sprite_atlas_artifact_writer_does_not_clone_the_full_rgba_payload
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native_load_state.rs::performance_tests::native_load_state_streams_loaded_plugin_checks
doc_type: module-detail
---

# Editor UI host

`ui::host` 是 retained workbench 与 headless editor core/runtime 之间的应用层 owner。它负责输入与命令路由、项目和资产会话、editor event 执行、布局 host、插件/导出协调，以及把 core 状态投影为 workbench snapshot。宿主可以调用 core service，但不应让每个指针、键盘、typing 或 focus 事件都重新解析静态配置、扫描项目、重放插件贡献或全量构建 UI 状态。

## 事件与投影契约

`EditorHostEventController` 在构造时冻结默认 keymap，并在 keyboard dispatch 中复用；稳定输入路径不得重新 TOML parse、排序或分配整份 binding 表。`EditorCapabilitySnapshot` 的 enabled 列表保持有序，单项检查用二分查找。未来 capability/keymap 动态替换应发布新的 generation snapshot，而不是把可替换锁重新放回每事件路径。

当前 `editor_event_runtime_reflection` 对任意 dirty mask 都会同步重建完整 reflection，并由 dispatch 在每个 event 后立即 drain。该行为保留为明确的未验收项：EditorUI08 应按 layout、selection、commands、assets、extensions、capabilities 等 domain 维护 generation 与 dirty projection，并在一帧内合并事件。构建期间不得持有 shell/command registry 锁，发布必须验证 generation，避免把旧快照覆盖到新状态。

## 项目、插件与导出边界

一次项目打开应产生可共享的 opened-project generation，供 authority、runtime asset manager、editor project document、workspace、watcher 和 locator 查询复用。每个资产 locator 不得重新创建 `ProjectManager` 或扫描 manifest。插件注册应在 batch staging 中一次建立 command、operation、asset 与 extension indexes；查询侧消费不可变 generation projection。导出 inventory、tool probe 与内容 digest 也应按 generation 缓存，不能为重叠 artifact tree 重复读盘。

这些跨模块优化由对应 failure handoff 承担；host 层禁止增加兼容 cache、隐藏的第二 authority 或仅对某个 UI 调用方绕过共享流程。

animation session 的稳定 asset route 在 restore/open 时已经进入 view instance；后续 mutation 只在真实变化时更新 title/dirty，不重新序列化 route。native dynamic preparation 在一次 export 中按 plugin id 建 borrowed discovery index；跨 export 的静态 staging 复用仍由 Editor15 的 generation inventory 统一处理。

## UI asset session

UI asset mutation 在成功改变文档后才同步 instance projection。一次同步只构建一次 reflection model；import change set 用 borrowed normalized id 查找；`.zui` 后缀使用 ASCII-insensitive byte comparison，避免复制整条资产路径。

细粒度 authoring mutation 的 typed domain delta 与 source typing debounce 仍未动态验收。Document/import 侧已经硬切为 physical-path generation cache：一个 watcher batch 跨受影响文档共享 read/parse 结果，diamond/cycle 在重复 I/O 前终止，同时保留 fragment alias、expected-kind validation 与诊断路径。

Watcher ingress 已按 normalized path 有界合并；`EditorJobSystem` worker 执行 read/hash/parse，主线程以 generation/baseline/fingerprint gate 提交，reverse dependency index 只选择受影响 session。Same-project save 保留 pending work，project-root cutover 才 cancel/reset；transient retry 与 exhausted/superseded 状态可观测。1k/10k storm p95、rename/remove、shutdown drain 和 source-bound Cargo 仍属于未完成验收合同，不能通过静默丢弃事件换取低延迟。

Editor asset catalog 同样需要独立于 UI asset document watcher 的 generation owner：runtime import delta 只重建受影响 record/reference/folder，preview decode/encode 与 meta I/O 在 worker 上完成，短锁提交前校验 source hash 和 catalog generation。catalog snapshot 与 change subscriber 不得把完整 DTO clone、无界队列或锁内 fanout 放回 retained reflection 热路径。

Plugin status 是只读 projection，不得调用 `load_discovered_all` 或触发 foreign entry；explicit refresh 发布 live-host/catalog generation 后，status、enablement、manifest completion 与 export plan 共享该 generation。export wizard 的输出事件只能携带 delta/共享 generation，UI 默认显示有界 tail；完整日志进入 artifact/分页 consumer，poll 和 retained row patch 均受主线程预算约束。

## 性能验收

静态源码守卫已经覆盖 keymap 复用、capability 二分查找、单次 reflection、borrowed import lookup、suffix 无分配和 no-op drag。完整通过仍要求当前源码 Windows Cargo 聚焦测试，以及产品级 open project、keyboard/pointer/typing storm、UI asset watcher storm、idle WPR 和 frame-coalesced reflection build-count 证据；完成前 `zircon_editor/src/ui/host` 必须继续留在 `docs/plans/performance/pending.md`。
