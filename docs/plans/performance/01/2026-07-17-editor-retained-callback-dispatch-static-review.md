---
related_code:
  - zircon_editor/src/ui/retained_host/callback_dispatch
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01/failure-2026-07-17-retained-control-index-and-virtual-row-sync.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/slint/internal/core/model/repeater.rs
tests:
  - builtin_template_binding_registry_is_process_cached
  - shared_viewport_pointer_bridge_skips_unchanged_frame_rebuilds
  - shared_viewport_surface_reuses_one_pointer_dispatcher
  - virtual_row_growth_indexes_existing_rows_and_node_ids_once
  - extension_navigation_uses_one_process_action_index
  - builtin_floating_window_source_template_bridge_reuses_surface_across_layout_recompute
  - responsive_tier_parsing_avoids_per_node_lowercase_allocation
  - current-source Windows zircon_editor focused Cargo pending
  - pointer/action/data-sync storm and control-index scaling pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained callback dispatch 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

当前工作树 `zircon_editor/src/ui/retained_host/callback_dispatch` 共 **135** 个 Rust 文件、**16,687** 行，已逐文件阅读 **135/135**。范围包括 root dispatch、asset/common/hierarchy/inspector/layout/pane/shared-pointer/viewport/welcome/workbench callback，以及 template bridge 的 asset、floating、inspector、pane、viewport toolbar、welcome 和完整 Workbench extension/navigation/feedback/data-sync 路径。

动态 Cargo、真实 retained editor pointer/action storm、1/100/10k row scaling 和 current-source WPR 尚未完成，因此本目录只在 `pending.md` 标记静态覆盖，不进入 `review.md`。

## 已直接优化

- PERF-MVP-124：builtin template binding 注册表原来每次 dispatch 都重建 659 行 `BTreeMap<String, EditorUiBinding>`。现由进程级 `LazyLock` 构建一次，lookup 后只 clone 最终 binding；runtime bootstrap 仍把 session-owned binding 注册到 runtime。
- PERF-MVP-125：shared viewport bridge 现在持有一次构建的 `UiPointerDispatcher`，不再每 pointer event 重注册；viewport frame 保留 committed 值，同 frame update 不执行 surface rebuild。
- PERF-MVP-126：virtual row growth 一次收集现有 row number，一次取得 next node id 后单调递增，删除了每个缺失 row 的两次全树扫描。
- PERF-MVP-127：44 个 extension workspace 的静态 specs 一次汇总为 process action index；workspace/tab/row/command/field query 不再逐 callback 扫描 specs 与 action slices。
- PERF-MVP-129：floating source bridge 对同 shell size 明确 no-op；responsive tier parse 用 `eq_ignore_ascii_case`，不再为每个 responsive node 分配 lowercase String。

这些修复已有源码 RED→GREEN、`rustfmt` 与 `git diff --check` 证据，但在协调器 Cargo 结果返回前仍是动态未验收。

## 共享架构瓶颈

静态扫描在该目录仍找到 10 处直接 `tree.nodes.values().find_map`、22 个 control-id lookup 调用。`popup_state`、`window_menu_state`、property/transform edit、component property rows 与 data sync 会为一次动作的多个字段分别从头扫描 surface；按 row 同步时，调用次数乘以 row 数形成平方级访问。virtual-row 局部修复只消除了创建阶段的明显双扫描，不能替代共享索引。

Workbench slow recompute 还会先重建 root/workbench surface，再在 drawer/responsive/data sync 后执行 template layout；focus、drawer toggle、viewport toolbar route 等 callback 会为单个 identity/settings 读取或 clone 大型 chrome/layout/model snapshot。若只在各 bridge 私建 map，会在动态 row insert/prune、template reload 与 surface generation replacement 后产生失效的第二权威。

PERF-MVP-128 因此要求：

- `UiSurface` 或其稳定 owner 维护 generation-owned `control_id -> node id(s)` 索引，insert/detach/reuse/reload 同步更新；duplicate id 和 descendant traversal 有明确语义。
- row/property sync 输入 typed delta，只 patch changed rows/properties；同 generation 不重复 full control lookup、TOML/string materialization 或 layout。
- root/workbench/template layout 每 dirty generation 每帧至多一次；floating/popup/toolbar 只消费已提交的 geometry/snapshot handle。
- 1/100/10k controls/rows 记录 lookup visited nodes、allocations、layout count、interaction p95 与最终视觉/route byte parity。

该项已写入 EditorUI01 failure；layout/coalescing/snapshot部分补充到 EditorUI08 failure。

## 参考实现约束

Bevy `UiSurface` 使用 `EntityHashMap<LayoutNode>` 保存 entity 到 Taffy node 的稳定映射，并在 upsert/remove/update children 时维护 owner index，不要求每次属性访问遍历整棵布局树。Slint `RepeaterTracker` 对 `row_changed/row_added/row_removed` 更新可见实例和 dirty state，`ensure_updated` 在未 dirty 时直接返回。Zircon 不照搬其类型，但应采用同样的稳定身份索引、结构变更维护和 row delta，而不是 callback 私有缓存或每事件全树物化。

## 待动态验收

协调器下运行 focused binding/viewport/template-bridge tests 与 `zircon_editor --lib performance_tests`；随后执行 known/unknown action storm、same-frame viewport sync、popup/menu control lookup、1/100/10k virtual rows、同尺寸/resize floating source 和 responsive node allocation trace。必须记录 build/scan/layout 次数与交互 p95，不能只用 wall-clock 或源码守卫把目录移入 `review.md`。
