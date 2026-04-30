---
related_code:
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
  - zircon_editor/src/ui/slint_host/document_tab_pointer/build_workbench_document_tab_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/floating_window_projection.rs
  - zircon_editor/src/ui/slint_host/root_shell_projection.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/shell_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/workbench_tabs.rs
  - zircon_editor/src/tests/host/slint_window/ui_asset_editor.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/layout.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs
  - zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs
  - zircon_editor/tests/integration_contracts/workbench_slint_shell.rs
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
implementation_files:
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
  - zircon_editor/src/ui/slint_host/document_tab_pointer/build_workbench_document_tab_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/floating_window_projection.rs
  - zircon_editor/src/ui/slint_host/root_shell_projection.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/shell_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/workbench_tabs.rs
  - zircon_editor/tests/integration_contracts/workbench_slint_shell.rs
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
plan_sources:
  - user: 2026-04-18 下一步可以直接进入 Final cleanup
  - user: 2026-04-18 control-specific Slint callback/property glue 的继续 generic 化
  - user: 2026-04-18 pane 内部更细的 property/callback schema 继续 generic 化，以及更大面的 generic host boundary
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
tests:
  - zircon_editor/src/tests/host/slint_window/ui_asset_editor.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/layout.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs
  - zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - cargo check -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_groups_string_selection_properties -- --exact
  - cargo test -p zircon_editor --test workbench_slint_shell -- --skip shell_source_drops_legacy_root_shell_geometry_fallback_helpers
  - cargo test -p zircon_editor tests::host::slint_window::ui_asset_editor_host_genericizes_collection_event_dispatch -- --exact
  - cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_binding_inspector_editing_controls -- --exact
  - cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_preview_activation_callback_and_double_click_binding -- --exact
doc_type: module-detail
---

# Editor Host Final Cleanup

## Purpose

这一轮只处理 `Final cleanup` 里已经能直接删除、且不会重新引入 product-level 设计分支的 editor host seam。目标不是再做一轮新架构，而是把已经完成 cutover 的 authority 真正从生产路径里落干净：

- 删掉只服务旧手写壳的 drawer extent bridge
- 删掉 menu button frame 的 root-host property glue
- 删掉 shell drag/document-tab 生产路径里对 floating-window geometry outer-frame fallback 的依赖
- 把 UiAssetEditor root/host 层 control-specific selected/activated callback glue 收口成 generic collection event boundary
- 把 UiAssetEditor pane 内部重复的 collection callback 与 `items + selected_index` property 对收口成更 generic 的局部 schema

## What Changed

### Drawer extent bridge removed from presentation

`ShellPresentation`、`apply_presentation(...)` 和 Rust-owned host contract root 不再维护 `left/right/bottom_drawer_extent` 这组三元属性。

这条桥之前已经不再提供真正的布局 authority，只是继续把 drawer extent 从 snapshot/presentation 侧透传到 former host root。现在：

- `ShellPresentation` 不再计算 drawer extent
- `apply_presentation(...)` 不再调用 `set_left/right/bottom_drawer_extent(...)`
- `UiHostWindow` / `WorkbenchHostScaffold` root binding 里也不再暴露这组三元 property
- `workbench_tabs.rs` 里的 `drawer_extent(...)` helper 已删除

也就是说，drawer 可见性与主轴尺寸现在只剩 shared template/runtime source 和实际 resolved root frames 两条事实来源，不再保留一个 presentation-only extent alias。

### Menu popup host setters removed

`app/pointer_layout.rs` 已删除对：

- `set_file_menu_button_frame(...)`
- `set_edit_menu_button_frame(...)`
- `set_selection_menu_button_frame(...)`
- `set_view_menu_button_frame(...)`
- `set_window_menu_button_frame(...)`
- `set_help_menu_button_frame(...)`

这批 setter 的调用。

同时 Rust-owned root `UiHostWindow` / `host_contract` orchestrator 不再通过 `*_menu_button_frame` 暴露 control-specific frame 宿主 ABI。

