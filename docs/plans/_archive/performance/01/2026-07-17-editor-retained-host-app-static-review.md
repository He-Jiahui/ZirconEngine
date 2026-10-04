---
related_code:
  - zircon_editor/src/ui/retained_host/app
  - zircon_editor/src/ui/retained_host/scroll_surface_host.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-viewport-toolbar-surface-rebuild-storm.md
  - docs/plans/zircon_editor/editor/09/failure-2026-07-17-editor-asset-catalog-full-rebuild-and-preview-lock.md
  - docs/plans/zircon_editor/editor/12/failure-2026-07-17-editor-plugin-catalog-rebuild-and-deep-copy.md
  - docs/plans/zircon_editor/editor/15/failure-2026-07-17-export-overlapping-recursive-digests.md
reference_sources:
  - dev/slint/internal/core/properties.rs
  - dev/slint/internal/core/model/repeater.rs
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/godot/editor/editor_node.cpp
tests:
  - retained-host performance source/behavior guards listed below
  - Windows cargo test -p zircon_editor --lib performance_tests running as coordinator job 0277e7ea70544ecfbd6fec3ea4a24a38
  - current-source editor idle/interaction/WPR and RenderDoc cold/warm capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Host App 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

当前工作树 `zircon_editor/src/ui/retained_host/app` 共 **451** 个 Rust 文件，已逐文件阅读 **451/451**。静态覆盖完成，但当前源码 Cargo 作业、产品交互 trace、WPR 与冷/暖 RenderDoc 门禁尚未全部完成，因此本模块继续留在 `pending.md`，不得进入 `review.md`。

| 压缩模块 | 已读/总数 | 主要结论 |
|---|---:|---|
| root / host_lifecycle / callback / helpers / invalidation / pointer_layout | 160/160 | 单一 slow recompute 串行覆盖全部 domain；render 成功会反馈置脏；callback 使用单线程 `Rc<RefCell>`，重工作必须帧级合并 |
| asset / hierarchy / inspector / detail pointer | 51/51 | pointer 完整 snapshot 与重复 layout 已直修；asset 三 channel 无界 drain、catalog/details/preview 全量刷新移交 Editor09 |
| build-export / module-plugin / profiling | 82/82 | 可见 pane 仍同步读 manifest/preset、逐 preset 建 plan、重建 plugin status；移交 Editor15/12；terminal wizard poll 已直修 |
| pane / native windows / viewport / welcome / docking | 68/68 | pane instance snapshot、close snapshot、locator compare、drag target publish 已直修；presentation/native-window/toolbar 仍需 generation patch |
| UI asset editor actions/detail events | 62/62 | palette hover 忽略底层 `changed` 的重投影已直修；其余 action 成功后普遍整 presentation 置脏，需返回 typed changed domain |
| showcase inputs / app integration tests | 28/28 | showcase action key 多次完整规范化为 P2 demo 成本；测试覆盖 child-window、pointer fallback、drag、toolbar 与 close prompt |

## 已直接优化

- Asset content/tree/reference pointer 复用慢路径提交的 `Arc<AssetWorkspaceSnapshot>`；hierarchy pointer 复用提交的 `Arc<[SceneEntry]>`。move/scroll 不再为一个命中目标构建完整 editor snapshot。
- Asset、hierarchy 与 detail-scroll callback 只有在实际 surface size 改变时才重建命中布局；Console、Inspector 与 Asset Details 的重复尺寸更新成为 no-op。
- UI asset/animation pane payload 只在对应 pane 可见时收集，并共享一次 `current_view_instances()`；main-window close 同样共享一次 instance snapshot。
- default-scene refresh 预解析一次 `ResourceLocator`，不再为每条 asset/editor/resource event 分配 locator 字符串。
- 重复 docking drag-target 不再回写 UI state；palette drag hover 只在底层返回 `changed=true` 时标记 presentation dirty。
- Export wizard 直接 `iter_mut` sessions，只为变化项 clone profile key，并跳过已 terminal session，删除每 tick keys Vec 与永久 terminal poll。

对应 RED→GREEN/行为守卫：

- `editor_pane_payloads_share_one_visible_instance_snapshot`
- `main_window_close_reuses_the_view_instance_snapshot`
- `default_scene_refresh_parses_the_locator_once`
- `pointer_snapshot_reuses_the_committed_asset_projection`
- `unchanged_pointer_sizes_do_not_rebuild_list_layouts`
- `hierarchy_pointer_reuses_the_committed_scene_projection`
- `repeated_surface_size_is_a_no_op`
- `asset_details_scroll_reuses_the_committed_browser_projection`
- `repeated_drag_target_group_does_not_republish_ui_state`
- `polling_streams_mutable_sessions_and_skips_terminal_snapshots`
- `unchanged_palette_drag_hover_does_not_dirty_presentation`

## P0/P1 架构热点

- PERF-MVP-103 / EditorUI08：`render_submission.rs` 在每次 `Ok(true)` 后无条件把 presentation 标脏，只为诊断统计可见，构成稳定帧反馈环。应改为独立 stats generation/consumer visibility。
- PERF-MVP-104 / Editor09：tick 对 asset/editor/resource 三条 channel 无预算 drain；任意事件可触发 manager full refresh、完整 editor/chrome snapshot、全 catalog sync 与 visible preview 重发。
- PERF-MVP-105 / EditorUI08：`active_activity_window_template_document_is` 为简单 control/edit/option gate 构建完整 chrome；Inspector drag/apply 也为一个对象读取完整 editor snapshot。
- PERF-MVP-106 / EditorUI08：任意 structural/presentation dirtiness 串行重建 chrome/model/geometry/template bridges、所有 payload/pointer/native presenters；viewport resize 在同次 recompute 二次构建 chrome/model，toolbar/world-space 又深 clone presentation/floating rows。
- PERF-MVP-107 / Editor15/12：Build/Export pane 每次可见 slow recompute 同步枚举 preset、逐文件 parse、逐 preset 重建 export plan；Module/Plugin pane 同步读 manifest/status，并可能触发 discovery/load/catalog rebuild。
- P2：showcase `action_matches` 每个候选都重新 split、camel-to-snake、collect/join 整个 action id；仅 demo 可达，不抢占 MVP lane。

## 参考实现约束

Slint 的 property tracker/repeater 以 dirty dependency 与 row generation 决定是否重算，支持 retained host 将 chrome、pane、pointer、native-window 与 diagnostics 拆成 generation-owned domain，而不是用单个 presentation dirty 位触发整链。Godot 的 editor file-system scan 将文件变化建模为独立扫描/更新阶段，支持 Zircon 把 asset/preset/plugin I/O 放到显式 generation 与 worker commit，而不是在 UI projection 中同步重扫。

## 未通过项

尚缺 coordinator Cargo 作业最终结果、完整 retained-host integration suites、1k pointer/asset-event/invalidation storm 的 build/queue/clone 计数、idle/interaction WPR、锁等待/持有、当前源码产品冷/暖 RenderDoc capture。以上动态门禁通过前，`zircon_editor/src/ui/retained_host/app` 只允许在 `pending.md` 标记“451/451 静态完成、动态待验收”。
