---
related_code:
  - zircon_editor/src/ui/host
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/07/failure-2026-07-17-ui-asset-editor-full-projection-and-import-rehydrate.md
  - docs/plans/zircon_editor/editor/09/failure-2026-07-17-ui-asset-watcher-unbounded-refresh.md
  - docs/plans/zircon_editor/editor/10/failure-2026-07-17-project-open-repeated-manager-scan.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-layout-metadata-full-recompute-and-sync-io.md
reference_sources:
  - dev/slint/internal/backends/winit/accesskit.rs
  - dev/slint/internal/core/model/repeater.rs
  - dev/godot/editor/export/editor_export_platform.cpp
tests:
  - editor UI host source guards listed below
  - current-source Windows zircon_editor focused tests pending
  - editor idle/interaction/WPR and watcher storm pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor UI host 逐文件性能静态审查（2026-07-17）

## 范围与当前覆盖

当前工作树 `zircon_editor/src/ui/host` 共 **250** 个 Rust 文件，已逐文件阅读 **250/250**。静态覆盖已经完成，但这不是目录验收；全部当前源码动态门禁完成前，该目录继续留在 `pending.md`。

| 模块 | 已读/总数 | 静态状态 | 主要结论 |
|---|---:|---|---|
| startup | 7/7 | 已读 | recent summary 复用已修；完整 open 的重复 manager/scan 已移交 Editor10 |
| host root | 37/37 | 已读 | keymap per-event parse、capability linear lookup 已修；event full reflection、project/plugin/export generation 缺失已移交 |
| editor_event_execution | 13/13 | 已读 | dispatch 完成后同步 reflection；未发现可独立于 PERF-076 的更低层简单修复 |
| layout_hosts | 9/9 | 已读 | focus 也 full metadata recompute、page switch 同步配置 I/O，已移交 EditorUI08 |
| builtin_layout | 6/6 | 已读 | 主要为声明/装配；动态频率随 layout generation 一并验证 |
| builtin_views | 29/29 | 已读 | 多数为 projection consumer；根因在 host reflection/asset projection generation |
| asset_editor_sessions | 19/19 | 已读 | 4 个低风险 allocation/no-op 修复；全 projection/import rehydrate 与 unbounded watcher 分别移交 Editor07/09 |
| animation_editor_sessions | 5/5 | 已读 | no-op mutation metadata/route serialization 已修；更大 pane projection 受 PERF-076 domain generation 约束 |
| editor_manager_asset_editor | 5/5 | 已读 | 纯 host facade；未新增独立热路径 owner |
| native_dynamic_export_preparation | 9/9 | 已读 | per-package discovered 线性扫描已修；warm 全量 staging 已补充移交 Editor15 |
| export_process_support | 5/5 | 已读 | 单次 reader 有 64 KiB yield，但上层仍全量累计；由 PERF-080 的 bounded tail/stream artifact 统一处理 |
| editor_asset_manager | 36/36 | 已读 | graph/folder/atlas 临时深复制已修；全 catalog refresh/snapshot、preview 锁内 I/O 与无界 change bus 已移交 Editor09 |
| editor_manager_plugins_export | 70/70 | 已读 | load-state 临时 Vec 已修；只读 status 重载动态库与 export wizard O(L²) snapshot/log/full UI projection 已分别补充移交 Editor12/15 |

## 已直接优化

- `EditorCapabilitySnapshot` 对有序 enabled rows 使用二分查找。
- `EditorHostEventController` 构造时解析并持有 keymap；每个 keyboard event 直接复用。
- UI asset instance 同步每次只生成一个 reflection model；changed-import set 用 borrowed `str` 查询。
- `.zui` 判断不再 lowercase 整条 id；palette drag 未改变文档时不重建 instance projection。
- animation mutation no-op 立即返回，真实变化不再序列化稳定 route payload；native preparation 一次建立 discovery id index。
- asset reference graph 借用 records；folder segment 流式遍历且 comparator 借用显示名；sprite atlas PNG 直接编码 borrowed RGBA slice。
- native load-state 用 streaming checks，不再物化 loaded rows。

对应源码 RED→GREEN 守卫：

- `enabled_capability_lookup_uses_the_sorted_snapshot`
- `keyboard_dispatch_reuses_the_controller_keymap`
- `syncing_instance_builds_one_reflection_model`
- `changed_import_lookup_borrows_the_normalized_asset_id`
- `zui_suffix_check_does_not_lowercase_the_whole_asset_id`
- `unchanged_palette_drag_does_not_rebuild_instance_projection`
- `animation_mutation_skips_noops_and_does_not_reserialize_the_stable_route`
- `native_package_preparation_indexes_discovery_once`
- `reference_graph_borrows_catalog_records_during_rebuild`
- `folder_sort_borrows_display_names`
- `sprite_atlas_artifact_writer_does_not_clone_the_full_rgba_payload`
- `native_load_state_streams_loaded_plugin_checks`

## 已移交的架构热点

- PERF-MVP-075：一次 project open 被 authority/runtime/editor/locator 多次 parse/scan。
- PERF-MVP-076：每 editor event 都同步 full reflection，dirty domain 与帧级 coalescing 未生效。
- PERF-MVP-077：focus/layout mutation 全量 metadata recompute，page switch 同步读写配置。
- PERF-MVP-078/079：asset/plugin generation projection 缺失，单次查询或逐插件注册反复重放/深复制。
- PERF-MVP-080：export Cargo output 全量驻留内存。
- PERF-MVP-082/083：UI asset 每 mutation 全量 projection/import hydrate；watcher unbounded 且主线程 drain/读盘/解析无预算。
- PERF-MVP-086：native dynamic warm export 无条件删除并重复制整个 staging tree。
- PERF-MVP-088：editor asset full refresh/snapshot、preview write-lock I/O 与 unbounded change bus。
- PERF-MVP-090：只读 native status 重复 discovery/dynamic load，enablement/manifest/catalog 重建。
- PERF-MVP-091：export output 每行 full snapshot、无界 channel/log、无预算 poll 与 full retained projection。

failure graph 已导入到 node count 226；唯一 diagnostic 仍是既有且无关的 tooling `maintenance-held-cpu-reservation-consumption-gap` origin workflow metadata。

## 参考实现

Slint AccessKit 只在全局或 node `PropertyTracker` dirty 时重新 evaluate cached node；repeater 对单 row instance 更新或标 dirty。这支持 Zircon 把 editor reflection 与 UI asset authoring 按 domain/generation 增量发布，而不是给每个输入事件建立完整 snapshot。Godot export `FileExportCache` 则为 source 保存 mtime、MD5 与输出路径，说明 warm export 应拥有跨 stage 的 inventory/cache authority。

## 未通过项

尚缺当前源码 Cargo、打开项目/键盘/指针/typing 产品 trace、idle WPR、reflection build-count、watcher storm budget、plugin status discovery/load-count 和 warm export bytes-read/RSS/output-storm。当前证据不得用于把 UI host 行从 `pending.md` 移入 `review.md`。