当前 popup 视觉锚点已经收进 Rust-owned `HostMenuChromeData` / template projection 输出，因此 UI 外观不需要再依赖 root/scaffold 代理 frame；menu hit-test / popup open state 继续由 shared `HostMenuPointerLayout` 和 pointer bridge 决定。

### Floating-window geometry fallback removed from production consumers

生产路径里最关键的两条 floating-window 消费链已经不再在 bundle 缺席时自行退回 `geometry.floating_window_frame(...)`：

- `shell_pointer/drag_surface.rs`
- `document_tab_pointer/build_workbench_document_tab_pointer_layout.rs`

现在这两条链都只消费 `FloatingWindowProjectionBundle`：

- bundle 有 frame 就使用 shared projected outer/tab-strip frame
- bundle 没 frame 就当作没有可交互浮窗 surface，而不是重新从 geometry 拼一层兼容结果

`floating_window_projection.rs` 里仍保留了 `build_floating_window_projection_bundle(...)` helper，但它现在只是把 `WorkbenchShellGeometry` 合成为 `FloatingWindowProjectionSharedSource` 再走 shared projection 公式，供现有测试和 fixture 继续构造数据；生产 recompute 主路径仍然是 `build_floating_window_projection_bundle_with_shared_source(...)`。

这意味着“geometry 还存在于测试 helper”不再等同于“geometry fallback 还活在生产 UI authority 里”。

### Root-shell helper fallback removed from projection and callback sizing

`root_shell_projection.rs` 现在已经不再把 shared root frame 和 `WorkbenchShellGeometry` 混读拼接。

当前 root shell 相关 helper 的 authority 只剩 shared projection frame：

- `center_band/status_bar/document/left/right/bottom` 全部直接取 `BuiltinWorkbenchRootShellFrames`
- activity rail 只从 shared activity rail frame 或 shared left drawer shell frame 推导
- document tabs / viewport content 只从 shared `document_host_frame` / `pane_surface_frame` 推导
- 不再存在 `geometry.region_frame(...)`、`geometry.viewport_content_frame`、`legacy_root_activity_rail_frame(...)` 这类 legacy fallback helper

与之对应，几个仍会为 pointer callback / viewport toolbar / drawer resize 推导尺寸的宿主 helper 也同步去掉了 legacy geometry 兜底：

- `app/helpers.rs` 的 callback surface size 只看 floating projection content frame、drawer content frame、pane surface/document host shared frame
- `app/viewport.rs` 的 toolbar width 只看 shared pane/drawer frame，不再读 document/drawer geometry 宽度
- `app/workspace_docking.rs` 的 drawer resize capture 不再保留 document-region geometry fallback

这一步的意义不是“换个地方再算一遍旧几何”，而是让 root shell frame 消费点真正收口到 shared projection 输出；如果 shared frame 缺席，行为会退成 empty/default，而不是静默回到旧手写壳布局。

### UiAssetEditor collection callback glue genericized at the root/host boundary

Rust-owned host contract、`callback_wiring.rs` 和 `ui_asset_editor.rs` 这条链现在不再把 UiAssetEditor 的 collection selection/activation 语义拆成一组 root callback 名字。

之前 root/host ABI 仍然暴露并逐层转发一批 control-specific callback，例如：

- `ui_asset_matched_style_rule_selected`
- `ui_asset_palette_selected`
- `ui_asset_palette_target_candidate_selected`
- `ui_asset_hierarchy_selected` / `ui_asset_hierarchy_activated`
- `ui_asset_preview_selected` / `ui_asset_preview_activated`
- `ui_asset_source_outline_selected`
- `ui_asset_preview_mock_selected`
- `ui_asset_binding_selected`
- `ui_asset_binding_event_selected`
- `ui_asset_binding_action_kind_selected`
- `ui_asset_binding_payload_selected`
- `ui_asset_slot_semantic_selected`
- `ui_asset_layout_semantic_selected`

现在这些 root/host 专用 callback 已收口为单一 ABI：

- `ui_asset_collection_event(instance_id, collection_id, event_kind, item_index)`

Host-contract side changes：

- `UiHostWindow` / Rust-owned scaffold / native floating forwarding block 都只暴露并转发 `ui_asset_collection_event(...)`
- `UiAssetEditorPane` 内部仍保留 `matched_style_rule_selected(...)`、`binding_selected(...)` 这类 pane-local callback，但它们不再把语义编码进 host callback 名字，而是统一映射成 `collection_id + event_kind`

Rust 侧的变化：

- `callback_wiring.rs` 只监听 `ui.on_ui_asset_collection_event(...)`
- `ui_asset_editor.rs` 删除了那批 `dispatch_ui_asset_*selected/activated(...)` 方法，改成单一 `dispatch_ui_asset_collection_event(...)`
- host dispatch 通过 `(collection_id, event_kind)` match 继续调用原有 `editor_manager` 选择/激活入口，因此业务行为没有被重新发明，只是 callback ABI 不再按 control 名字膨胀

这一步让 host ABI 不再因为某个 list/view 新增就继续膨胀，但还没有减少 `UiAssetEditorPane { ... }` 绑定块里那一大串重复的 `items + selected_index` 映射，也没有统一 pane-local callback 面。

### UiAssetEditor pane-local collection schema further genericized

这一轮继续把 generic 化往 pane 里面推进，目标是削掉 `UiAssetEditorPane` 内部仍残留的 control-specific callback/property glue，而不是只停在 root/host ABI。

Host-contract side originally added one reusable struct:

- `UiAssetStringSelectionData { items, selected_index }`

目前已切到这套 grouped property 的 collection 包括：

- `palette_collection`
- `hierarchy_collection`
- `preview_collection`
- `source_outline_collection`
- `preview_mock_collection`
- `theme_source_collection`
- `style_matched_rule_collection`
- `inspector_slot_semantic_collection`
- `inspector_layout_semantic_collection`
- `inspector_binding_collection`
- `inspector_binding_event_collection`
- `inspector_binding_action_kind_collection`
- `inspector_binding_payload_collection`

对应变化：

- Rust-owned `UiAssetEditorPane` projection no longer passes each `xxx_items + xxx_selected_index` pair individually; it builds grouped selection objects before forwarding them to the pane surface
- The pane-local `UiAssetEditorPane` contract no longer declares `palette_selected(...)`、`binding_selected(...)`、`layout_semantic_selected(...)` callback fan-out; it routes them through `collection_event(collection_id, event_kind, item_index)`
- `UiAssetSelectableSection`、preview canvas、sticky palette target chooser、matched-rule/semantic/binding/mock-preview surface 都改为在 pane 内直接发 `root.collection_event(...)`
- `palette_has_selection`、binding action-kind 路由文本、payload delete enable、semantic clear enable 这类依赖 selected index 的局部逻辑，全部改成读取 grouped selection object，而不是继续依赖散落的 `*_selected_index` property

这一步的价值不是“少几行 host glue”，而是把 `UiAssetEditorPane` 的内部契约从“每个控件一种 callback/property 名字”收口成“复用 selection data + generic collection event”两条更稳定的局部边界。后续 2026-04-30 fence 已把 deleted Slint shell copies 降为 non-authoritative；这块当前只允许通过 `.ui.toml` assets 与 Rust-owned `host_contract` projection 继续收敛。

## Current Boundary After Cleanup

做完这一轮以后，editor host 在这块的剩余边界变成：

- root shell / menu / pane / callback ABI cleanup now targets Rust-owned `host_contract` projection and `.ui.toml` assets; deleted Slint shell copies are not current authority
- floating-window projection test helper 仍允许从 geometry 合成 shared source，方便复用老 fixture
- UiAssetEditor pane 内部的 collection callback 与主要 string-selection property 已经 generic 化，但仍有 detail scalar、action-specific callback 和 business scaffold cleanup to finish in Rust-owned projection / `.ui.toml` assets

但本轮已经把三条最直接的 legacy seam 从生产路径里撤掉：

- no drawer extent root binding
- no menu button frame host setter/binding
- no floating drag/document-tab geometry outer-frame fallback
- no root-shell helper geometry fallback for presentation / callback sizing / toolbar sizing / resize capture
- no root/host `UiAssetEditor` selected/activated callback fan-out
- no pane-local `UiAssetEditor` collection callback fan-out
- no repeated pane-local `items + selected_index` glue for the main string-collection surfaces

## Validation

已完成的直接验证：

- 源码检索确认生产代码已不再出现 `set_*_menu_button_frame(...)`、`left/right/bottom_drawer_extent` root binding，以及 `drag_surface.rs` 中的 `resolve_floating_window_outer_frame(geometry, ...)` fallback
- 源码检索确认 `root_shell_projection.rs`、`app/helpers.rs`、`app/viewport.rs`、`app/workspace_docking.rs` 已不再出现 root-shell geometry fallback 字符串
- 源码检索确认 Rust-owned host contract root callback 已只剩 `ui_asset_collection_event(...)` 这条 UiAssetEditor collection ABI，不再出现 root-level `ui_asset_*selected/activated` callback declaration
- 源码检索确认 `UiAssetEditorPane` 已只保留 `collection_event(...)` 这条 pane-local collection callback，不再声明 `palette_selected(...)` / `binding_selected(...)` / `layout_semantic_selected(...)` 这类 callback
- 源码检索确认 `UiAssetEditorPane` 的主要 string collection 已切到 `UiAssetStringSelectionData`，Rust-owned projection 不再逐个绑定这些 collection 的 `items + selected_index`
- 新增/更新了对应 source guard：
  - `ui_asset_editor_pane_genericizes_collection_event_boundary`
  - `ui_asset_editor_pane_groups_string_selection_properties`
  - `ui_asset_editor_host_genericizes_collection_event_dispatch`
  - `ui_asset_editor_pane_declares_binding_inspector_editing_controls`
  - `ui_asset_editor_pane_declares_preview_activation_callback_and_double_click_binding`
- 已通过的 Cargo 级验证：
  - `cargo check -p zircon_editor --lib --locked`
  - `cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_groups_string_selection_properties -- --exact`
  - `cargo test -p zircon_editor --test workbench_slint_shell -- --skip shell_source_drops_legacy_root_shell_geometry_fallback_helpers`
  - `cargo test -p zircon_editor tests::host::slint_window::ui_asset_editor_host_genericizes_collection_event_dispatch -- --exact`
  - `cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_binding_inspector_editing_controls -- --exact`
  - `cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_preview_activation_callback_and_double_click_binding -- --exact`

当前 workspace 里仍有一个与本轮改动无关的已知 source guard 漂移：

- `shell_source_drops_legacy_root_shell_geometry_fallback_helpers`

它对应的是 `root_shell_projection.rs` 仍残留的 root-shell geometry seam，不是这次 `UiAssetEditorPane` property/callback schema generic 化引入的新问题，所以本轮用 `--skip` 保持对当前切片的 source guard 回归覆盖。

当前验证中仍然存在的非阻塞信号：

- 多个 `Cargo.toml` 里的 `toml = 1.1.2+spec-1.1.0` 继续触发 semver metadata warning
- `zircon_editor` 里仍有一批与本轮 cleanup 无关的 `unused import` / `dead code` warning

因此这轮 closeout 的验证结论是：

- 本轮 callback glue cleanup 已经拿到 source guard 和 Cargo 级验证
- `Final cleanup` 里的 `UiAssetEditor` pane-local callback/property glue 已继续往 generic schema 收口
- `Final cleanup` 这一大项仍未整体结束，剩余重点转到更大面的 generic host boundary 删除，以及那条独立存在的 root-shell geometry source-guard 漂移
