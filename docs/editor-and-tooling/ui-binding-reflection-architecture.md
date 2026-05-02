---
related_code:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/binding/model/mod.rs
  - zircon_ui/src/event_ui/manager/mod.rs
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/layout/scroll.rs
  - zircon_ui/src/layout/virtualization.rs
  - zircon_ui/src/dispatch/mod.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/hit_test.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_editor/src/ui/binding/mod.rs
  - zircon_editor/src/ui/binding/core/payload.rs
  - zircon_editor/src/ui/binding/core/editor_ui_binding_conversion.rs
  - zircon_editor/src/ui/binding/core/payload_codec.rs
  - zircon_editor/src/ui/binding/core/payload_constructors.rs
  - zircon_editor/src/ui/binding/asset/mod.rs
  - zircon_editor/src/ui/binding/dock/mod.rs
  - zircon_editor/src/ui/binding/dock/command.rs
  - zircon_editor/src/ui/binding/draft/mod.rs
  - zircon_editor/src/ui/binding/draft/command.rs
  - zircon_editor/src/ui/binding/selection/mod.rs
  - zircon_editor/src/ui/binding/viewport/mod.rs
  - zircon_editor/src/ui/binding/viewport/command.rs
  - zircon_editor/src/ui/binding/viewport/codec.rs
  - zircon_editor/src/ui/binding/welcome/mod.rs
  - zircon_editor/src/ui/control.rs
  - zircon_editor/src/ui/reflection.rs
  - zircon_editor/src/tests/ui/binding/mod.rs
  - zircon_editor/src/tests/ui/binding/animation.rs
  - zircon_editor/src/tests/ui/binding/asset_selection.rs
  - zircon_editor/src/tests/ui/binding/dock_and_welcome.rs
  - zircon_editor/src/tests/ui/binding/inspector_and_draft.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/ui/control/mod.rs
  - zircon_editor/src/tests/ui/control/activity_descriptors.rs
  - zircon_editor/src/tests/ui/control/reflection_projection.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/core/editor_event/mod.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/editor_event/runtime.rs
  - zircon_editor/src/core/editor_event/runtime/editor_event_runtime.rs
  - zircon_editor/src/core/editor_event/listener.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_event/journal.rs
  - zircon_editor/src/core/editor_event/replay.rs
  - zircon_editor/src/core/editor_event/inspector_field_change.rs
  - zircon_editor/src/core/editor_event/selection_host_event.rs
  - zircon_editor/src/core/editor_event/workbench/mod.rs
  - zircon_editor/src/core/editor_event/workbench/layout_command.rs
  - zircon_editor/src/core/editor_event/workbench/menu_action.rs
  - zircon_editor/src/core/editor_event/runtime/editor_event_runtime_inner.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_editor/src/ui/workbench/reflection/transient_ui_state.rs
  - zircon_editor/src/ui/workbench/event/core_event_conversion.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/app/tests.rs
  - zircon_editor/src/ui/slint_host/document_tab_pointer/build_host_document_tab_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/root_shell_projection.rs
  - zircon_editor/src/ui/slint_host/viewport/poll_image.rs
  - zircon_editor/src/ui/slint_host/viewport/tests/controller_polls_latest_captured_frame_from_render_framework.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/pointer_dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/pane/surface_control.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/event_bridge.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/editor_event_listener_control.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/layout_hosts/mod.rs
  - zircon_editor/src/ui/host/builtin_views/mod.rs
  - zircon_ui/src/template/document.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/asset_editor/source/source_sync.rs
  - zircon_editor/src/ui/asset_editor/tree/tree_editing.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs
  - zircon_editor/src/ui/asset_editor/style/inspector_semantics.rs
  - zircon_editor/src/ui/asset_editor/command.rs
  - zircon_editor/src/ui/asset_editor/undo_stack.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/mod.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state.rs
  - zircon_editor/src/tests/editing/ui_asset/
  - zircon_editor/src/tests/host/slint_window/callback_source_window.rs
  - zircon_editor/src/ui/workbench/event/mod.rs
  - zircon_editor/src/ui/workbench/event/editor_operation_binding.rs
  - zircon_editor/src/ui/workbench/event/dispatch_editor_host_binding.rs
  - zircon_editor/src/ui/workbench/event/editor_host_event.rs
  - zircon_editor/src/ui/workbench/event/editor_host_event_error.rs
  - zircon_editor/src/ui/workbench/model/mod.rs
  - zircon_editor/src/ui/workbench/model/menu/extension_menu.rs
  - zircon_editor/src/ui/workbench/model/menu/default_menu_bar.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/assets/ui/editor/assets_activity.ui.toml
  - zircon_editor/assets/ui/editor/asset_browser.ui.toml
  - zircon_editor/assets/ui/editor/component_showcase.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_editor/ui/workbench/template_node_data.slint
  - zircon_editor/ui/workbench/template_pane.slint
implementation_files:
  - zircon_ui/src/binding/model/mod.rs
  - zircon_ui/src/event_ui/manager/mod.rs
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/layout/scroll.rs
  - zircon_ui/src/layout/virtualization.rs
  - zircon_ui/src/dispatch/mod.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/hit_test.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_editor/src/ui/binding/mod.rs
  - zircon_editor/src/ui/binding/core/payload.rs
  - zircon_editor/src/ui/binding/core/editor_ui_binding_conversion.rs
  - zircon_editor/src/ui/binding/core/payload_codec.rs
  - zircon_editor/src/ui/binding/core/payload_constructors.rs
  - zircon_editor/src/ui/binding/dock/command.rs
  - zircon_editor/src/ui/binding/dock/codec.rs
  - zircon_editor/src/ui/binding/draft/command.rs
  - zircon_editor/src/ui/binding/draft/codec.rs
  - zircon_editor/src/ui/binding/viewport/command.rs
  - zircon_editor/src/ui/binding/viewport/codec.rs
  - zircon_editor/src/ui/control.rs
  - zircon_editor/src/ui/reflection.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/core/editor_event/mod.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/editor_event/runtime.rs
  - zircon_editor/src/core/editor_event/listener.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_event/journal.rs
  - zircon_editor/src/core/editor_event/replay.rs
  - zircon_editor/src/core/editor_event/inspector_field_change.rs
  - zircon_editor/src/core/editor_event/selection_host_event.rs
  - zircon_editor/src/core/editor_event/workbench/mod.rs
  - zircon_editor/src/core/editor_event/workbench/layout_command.rs
  - zircon_editor/src/core/editor_event/workbench/menu_action.rs
  - zircon_editor/src/core/editor_event/runtime/editor_event_runtime_inner.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_editor/src/ui/workbench/reflection/transient_ui_state.rs
  - zircon_editor/src/ui/workbench/event/core_event_conversion.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/event_bridge.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/editor_event_listener_control.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/layout_hosts/mod.rs
  - zircon_editor/src/ui/host/builtin_views/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/asset_editor/source/source_sync.rs
  - zircon_editor/src/ui/asset_editor/tree/tree_editing.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs
  - zircon_editor/src/ui/asset_editor/style/inspector_semantics.rs
  - zircon_editor/src/ui/asset_editor/command.rs
  - zircon_editor/src/ui/asset_editor/undo_stack.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/mod.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state.rs
  - zircon_editor/src/ui/workbench/event/mod.rs
  - zircon_editor/src/ui/workbench/event/editor_operation_binding.rs
  - zircon_editor/src/ui/workbench/event/dispatch_editor_host_binding.rs
  - zircon_editor/src/ui/workbench/event/editor_host_event.rs
  - zircon_editor/src/ui/workbench/event/editor_host_event_error.rs
  - zircon_editor/src/ui/workbench/model/mod.rs
  - zircon_editor/src/ui/workbench/model/build/workbench_view_model_build.rs
  - zircon_editor/src/ui/workbench/model/menu/extension_menu.rs
  - zircon_editor/src/ui/workbench/model/menu/default_menu_bar.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
  - zircon_editor/src/ui/workbench/reflection/model_build.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/assets/ui/editor/assets_activity.ui.toml
  - zircon_editor/assets/ui/editor/asset_browser.ui.toml
  - zircon_editor/assets/ui/editor/component_showcase.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_editor/ui/workbench/template_node_data.slint
  - zircon_editor/ui/workbench/template_pane.slint
plan_sources:
  - user: 2026-04-13 收束 JetBrains Hybrid Shell 的 UI 事件、反射和宿主契约
  - user: 2026-04-14 UI事件系统不应该直接和slint耦合，而是独立一套调度和绑定系统
  - user: 2026-04-14 实现运行时/编辑器共享 UI 布局与事件系统架构计划的首个共享 core 切片
  - user: 2026-04-15 继续实现 ScrollableBox、scroll state、visible range invalidation 和 pointer dispatcher
  - user: 2026-04-15 把 Container / Overlay / Space 落到 retained layout core，并把 editor host pointer/scroll 输入适配到 UiSurface + UiPointerDispatcher
  - user: 2026-04-15 继续把更完整的 editor shell pointer hit-test / dock target route 往 shared core 迁移
  - user: 2026-04-15 继续完善 shared navigation dispatcher，并把后续 editor keyboard/gamepad 入口固定到 shared core
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - .codex/plans/Editor Event Decoupling And Replay Plan.md
  - .codex/plans/Zircon UI Editor UI Binding & Reflection Architecture.md
  - user: 2026-04-17 Source/Hierarchy/Canvas 的更强选中同步和 source roundtrip 体验
  - user: 2026-04-17 parent-specific slot/layout inspector，补 Overlay/Grid/Flow/ScrollableBox 语义
  - user: 2026-04-17 designer canvas 的可视化 authoring：插入、重排、reparent、wrap/unwrap
  - user: 2026-04-17 Bindings Inspector 的下一版：事件枚举选择、action/payload 结构化编辑
  - user: 2026-04-26 Runtime/Editor 插件注册与 EditorOperation 设计计划
  - user: 2026-04-28 继续完成 Runtime/Editor 插件注册与 EditorOperation 统一派发闭环
  - user: 2026-04-28 继续接通 ui.toml/reflection EditorOperation binding 参数到 EditorOperationInvocation
  - user: 2026-04-28 继续收紧 EditorOperation `XXX.YYY.ZZZ` dotted path 命名契约
  - user: 2026-04-28 继续修复 Component Showcase Slint host retained row state 验证阻断
  - user: 2026-04-28 继续收束 EditorOperation undo/redo 命名历史栈
  - user: 2026-04-28 继续补齐 EditorOperationStack source 审计元数据
  - user: 2026-04-28 继续完善 EditorEventListener source/result 审计过滤
  - user: 2026-04-28 继续完善 EditorEventListener 未知 listener 控制错误
  - user: 2026-04-29 继续完善 EditorEventListener 单 listener 状态查询
  - user: 2026-04-29 继续实现 EditorOperationStack operation group 合并
  - user: 2026-05-02 继续完善 EditorOperation 失败审计的 operation group 传播
  - user: 2026-05-02 继续完善 EditorEventListener operation group 过滤
  - user: 2026-05-02 继续补齐编辑器 CLI operation group 调用入口
  - user: 2026-05-02 继续收紧 ComponentDrawer operation binding 注册校验
  - user: 2026-05-02 继续收紧扩展菜单 operation binding 注册校验
  - user: 2026-05-02 继续收紧扩展菜单路径注册校验
  - user: 2026-05-02 继续收紧编辑器 CLI operation 控制模式互斥校验
  - user: 2026-05-02 继续收紧扩展 View open operation 路径注册校验
  - user: 2026-05-02 继续收紧 EditorOperation menu_path 注册校验
  - user: 2026-05-02 继续收紧编辑器 CLI operation 控制模式 headless 显式校验
  - user: 2026-05-02 继续收紧编辑器 CLI operation 重复参数校验
  - user: 2026-05-02 继续收紧 EditorExtension 注册失败不污染 live operation registry
  - user: 2026-05-02 继续收紧 EditorExtension 跨插件贡献 id 冲突校验
  - user: 2026-04-17 Palette 到真实节点/引用节点创建的落地
  - user: 2026-04-17 结构化 undo/redo，从当前 source-text 级别继续往 tree-command 演进
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - user: 2026-04-21 继续执行 zircon_editor UI 回迁 + 树形 TOML cutover，清理 core 中残余 UI owner
tests:
  - cargo check -p zircon_editor --lib --locked --target-dir target\codex-editor-operation-check -q
  - cargo test -p zircon_editor editor_operation_registry_exposes_builtin_menu_operations_by_path --lib --locked --target-dir target\codex-editor-operation-check
  - cargo test -p zircon_editor editor_extension_registry_collects_plugin_windows_menus_drawers_and_operations --lib --locked --target-dir target\codex-editor-operation-check
  - cargo test -p zircon_editor operation_invocation_dispatches_to_the_same_event_and_marks_the_journal_record --lib --locked --target-dir target\codex-editor-operation-check
  - cargo test -p zircon_editor operation_control_request_returns_structured_success_and_failure --lib --locked --target-dir target\codex-editor-operation-check
  - cargo test -p zircon_editor --lib workbench_view_model_projects_menu_strip_drawers_and_status --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib editor_ui_reflection_adapter_projects_activity_hosts_and_menu_bindings --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib editor_runtime_projects_plugin_menu_operations_into_remote_callable_reflection --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib editor_runtime_registers_plugin_views_as_activity_descriptors --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib editor_runtime_projects_plugin_views_into_view_menu_operations --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib editor_runtime_exposes_plugin_component_drawer_templates_for_inspector_lookup --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib operation_control_request_lists_registered_operations_for_remote_discovery --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib operation_control_request_returns_named_operation_history_stack --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib operation_control_request_can_record_cli_source --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-source-20260427-0645 -- --test-threads=1
  - cargo test -p zircon_editor --lib explicit_plugin_operation_records_its_own_undo_stack_entry_when_reusing_builtin_event --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-source-20260427-0645 -- --test-threads=1
  - cargo test -p zircon_editor --lib failed_operation_control_request_is_journaled_without_polluting_undo_stack --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-failure-20260428-0010 -- --test-threads=1
  - cargo test -p zircon_editor --lib remote_and_cli_operation_invocation_respects_callable_from_remote_gate --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --test-threads=1
  - cargo test -p zircon_editor --lib editor_operation_ui_binding_arguments_are_preserved_in_journal --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-op-args -- --test-threads=1
  - cargo test -p zircon_editor --lib editor_operation_path_requires_namespace_action_and_leaf_segments --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-path -- --test-threads=1
  - cargo test -p zircon_editor --lib tests::editor_event::runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-path -- --test-threads=1
  - cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-path -- --test-threads=1
  - TDD red: cargo test -p zircon_editor --lib editor_extension_registry_rejects_invalid_component_drawer_operation_bindings --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --test-threads=1 (initially failed after Slint contract repairs because invalid drawer binding registered successfully)
  - cargo test -p zircon_editor --lib editor_extension_registry_rejects_invalid_component_drawer_operation_bindings --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --test-threads=1
  - cargo test -p zircon_editor --lib tests::editor_event::runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --test-threads=1
  - cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --test-threads=1
  - rustfmt --edition 2021 --check zircon_editor/src/core/editor_operation.rs zircon_editor/src/ui/host/editor_event_dispatch.rs zircon_editor/src/tests/editor_event/runtime.rs
  - blocked attempt: cargo test -p zircon_editor --lib operation_stack_moves_entries_across_undo_and_redo_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-stack -- --test-threads=1 (package cache lock/build contention from other active sessions; Cargo exited after dependency compilation without Rust diagnostics)
  - cargo test -p zircon_editor --lib operation_stack_moves_entries_across_undo_and_redo_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings --message-format short -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib tests::editor_event::runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --test-threads=1
  - TDD red: cargo test -p zircon_editor --lib operation_stack_preserves_original_source_across_undo_and_redo --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings --message-format short -- --test-threads=1 --nocapture (first failed on missing `EditorOperationStackEntry.source`, then also surfaced unrelated active Runtime UI showcase compile errors)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings (passed with existing editor dead-code and Runtime UI showcase unused-variant warnings)
  - timed out attempt: cargo test -p zircon_editor --lib operation_stack_preserves_original_source_across_undo_and_redo --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings --message-format short -- --test-threads=1 --nocapture (15 minute local timeout while compiling/linking `zircon_editor` test profile; no Rust or assertion diagnostic was emitted, and no same-target cargo/rustc process remained afterward)
  - direct test binary rerun: D:\cargo-targets\zircon-codex-editor-listener-audit-green\debug\deps\zircon_editor-86a81a58131e5a21.exe operation_stack_preserves_original_source_across_undo_and_redo --test-threads=1 --nocapture (passed: 1 test, 0 failed, 879 filtered out)
  - cargo test -p zircon_editor --lib workbench_view_model_filters_and_orders_plugin_menu_contributions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-menu-capability -- --test-threads=1
  - cargo test -p zircon_editor --lib editor_runtime_projects_plugin_menu_operations_into_remote_callable_reflection --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-menu-capability -- --test-threads=1
  - cargo test -p zircon_editor --lib editor_ui_reflection_adapter_projects_activity_hosts_and_menu_bindings --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-menu-capability -- --test-threads=1
  - cargo test -p zircon_editor --lib tests::workbench::view_model --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-menu-capability -- --test-threads=1
  - cargo test -p zircon_editor --lib tests::editor_event::runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-menu-capability -- --test-threads=1
  - cargo test -p zircon_editor --lib tests::editor_event::runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-op-args -- --test-threads=1
  - cargo test -p zircon_editor --lib tests::editor_event::runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-source-20260427-0645 -- --test-threads=1
  - cargo test -p zircon_editor --lib event_listener_control_gates_named_event_deliveries --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib event_listener_filter_limits_delivery_by_operation_path_prefix --locked --jobs 1 -- --nocapture
  - TDD red: cargo check -p zircon_editor --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings --message-format short (failed on missing `EditorEventListenerFilter::source`)
  - cargo check -p zircon_editor --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short (passed with existing `editor_meta.rs::save` dead-code warning)
  - timed out attempt: cargo test -p zircon_editor --lib event_listener_filter_limits_delivery_by_source_and_failure_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture (15 minute local timeout while linking the `zircon_editor` test binary; no Rust or assertion diagnostic was emitted, and the owned cargo/rustc processes were stopped by exact command-line match)
  - failed assertion rerun: cargo test -p zircon_editor --lib event_listener_filter_limits_delivery_by_source_and_failure_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture (delivery filtering worked, but the test expected the wrong error text fragment)
  - cargo test -p zircon_editor --lib event_listener_filter_limits_delivery_by_source_and_failure_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib event_listener_control_clears_operation_path_filter --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib event_listener_control_unregisters_listener_and_drops_deliveries --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib event_listener_control_queries_deliveries_after_sequence_cursor --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib event_listener_control_acknowledges_deliveries_through_sequence --locked --jobs 1 -- --nocapture
  - TDD red: cargo test -p zircon_editor --lib event_listener_control_rejects_unknown_listener_queries --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture (failed because `QueryDeliveries` for an unknown listener returned success with `error = None`)
  - cargo test -p zircon_editor --lib event_listener_control_rejects_unknown_listener_queries --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib event_listener_control --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture
  - pending focused test: event_listener_filter_limits_delivery_by_operation_group
  - TDD red: cargo check -p zircon_editor --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short (failed on missing `EditorEventListenerControlRequest::QueryListenerStatus`)
  - blocked attempt: cargo test -p zircon_editor --lib event_listener_control_reports_listener_status_with_pending_delivery_bounds --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture (10 minute local timeout while linking `zircon_editor` test binary; no Rust or assertion diagnostic was emitted)
  - blocked attempt: cargo check -p zircon_editor --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short (5 minute and 15 minute local timeouts before any Rust diagnostic after the status-query implementation)
  - blocked attempt: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short (10 minute local timeout while compiling the editor lib after the status-query implementation)
  - WSL blocked attempt: CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short (first failed in unrelated active runtime graphics render cutover private-field accessors; retry later timed out while compiling `zircon_editor` without a listener-status Rust diagnostic)
  - WSL accepted check: CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status RUSTFLAGS="-C debuginfo=0" cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (finished in 14m53s with existing editor warnings only)
  - WSL accepted test: CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status RUSTFLAGS="-C debuginfo=0" cargo test -p zircon_editor --lib event_listener_control_reports_listener_status_with_pending_delivery_bounds --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture (1 passed, 0 failed, 886 filtered out)
  - TDD red: CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status RUSTFLAGS="-C debuginfo=0" cargo check -p zircon_editor --tests --locked --jobs 1 --message-format short --color never (failed on missing `EditorOperationInvocation::with_operation_group` and `EditorOperationStackEntry.operation_group`)
  - WSL accepted test: CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status RUSTFLAGS="-C debuginfo=0" cargo test -p zircon_editor --lib operation_stack_merges_continuous_invocations_with_same_operation_group --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture (1 passed, 0 failed, 887 filtered out)
  - WSL accepted check: CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status RUSTFLAGS="-C debuginfo=0" cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (passed with existing editor warnings only)
  - blocked attempt: cargo test -p zircon_editor --lib failed_operation_control_request_preserves_operation_group_for_audit_delivery --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-failure-group --message-format short --color never -- --test-threads=1 --nocapture (timed out after 10 minutes before any Rust diagnostic or assertion output while other Cargo/Rustc jobs were active)
  - blocked attempt: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-failure-group --message-format short --color never (timed out after 5 minutes before any Rust diagnostic while unrelated Runtime UI/runtime-interface Cargo/Rustc jobs were active)
  - pending focused test: cargo test -p zircon_app --lib editor_cli_operation_parser_accepts_operation_group --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-app-editor-cli-group --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_app --lib editor_cli_operation_parser_rejects_args_without_operation --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-app-editor-cli-group --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_app --lib editor_cli_operation_parser_rejects_headless_without_control_request --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-app-editor-cli-group --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_app --lib editor_cli_operation_parser_rejects_operation_group_without_operation --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-app-editor-cli-group --message-format short --color never -- --test-threads=1 --nocapture
  - blocked attempt: cargo test -p zircon_app --lib editor_cli_operation_parser --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-app-editor-cli-group --message-format short --color never -- --test-threads=1 --nocapture (failed before parser tests executed because active UI runtime-interface cutover currently leaves `zircon_editor/src/ui/slint_host/viewport/submit_extract.rs:21:62` with mismatched `UiRenderExtract` identities)
  - pending focused test: cargo test -p zircon_editor --lib editor_runtime_rejects_component_drawer_bindings_to_missing_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_editor --lib editor_runtime_consumes_plugin_registration_reports_with_capability_gate --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-discovery --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_editor --lib editor_runtime_rejects_menu_items_to_missing_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-menu-operation-bindings --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_editor --lib editor_extension_registry_rejects_invalid_menu_item_paths --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-menu-operation-bindings --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_editor --lib editor_extension_registry_rejects_view_ids_that_cannot_form_open_operation_paths --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-extension-view-open-operations --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_editor --lib editor_operation_registry_rejects_invalid_menu_paths --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-menu-paths --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_app --lib editor_cli_operation_parser_rejects_operation_mixed_with_list_operations --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-app-editor-cli-group --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_app --lib editor_cli_operation_parser_rejects_list_operations_mixed_with_stack_query --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-app-editor-cli-group --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_app --lib editor_cli_operation_parser_rejects_control_request_without_headless --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-app-editor-cli-group --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_app --lib editor_cli_operation_parser_rejects_null_args_without_operation --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-app-editor-cli-group --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_app --lib editor_cli_operation_parser_rejects_duplicate_control_arguments --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-app-editor-cli-group --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_editor --lib editor_runtime_rejects_duplicate_extension_view_without_registering_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-extension-registration-atomicity --message-format short --color never -- --test-threads=1 --nocapture
  - pending focused test: cargo test -p zircon_editor --lib editor_runtime_rejects_duplicate_extension_menu_paths_without_registering_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-extension-registration-atomicity --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_app editor_cli_operation_parser_accepts_list_operations --features target-editor-host --no-default-features --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_app editor_cli_operation_ --features target-editor-host --no-default-features --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_app --lib editor_cli_operation --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-app-editor-cli-source-20260427-0710 -- --test-threads=1
  - cargo run -p zircon_app --bin zircon_editor --features target-editor-host --no-default-features --locked -- --operation-stack --headless
  - cargo test -p zircon_editor slint_adapter_binding_and_call_action_share_the_same_normalized_menu_event --lib --locked --target-dir target\codex-editor-operation-check
  - cargo test -p zircon_editor editor_runtime_accepts_plugin_extension_operations_for_later_invocation --lib --locked --target-dir target\codex-editor-operation-check
  - cargo test -p zircon_editor workbench_view_model_projects_menu_strip_drawers_and_status --lib --locked --target-dir target\codex-editor-operation-check
  - cargo check -p zircon_app --features target-editor-host --no-default-features --locked --target-dir target\codex-app-operation-check --message-format short
  - cargo test -p zircon_app editor_cli_operation_parser --features target-editor-host --no-default-features --locked --target-dir target\codex-app-operation-check
  - zircon_ui/src/tests/shared_core.rs
  - zircon_editor/tests/workbench_autolayout.rs
  - zircon_editor/src/tests/ui/binding/animation.rs
  - zircon_editor/src/tests/ui/binding/asset_selection.rs
  - zircon_editor/src/tests/ui/binding/dock_and_welcome.rs
  - zircon_editor/src/tests/ui/binding/inspector_and_draft.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/ui/control/activity_descriptors.rs
  - zircon_editor/src/tests/ui/control/reflection_projection.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/mod.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/viewport/toolbar_dispatch.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/pane/trigger_action.rs
  - zircon_editor/src/tests/host/slint_event_bridge/mod.rs
  - zircon_editor/src/tests/host/slint_viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/tests/editing/ui_asset/
  - zircon_editor/src/tests/host/slint_window/callback_source_window.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/mod.rs
- zircon_editor/src/tests/host/slint_tab_drag/
  - zircon_editor/tests/workbench_drag_targets.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor/src/tests/workbench/host_events/menu_binding.rs
  - zircon_editor/src/tests/workbench/reflection/model_projection.rs
  - zircon_editor/src/tests/workbench/reflection/remote_routes.rs
  - zircon_editor/src/tests/workbench/reflection/action_dispatch.rs
  - zircon_editor/src/tests/ui/boundary/editor_event_cutover.rs
  - cargo test -p zircon_editor --locked
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_editor editor_event_cutover --locked --quiet
  - cargo test -p zircon_editor workbench_state_cutover --locked --quiet
  - cargo check -p zircon_editor --locked --quiet
  - cargo test -p zircon_ui --lib --locked
  - cargo test -p zircon_ui --locked
  - cargo test -p zircon_ui --offline --verbose
  - cargo test -p zircon_ui shared_core -- --nocapture
  - cargo test -p zircon_editor slint_drawer_resize -- --nocapture
  - cargo test -p zircon_editor slint_viewport_toolbar_pointer --locked
  - cargo test -p zircon_editor --lib shared_viewport_surface_replaces_legacy_direct_pointer_callback_abi --locked -- --nocapture
  - cargo test -p zircon_editor --test workbench_autolayout -- --nocapture
  - cargo build --workspace --locked --verbose
  - cargo test --workspace --locked --verbose
  - cargo test -p zircon_editor --lib --locked ui_asset_editor_
  - cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_
  - cargo test -p zircon_editor --lib --locked editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo
  - cargo test -p zircon_editor --lib --locked tests::host::slint_window::child_window_callback_wiring_tracks_source_window_for_pane_interactions
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_
  - cargo test -p zircon_editor shared_document_tab_pointer_click_dispatches_focus_view_through_runtime_dispatcher --lib --locked
  - cargo test -p zircon_editor controller_does_not_republish_unchanged_captured_frame --lib --locked
  - cargo test -p zircon_editor assets_activity_bootstrap_layout_self_hosts_shell_sections --lib --locked
  - cargo test -p zircon_editor asset_browser_bootstrap_layout_self_hosts_shell_sections --lib --locked
  - cargo test -p zircon_editor assets_activity_projection_maps_bootstrap_asset_into_mount_nodes --lib --locked
  - cargo test -p zircon_editor asset_browser_projection_maps_bootstrap_asset_into_mount_nodes --lib --locked
  - cargo test -p zircon_editor assets_activity_pane_consumes_template_mount_nodes_for_toolbar_and_utility_sections --lib --locked
  - cargo test -p zircon_editor asset_browser_pane_consumes_template_mount_nodes_for_toolbar_and_utility_sections --lib --locked
  - cargo test -p zircon_editor pane_data_routes_remaining_pane_data_through_dedicated_owner_files --lib --locked
  - cargo test -p zircon_editor asset_surface_controls_use_generic_template_callbacks_instead_of_legacy_business_abi --lib --locked
  - cargo test -p zircon_editor asset_surface_templates_expose_physics_and_animation_kind_filters --lib --locked
  - cargo test -p zircon_editor asset_surface_templates_map_no_preview_physics_and_animation_assets_to_specific_icons --lib --locked
  - cargo test -p zircon_editor editor_builtin_template_files_migrate_to_asset_tree_authority --lib --locked
  - cargo check --workspace --locked
  - cargo test -p zircon_editor --lib --locked -- --test-threads=1
  - cargo test -p zircon_app --locked
doc_type: module-detail
---

# UI Binding And Reflection Architecture

## Purpose

`nativeBinding`、headless 测试、反射树远控、真实宿主点击，现在都必须汇到同一套 editor shell 协议。  
这份文档只定三件事：

- 哪些 payload 是稳定 editor UI 协议
- 哪些 shell chrome 命名空间是稳定的
- 每类 payload 在 `zircon_editor` 中最终映射到什么宿主事件或状态变更

## Viewport Raw Pointer Route

viewport 外层原始输入现在也归到同一套协议思路里了，虽然它不是 template business control：

- Slint 不再向 host 暴露 7 个分散的 viewport move/down/up/wheel 直接回调
- 宿主只接收统一 `viewport_pointer_event(kind, button, x, y, delta)` pointer fact
- `app/viewport.rs` 负责把这个 fact 还原成 `UiPointerEvent`
- [`callback_dispatch/viewport/bridge.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/bridge.rs) 与 [`callback_dispatch/viewport/pointer_dispatch.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/pointer_dispatch.rs) 再用 shared `UiSurface + UiPointerDispatcher` 产出稳定的 `EditorViewportEvent`

这一步让 viewport 原始指针输入也满足“宿主上传事实，shared dispatcher 归一化，runtime 执行 typed 事件”的统一契约，而不是继续保留一条 editor-only 的旁路输入链。

## Shared Shell Projection Notes

Workbench pointer bridges treat projected surface keys as ABI. The main document tab strip uses the stable `"document"` key because `host_document_dock_surface.slint` and the root host callback context submit that key when routing document tab activation and close events. `build_host_document_tab_pointer_layout(...)` must preserve this key even when the surrounding layout model changes, otherwise the shared pointer dispatcher cannot map Slint facts back to `LayoutCommand::FocusView` or `CloseView`.

Viewport image polling is edge-triggered from the Slint host perspective. `SlintViewportController::poll_image()` returns `Some(image)` only when the render framework exposes a new captured-frame generation; repeated captures with the same generation keep the cached image internally but return `None` so the host does not republish unchanged pixels.

## Builtin Template Asset Authority

Builtin editor host templates now enter the runtime through [`EditorUiHostRuntime::register_document_file(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs). The method compiles the referenced `ui.toml` file with recursive widget/style import resolution from crate `assets/`, so builtin host assets and future editor plugin `ui.toml` files use the same file-backed registration seam. [`load_builtin_host_templates(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/runtime/build_session.rs) no longer embeds production template strings or manually registers compiled documents behind the runtime API.

The asset activity and asset browser panes now expose their tree/source columns as template-owned mount nodes in [`assets_activity.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/assets_activity.ui.toml) and [`asset_browser.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/asset_browser.ui.toml). [`pane_content.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_content.slint) consumes those nodes directly instead of falling back to a legacy hard-coded `FolderTreeView`, while still routing pointer facts into the shared asset tree dispatch path.

`TemplatePaneNodeData` is the Slint host's generic retained node bridge for editor-authored `ui.toml` documents and future plugin component drawers. Runtime component projection now preserves row state that would otherwise be lost at the host boundary: `selected`, `focused`, `hovered`, `expanded`, `tree_depth`, and `tree_indent_px` all survive from document props or runtime component flags into [`template_pane.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/template_pane.slint). The showcase runtime overlay only writes transient flags when they are active, so declarative visual state in [`component_showcase.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/component_showcase.ui.toml) is not erased by an inactive default `UiComponentState`; tree rows use explicit `tree_indent_px` when present and otherwise derive indentation from `tree_depth`. This matters for plugin-provided `ui.toml` drawers because custom inspectors need ListRow/TreeRow selection and hierarchy affordances to be first-class data, not hard-coded Slint-only styling.

Asset surface controls also use explicit generic routes for each template control. Kind chips call `root.control_changed("SetKindFilter", "...")`, view buttons call `root.control_changed("SetViewMode", "...")`, utility tabs call `root.control_changed("SetUtilityTab", "...")`, and commands such as `OpenAssetBrowser`, `LocateSelectedAsset`, and `ImportModel` call `root.control_clicked(...)`. No asset-specific business callback is exposed from the Slint shell.

No-preview asset icons are centralized in [`assets.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/assets.slint) through shared selection preview components. Thumbnail/list fallbacks and selection/details fallbacks all classify physics and animation assets through the same kind-to-icon mapping.

## Layer Boundary

### `zircon_ui`

`zircon_ui` 现在是共享 UI 权威层，不再只承载 binding/reflection：

- layout primitives
  - `StretchMode`
  - `AxisConstraint`
  - `ResolvedAxisConstraint`
  - `BoxConstraints`
  - `DesiredSize`
  - `LayoutBoundary`
  - `UiAxis`
  - `UiContainerKind`
  - `UiLinearBoxConfig`
  - `UiScrollState`
  - `UiScrollableBoxConfig`
  - `UiVirtualListConfig`
  - `UiPoint` / `UiSize` / `UiFrame`
  - `compute_layout_tree(...)`
- retained UI tree and invalidation
  - `UiTree`
  - `UiTreeNode`
  - `zircon_ui::tree::UiDirtyFlags`
  - `zircon_ui::tree::UiLayoutCache`
  - `UiInputPolicy`
- hit-test and surface state
  - `zircon_ui::tree::UiHitTestIndex`
  - `zircon_ui::tree::UiHitTestResult`
  - `UiSurface`
  - `UiFocusState`
  - `UiNavigationState`
  - `UiPointerEvent`
  - `UiPointerRoute`
  - `UiNavigationRoute`
  - `UiNavigationDispatcher`
  - `UiNavigationDispatchEffect`
  - `UiNavigationDispatchResult`
  - `UiPointerDispatcher`
  - `UiPointerDispatchEffect`
  - `UiPointerDispatchResult`
  - `UiVirtualListWindow`
  - `UiRenderExtract`
- binding AST
  - 现在落在 `binding/model/*` 子树，而不是单个 `model.rs`
- binding codec
- 反射树快照与 diff
- route id 注册与调用
  - 现在落在 `event_ui/manager/*` 子树，而不是单个 `manager.rs`
- transport-neutral request/subscribe control plane

这意味着 binding/reflection 只是 `zircon_ui` 的一部分。共享布局、`HorizontalBox` / `VerticalBox` / `ScrollableBox` 容器、visible-range invalidation、命中、clip 链检查、surface/render extract，以及 pointer/navigation dispatcher 权限也已经下沉到这个 crate；editor 侧只保留 docking/workbench 语义和 editor-only payload。共享布局基础的细节单独记录在 [Shared UI Core Foundation](../ui-and-layout/shared-ui-core-foundation.md)。

### `zircon_editor::ui`

`zircon_editor::ui` 负责 editor-only 协议：

- `EditorUiBinding`
- `EditorUiBindingPayload`
- `SelectionCommand`
- `AssetCommand`
- `WelcomeCommand`
- `DraftCommand`
- `DockCommand`
- `ViewportCommand`
- `InspectorFieldBatch`
- editor reflection adapter / control service

这些 typed payload 现在不再继续堆在单一 `binding.rs` 里，而是固定拆成目录化 command/codec 子树：

- `binding/asset/*`
- `binding/dock/*`
- `binding/draft/*`
- `binding/selection/*`
- `binding/viewport/*`
- `binding/welcome/*`

这次收束里，`DockCommand`、`DraftCommand`、`ViewportCommand` 的 command+codec 支撑层已经补齐，`viewport/*` 还进一步拆出 tool / transform space / projection / display / grid / orientation 这些 typed value codec 子模块。这样 `zircon_editor::ui` 的稳定协议面可以继续增长，而不需要重新把所有 editor binding 解析塞回一个大文件。

### `zircon_editor`

`zircon_editor` 现在通过 `editor_event` 子系统独占 editor 行为执行权：

- semantic event normalization
- event dispatch / execution
- transient UI state tracking
- reflection rebuild
- event journal + replay
- menu dispatch
- layout mutation
- selection propagation
- inspector batch commit
- asset open/reveal intent dispatch
- viewport input dispatch

同时，`zircon_editor::workbench::autolayout` 不再拥有底层 `AxisConstraint` / `StretchMode` / `PaneConstraints` 的定义权。它只负责把 `WorkbenchLayout`、descriptor 默认约束和 region override 映射成共享 `zircon_ui` 约束，再把求解结果投影回 editor shell frame。

`zircon_editor::ui` / `zircon_ui` 不再拥有 editor 行为闭包。  
它们只保留 typed binding、route metadata、reflection schema、codec 和 query/control primitives。

热路径现在固定为：

1. Slint / headless / MCP 输入先被适配成 `EditorEventEnvelope` 或 `EditorUiBinding`
2. `EditorEventRuntime` 归一化成 `EditorEvent`
3. 同一个 dispatcher 路径执行状态变更
4. runtime 记录 journal record
5. runtime 用 `EditorState + EditorManager + EditorTransientUiState` 重建 reflection snapshot

因此 `nativeBinding`、Slint callback 名称、route id 都只是外层 transport / adapter 输入，不再是语义层日志格式。

同一轮 manager 边界整治里，editor shell 的 descriptor catalog 和 layout host bookkeeping 也被压回结构化入口：

- `zircon_editor/src/core/host/manager/builtin_views/mod.rs` 现在只保留 builtin view catalog wiring，activity-view / activity-window descriptor construction 与 welcome / UI asset editor descriptor 拼装已经分层下沉
- `zircon_editor/src/core/host/manager/layout_hosts/mod.rs` 现在只保留 layout host wiring，active tab lookup、document host traversal、workbench root repair、builtin shell repair 不再混在同一个 manager 脚本里

当前 viewport toolbar 也已经并入这条 shared dispatch 主链：

- Slint toolbar 不再为每个按钮暴露独立 direct callback 路径，而是统一走 shared pointer 点击入口
- [`viewport_toolbar_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs) 负责 surface-local hit route 归一化
- [`callback_dispatch/shared_pointer/viewport_toolbar.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/shared_pointer/viewport_toolbar.rs) 与 [`callback_dispatch/viewport/toolbar_control.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/toolbar_control.rs) 把 route 转回 typed `ViewportCommand` 或 cycle/toggle 语义
- runtime journal 中记录的仍是统一 `EditorViewportEvent`，而不是某个 Slint-only callback 名称

同一轮里，transient pane surface action 也进入了 template/runtime authority：

- [`pane_surface_controls.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml) 定义 builtin `PaneSurface/TriggerAction`
- [`callback_dispatch/pane/surface_control.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/pane/surface_control.rs) 用 `BuiltinPaneSurfaceTemplateBridge` 把 `control_id + action_id` 重组回 canonical `MenuAction`
- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 不再暴露 root `menu_action(action_id)` callback；Scene/Game empty-state 和 Project overview 的 `Open Assets` 现在只上传 generic `pane_surface_control_clicked(control_id, action_id)`

这意味着 `PaneActionModel` 虽然仍然携带 `menu_action_binding(...)`，但 Slint compatibility shell 不再把这类动作直接当 handwritten callback ABI 透传出去。

当前有一条刻意保留的例外 seam：

- `WelcomeCommand`

它现在已经拥有稳定 `WelcomeSurface/*` binding 命名空间和统一 `dispatch_welcome_binding(...)` 解析，但仍停在 host-owned `WelcomeHostEvent`，没有被伪装成 runtime 已接管的 `EditorEvent`。原因是 `EditorStartupSessionDocument`、recent project 验证、create/open project 流程以及 exclusive welcome page 生命周期还由 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 和 [`startup/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/startup/mod.rs) 持有。当前架构边界是：

- 模板 authority 决定 welcome control 语义
- 宿主 authority 决定 startup session 执行与页面生命周期

## Editor Event Runtime

`zircon_editor/src/core/editor_event/` 当前包含：

- `inspector_field_change.rs`
  - inspector batch/live-edit 用的结构化字段变更 DTO
- `selection_host_event.rs`
  - selection binding 的 typed host event DTO
- `workbench/`
  - canonical workbench event DTO family，包括 `MenuAction`、`LayoutCommand`、drawer/page/workspace/view identity 和 split/attach 元数据
- `types.rs`
  - 定义 canonical `EditorEvent`、`EditorDraftEvent`、`EditorEventRecord`、`EditorEventResult`、`EditorEventUndoPolicy`
- `runtime.rs`
  - `EditorEventRuntime` / `EditorEventDispatcher`
  - 统一拦截 `InvokeBinding`、`InvokeRoute`、`CallAction`
- `runtime/editor_event_runtime_inner.rs`
  - `EditorEventRuntime` 的私有 state container owner
  - 持有 `EditorState`、`EditorManager`、transient projection、journal 和 control service
- `journal.rs`
  - session-local event record 存储
- `replay.rs`
  - recorded `EditorEvent` 重新走同一 dispatcher path

transient hover/focus/pressed/drawer-resize 投影现在已经迁到 [`transient_ui_state.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/reflection/transient_ui_state.rs)，而 Slint workbench 菜单/动态 preset 的字符串归一化则由 [`callback_dispatch/workbench/menu_action.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs) 持有，再通过 [`core_event_conversion.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/event/core_event_conversion.rs) 把 UI 内部 layout model 显式转换成 canonical `core::editor_event::workbench::*` DTO。

`EditorEventRuntimeInner` 的声明 owner 这一轮也从 `ui/host` 收回到了 [`core/editor_event/runtime/editor_event_runtime_inner.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/editor_event/runtime/editor_event_runtime_inner.rs)。`ui/host` 现在只保留 bootstrap、dispatch、execution、reflection 这些行为模块，直接消费 core runtime inner，而不是继续拥有 runtime state declaration 本身。`EditorEventRuntime::lock_inner()` 是这些 host 行为模块的统一锁入口；如果先前 editor callback 已经 panic 并 poison 了 mutex，后续 snapshot、reflection 和 descriptor 查询会显式取回 inner 状态，而不是在每个访问点继续 `unwrap()` 并把 editor shell 永久卡死在连锁 panic 上。

当前 canonical log record 固定保存：

- `event_id`
- `sequence`
- `source`
- normalized `EditorEvent`
- optional `operation_id` / `operation_display_name` / `operation_arguments` / `operation_group`
- `before_revision` / `after_revision`
- emitted effects
- undo policy
- structured success / failure result

## Slint Host Adapter Path

桌面宿主现在也必须遵守和 headless / reflection 相同的 runtime authority：

- `zircon_editor/src/ui/slint_host/app.rs`
  - `SlintEditorHost` 持有 `EditorEventRuntime`
  - 大多数 `ui.on_*` callback 只负责采集桌面输入、必要的瞬态宿主 bookkeeping、提交 dispatch、消费 effect
  - 不再直接持有 editor 行为语义，也不再在 callback 里直接改 `EditorState` / `EditorManager`
- `zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs`
  - 把 raw Slint callback 输入收敛成 `EditorEventEnvelope` 或直接语义化 `LayoutCommand`
  - 统一走 `runtime.dispatch_envelope(...)`
  - workbench menu/preset 字符串入口现在由 `callback_dispatch/workbench/menu_action.rs` 统一生成 canonical `core::editor_event::MenuAction` / `LayoutCommand`
  - UI 内部 `ui::workbench::layout::LayoutCommand` 与 canonical `core::editor_event::LayoutCommand` 的边界转换固定在 `ui/workbench/event/core_event_conversion.rs`
  - viewport pointer/scroll 现在先经过 `SharedViewportPointerBridge` 的 shared `UiSurface + UiPointerDispatcher`，再映射成 `EditorViewportEvent`
  - `AssetSurface/*` 已经通过 builtin template bridge 直接落到 typed runtime dispatch
  - `WelcomeSurface/*` 已经通过 builtin template bridge 落到 typed `WelcomeHostEvent`，再由宿主执行 startup session 逻辑
- `zircon_editor/src/ui/slint_host/event_bridge.rs`
  - 把 `EditorEventRecord.effects` 映射成宿主展示侧需要的 `SlintDispatchEffects`
  - 宿主只消费 `presentation_dirty / layout_dirty / render_dirty / sync_asset_workspace` 之类的结果，不重新解释语义
- `zircon_editor/src/ui/slint_host/drawer_resize.rs`
  - 把 splitter group 级别的桌面手势换成 `LayoutCommand::SetDrawerExtent`
  - `begin_drawer_resize(...)` 现在只上传 pointer 坐标；真正的 `left / right / bottom` target group 由统一 shell pointer bridge 在 shared `UiSurface + UiPointerDispatcher` 上解析
- `zircon_editor/src/ui/slint_host/shell_pointer.rs`
  - `WorkbenchShellPointerBridge` 把 shell drag target 与 splitter target 收口到同一棵 `UiSurface`
  - drag route 与 resize route 共用 hit-test / clip / capture 状态，但通过两个 mode-specific dispatcher 解释 editor-only 语义
- `zircon_editor/src/ui/slint_host/tab_drag.rs`
  - tab drop 继续把 resolved group 翻译成 editor-only `ViewHost`

保留在宿主侧但不进入当前 canonical event catalog 的内容仍然只有 adapter 级职责：

- raw `InputManager` pointer / keyboard forwarding
- shell geometry 和 viewport texture bridge
- asset/resource polling refresh
- startup draft / recent project 生命周期

随着 `zircon_ui` 已经拥有 `ScrollableBox` 和第一版 `UiPointerDispatcher`，后续宿主接线的目标也更明确：

- 宿主先把 pointer / scroll 输入适配成 shared `UiPointerEvent`
- 共享 `UiSurface + UiPointerDispatcher` 先完成 route、stacked target 语义、capture 和默认 scroll container 响应
- keyboard / gamepad 输入后续也必须先适配成 `UiNavigationEventKind`，再由 `UiSurface::dispatch_navigation_event(...)` 走 shared `UiNavigationDispatcher`
- shared navigation fallback 现在先于宿主生效：`Next` / `Previous` 走共享 tab order，`Right` / `Down` 在无焦点时从首个 focusable 节点开始，`Left` / `Up` 从末尾 focusable 节点开始，`Activate` / `Cancel` 不做隐式焦点跳转
- 只有 editor-only 的 docking / menu / inspector / viewport payload 再继续上送到 `EditorEventRuntime`

这一层现在已经开始进入真实接线阶段，但范围仍然是刻意收窄的：

- viewport pointer/scroll callback 已经改成先走 shared dispatcher
- menu、hierarchy、asset、drawer resize、tab drag 仍然保持各自的薄 adapter，不把 editor-only payload 混进 `zircon_ui`
- 更细粒度的 docking transient 仍然属于后续迁移面，但 group 级别 shell pointer hit-test / dock target route 已经进一步收口到统一 shell bridge

这一条“后续迁移面”现在已经向前推进了一段：

- workbench tab drag 的 `left / right / bottom / document` target route 不再由 `workbench.slint` 本地 `drag_target_group` 公式决定
- Slint shell 新增 `update_drag_target(x, y)` callback，只把 pointer 位置交回 host
- host 通过 `WorkbenchShellPointerBridge` 在 shared `UiSurface` 上同时维护 drag target retained 节点和 splitter target retained 节点
- overlap 区域通过 `UiPointerDispatchEffect::{Handled, Passthrough}` 决定 side/bottom 归属
- splitter `Down` 会在 shared dispatcher 里触发 capture，后续 `Move / Up` 即使移出 splitter hit bounds 也继续路由到同一 target
- Slint 现在只消费 host 写回的 `active_drag_target_group`，以及 host 内部基于同一 shell bridge 解析出的 resize group

这意味着 editor shell 的 dock target hit-test 已经开始进入 shared-core-first 路线，而不是继续把 pointer route 留在宿主 UI 壳里。

## Stable Shell Chrome Namespaces

这一轮固定的 shell chrome 命名空间如下：

- `WorkbenchMenuBar/*`
- `ActivityRail/*`
- `ToolWindow/*`
- `DocumentTabs/*`
- `InspectorView/*`
- `ViewportToolbar/*`
- `StatusBar/*`
- `AssetSurface/*`
- `WelcomeSurface/*`
- `PaneSurface/*`

这些名字的作用不是替代 activity instance id，而是给壳层 chrome 提供稳定协议面。  
例如：

- `WorkbenchMenuBar/OpenProject`
- `ActivityRail/ProjectToggle`
- `InspectorView/ApplyBatchButton`

## Editor Payload Surface

`zircon_editor::EditorUiBindingPayload` 当前固定包含：

- `PositionOfTrackAndFrame`
- `MenuAction`
- `SelectionCommand`
- `AssetCommand`
- `WelcomeCommand`
- `DraftCommand`
- `DockCommand`
- `ViewportCommand`
- `InspectorFieldBatch`
- `Custom`

其中 `MenuAction` 和 `InspectorFieldBatch` 继续保留原形态；workbench shell 相关的新行为全部进入 typed command family，而不是继续堆 stringly-typed custom payload。

### Draft / Live Edit

`DraftCommand` 当前承担“只更新 live snapshot、不立即触发持久化执行”的编辑语义。已经落地的 typed draft 面有两类：

- `DraftCommand::SetInspectorField { subject_path, field_id, value }`
- `DraftCommand::SetMeshImportPath { value }`

它们统一归一化成 canonical `EditorEvent::Draft(EditorDraftEvent::...)`，并且遵守以下边界：

- draft 只更新 runtime/editor snapshot 与 reflection surface
- draft 不会隐式触发 `ApplyInspectorChanges`
- draft 不会隐式触发 mesh import、asset sync 或 render side effect
- draft 属于 `NonUndoable`，不会混入 editor history 的持久化命令语义

## Workbench Shell Event Contract

## Editor Operation Layer

`EditorOperation` 现在是 editor 对外暴露的“可命名操作”层，路径采用 `XXX.YYY.ZZZ` 形式，例如 `Window.Layout.Reset`、`Scene.Node.CreateCube` 和 `Edit.History.Undo`。这一层不替代 `EditorEvent`：operation registry 只负责声明路径、菜单路径、远控可调用性、可撤销展示名以及最终要提交的 canonical editor event。

当前第一阶段的执行链路固定为：

1. 菜单、插件扩展、headless 测试、远控或 CLI 入口提交 `EditorOperationInvocation`
2. `EditorEventRuntime::invoke_operation(...)` 在 `EditorOperationRegistry` 中解析路径
3. descriptor 解析成 canonical `EditorEvent`
4. runtime 通过 `dispatch_normalized_event_with_operation(...)` 把显式 operation path、display name 和 undoable 标记带入 canonical event dispatch
5. `EditorOperationDescriptor.callable_from_remote` 会 gate Remote / CLI control request；菜单与 `ui.toml`/reflection UI binding 仍可走内部来源触发
6. registry / remote-callability / capability / handler control failure 会先归一化成 `EditorEvent::Operation(EditorOperationEvent::ControlFailure)`，再写入同一 journal
7. journal record 额外写入 `operation_id`、`operation_display_name`、可选 `operation_arguments` 和可选 `operation_group`
8. undoable operation 成功执行后，`EditorOperationStack` 记录被显式调用的 operation id/display name/sequence/source/group

这让 operation 成为 Unity `MenuItem` / `ExecuteMenuItem` 类似的公共命名入口，而 journal/replay 仍然保留 Zircon 自己的 typed event 权威格式。Operation id 至少需要三段 dotted namespace，例如 `Weather.CloudLayer.Refresh`；更深层路径如 `View.weather.cloud_layers.Open` 也合法，但 `Weather.Refresh` 这种缺少命名空间/叶子层级的短路径会在注册或调用前被拒绝。`EditorOperationDescriptor.menu_path` 使用同一类 Unity 风格 slash path 约束：至少包含顶层菜单和叶子项，且不能有空 segment 或首尾空白 segment；`EditorOperationRegistry::register(...)` 会在接受 descriptor 时校验它，避免远控 discovery 和 Workbench 菜单投影拿到不可构建的菜单元数据。旧 Slint 菜单 callback、`InvokeBinding` 和普通 `CallAction` 仍可按 canonical event 反查 builtin `EditorOperationRegistry`，因此同一个内建菜单行为会得到同一个 `operation_id`。显式 `invoke_operation(...)`、扩展菜单和 CLI 入口则不再在 dispatch 后补写 metadata，而是在提交 canonical event 前携带被调用 descriptor 的身份；这避免插件 operation 复用 `MenuAction::ResetLayout` 这类内建 event 时，journal 或 operation stack 被错误记录成 `Window.Layout.Reset`。`EditorOperation` UI binding 还会保留 `CallAction` / `InvokeRoute` 带入的参数，并把它们转换为 `EditorOperationInvocation.arguments` 与 journal/listener delivery 上的 `operation_arguments`，因此 `ui.toml` 控件事件、反射调用和外部程序可以共享同一条带参数 operation 审计记录。连续编辑还可以设置 `EditorOperationInvocation.operation_group`；journal/listener 仍保留每一次 dispatch 的独立 record，但 `EditorOperationStack` 会把同一 operation id 与 group 的连续 undoable invocation 合并到一条历史项，并把 sequence 更新到最新 dispatch。后续 View、Drawer、Component Inspector 的 `ui.toml` 控件只需要绑定 operation path；控制脚本注册的 handler 也应先进入 operation registry，再由 operation 统一提交 editor event。

失败的 control request 也必须进入同一条记录链路。`EditorEvent::Operation(EditorOperationEvent::ControlFailure)` 是非 undoable event：它会把错误写入 status line、journal result 和 listener delivery，并保留调用方提供的 `operation_group` 供外部批处理/连续交互审计，但不会污染 `EditorOperationStack`。`EditorEventReplay` 会把 journal 中原本就是失败的 record 当作预期失败重放，并继续处理后续记录；原本成功的 record 如果重放失败仍然会中断。这样外部程序、CLI 和 UI 绑定看到的是完整操作审计流，而 undo/redo 栈只保留真正成功并声明可撤回的操作。

`callable_from_remote=false` 只限制远控/CLI 入口，不限制 editor 内部菜单和 `ui.toml` 控件。这样插件可以把某些操作作为内部 UI 事件或 validator 后续动作，同时不把它暴露成外部程序可直接调用的 command surface；被拦截的远控/CLI invocation 仍然按失败 operation 写入 journal，方便审计外部自动化尝试。

`UndoableEditorOperation` 当前先作为 operation descriptor 上的命名元数据存在，用于菜单、operation stack 和 journal 展示。真正可逆 mutation 仍然由现有 `EditorHistory` / `EditorCommand` 以及 `EditorEventUndoPolicy` 执行；下一阶段再把非 scene 的 layout、asset、component-editor mutation 从 `FutureInverseEvent` 收束为显式 inverse operation 或 transaction group。

`EditorExtensionRegistry` 是 editor 插件贡献的强类型入口。它收集 view、drawer、Unity 风格菜单路径、component drawer、`ui.toml` 模板和 operation descriptor，并把重复 id 诊断收束在注册阶段。`EditorEventRuntime::register_editor_extension(...)` 会把 extension registry 中的 operation descriptor 合并进 live operation registry，并保存原始 extension registry，给后续 workbench 菜单、view、drawer 投影继续消费。注册前 runtime 会先在 scratch operation registry 中合并内建 operation、本扩展 operation、以及扩展 View 自动生成 open operation，再用这份候选集合校验菜单项和组件 Inspector bindings；扩展 View 也会先经过 Workbench ViewRegistry 冲突预检。已注册扩展的 drawer id、菜单路径、component drawer component type、ui template id 会作为 live contribution set 参与冲突校验，避免两个插件把同一投影入口发布成不同语义。只有所有校验都通过时，runtime 才把 scratch operation registry、扩展 View 和 extension registration 提交到 live 状态，避免注册失败时留下半注册 operation。组件 Inspector 的 `ComponentDrawerDescriptor` 记录 component type、ui document、controller 和 bindings；bindings 的最终执行目标仍然应该归一化为 `EditorOperationInvocation`，避免自定义 UI 脚本绕过 journal。

扩展 View 已经接入 Workbench ViewRegistry。`register_editor_extension(...)` 会把 extension `ViewDescriptor` 转换成 ActivityView 类型的 workbench descriptor，并在下一次 reflection refresh 时注册到 `EditorUiControlService` 的 activity descriptor 列表；这让外部工具能先发现 `weather.cloud_layers` 这类插件 View。注册扩展 View 时 runtime 还会自动补出 `View.<id>.Open` operation，例如 `View.weather.cloud_layers.Open`，其 canonical event 是 `MenuAction::OpenView(...)`。`EditorExtensionRegistry::register_view(...)` 会在接受 descriptor 前先解析这条自动 operation path，拒绝包含 `/`、空格或其它 dotted operation 非法字符的 View id；这样插件不会贡献一个 workbench 可以显示、但菜单/远控无法生成合法 operation 的窗口。Workbench View 菜单会投影同一个 operation binding，因此外部程序可以从 reflection menu node 调 `CallAction(onClick)` 打开插件 View，并在 journal 中得到同一条 `operation_id`。扩展 Drawer 仍保持为 metadata registry，真正布局层当前仍是固定 slot 模型，后续需要在不破坏现有 drawer ownership 的前提下定义动态 drawer 容器。

组件 Inspector 的扩展入口现在是可查询元数据，而不是脚本旁路。`ComponentDrawerDescriptor::with_binding(...)` 记录 controller 预期注册或触发的 operation path，`EditorExtensionRegistry::register_component_drawer(...)` 会用同一个 `EditorOperationPath::parse(...)` 校验这些 binding，拒绝 `Weather.Refresh` 这类不满足 `XXX.YYY.ZZZ` 命名契约的 drawer 贡献。`EditorEventRuntime::register_editor_extension(...)` 在合并 extension 前还会把内建 operation、本扩展声明的 operation、以及扩展 View 自动生成的 `View.<id>.Open` operation 合并成候选集合，并拒绝绑定到不存在 operation 的 ComponentDrawer；这样插件 Inspector 不会发布一个 UI 控件能显示但无法通过 operation registry 派发的按钮。`EditorEventRuntime::component_drawer_descriptor(...)` 和 `ui_template_descriptor(...)` 提供运行时查询入口。执行阶段仍应由 controller 把控件事件提交成 `EditorOperation`，再进入 journal / operation stack。

`EditorOperationStack` 当前记录通过 operation 层成功执行的可撤销操作名称、source、sequence 和可选 operation group。显式 operation 调用会记录调用者选择的 operation path，而不是从 canonical event 反推出的第一个内建 descriptor；这保证插件提供的 `Tools/...` operation 可以复用内建 event handler，同时仍在 Photoshop 风格历史栈里保持自己的命名身份。带 `operation_group` 的连续操作只在最后一条 undo stack entry 上更新 sequence/source/group，不会为拖拽、滑块或连续字段编辑的每个中间值堆出独立历史项；redo stack 仍会在新 grouped write 后清空。内建 `Edit.History.Undo` / `Edit.History.Redo` 是非 undoable dispatcher command：执行成功后只把上一条命名操作从 undo stack 移到 redo stack，或从 redo stack 移回 undo stack，不会把 Undo/Redo 自己压入历史，也不会覆盖原条目的 source。它先作为旧 `EditorHistory` 旁边的统一命名栈存在；scene mutation 的真正 undo/redo 仍由旧 command history 执行，operation stack 提供菜单、远控和未来 grouped transaction 需要的公共展示与归档结构。

`EditorEventListenerRegistry` 是 runtime 内部的事件监听控制层。外部历史面板、自动化桥或后续 MCP/WebSocket transport 可以先注册 listener，再用 `SetEnabled` 暂停/恢复投递，用 `SetFilter` 约束接收范围，用 `ClearFilter` 恢复全量监听，用 `ListListeners` 枚举 descriptor，用 `QueryListenerStatus` 查询单个 listener 的 descriptor、pending delivery 数量、首个 pending sequence 和最后一个 pending sequence，再用 `QueryDeliveries` 读取已投递的 event id、sequence、source、operation path、operation display name、operation arguments、operation group 和 result。长期轮询端应优先使用 `QueryDeliveriesSince { after_sequence }`，只拉取上次 cursor 之后的新 delivery，避免每轮消费全量缓存；消费成功后再提交 `AckDeliveriesThrough { listener_id, sequence }`，runtime 会移除该 listener 已确认 sequence 及之前的缓存投递，保留后续未消费记录。filter 支持 operation path prefix、operation group 精确匹配、event source 列表以及 success/failure 状态开关，例如只订阅 `Scene.Node.`、只订阅 `Viewport.TransformDrag.42`、只订阅 `EditorEventSource::Cli`、或只订阅失败的 CLI operation control request；这样外部工具可以按 `XXX.YYY.ZZZ` 命名空间、连续操作批次、调用来源和审计结果订阅事件，而不需要自己消费全量 journal。外部连接关闭时应调用 `Unregister`，runtime 会同时移除 listener descriptor 和该 listener 的投递缓存，避免断开的远控面板继续累积事件；注销后继续 `QueryDeliveries`、`QueryDeliveriesSince` 或 `AckDeliveriesThrough` 会得到结构化 failure，而不是一个看起来成功的空投递结果。dispatch 成功或失败都会形成带 operation metadata 的 `EditorEventRecord` 并写入 journal；listener 投递引用同一条 record 的身份信息，因此外部历史面板不会再经历“匿名事件先投递、随后再补写 operation id”的中间状态。

`EditorOperationControlRequest::ListOperations` 是远控和 CLI 的 operation discovery 面。返回列表会先按当前 enabled capabilities 过滤不可用 operation，再为每个可见 operation 暴露 `operation_id`、display/menu 名称、remote callable 标记、undoable 元数据以及 `required_capabilities`；这样外部面板既不会显示当前能力关闭的菜单项，也能解释已显示 operation 依赖哪些插件能力。

Workbench 菜单模型现在也带 operation metadata：

- [`MenuItemModel`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/model/menu_item_model.rs) 保留旧 `MenuAction` 和 typed binding，同时通过 `operation_path_for_menu_action(...)` 给已注册 builtin operation 填入 `operation_path`
- [`build_workbench_reflection_model(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/reflection/model_build.rs) 会把该路径和可选 shortcut 投影进 `EditorMenuItemReflectionModel`
- [`EditorUiReflectionAdapter`](/E:/Git/ZirconEngine/zircon_editor/src/ui/reflection.rs) 会在 menu item node 上暴露 `operation_path` 和可选 `shortcut` 属性

这样菜单点击仍可沿用旧 `MenuAction` 热路径，外部程序和未来命令行则可以从同一反射节点读到 `File.Project.Save` / `Window.Layout.Reset` 等 operation 路径，再通过 operation control request 触发同一行为。

扩展贡献的菜单项不再只停留在 `EditorExtensionRegistry`。`EditorMenuItemDescriptor` 现在支持 Unity `MenuItem` 风格的 `path`、`priority`、`shortcut`、`enabled` 和 `required_capabilities` 元数据；`EditorExtensionRegistry::register_menu_item(...)` 会先拒绝空路径、单段路径、空 segment、首尾 slash、以及带首尾空白的 segment，确保路径至少有顶层菜单和叶子项。`EditorEventRuntime::register_editor_extension(...)` 会拒绝指向不存在 operation 的菜单项，所以 Workbench 不会显示一个无法通过 operation registry 派发的插件菜单。`WorkbenchViewModel::build_with_extensions_and_capabilities(...)` 会先按菜单项 capability 过滤，再按 priority / path 稳定排序，把 `EditorMenuItemDescriptor.path()` 的 Unity 风格路径投影到顶层菜单，把叶子段作为显示 label，并使用 [`editor_operation_binding(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/event/editor_operation_binding.rs) 生成 `EditorOperation` UI payload。`enabled` 是 v1 的菜单验证结果承载字段：插件或后续 Rust/VM validate handler 可以把当前上下文的校验结果投影成 disabled 菜单项，而 capability 仍负责隐藏不可用贡献。`CallAction(onClick)` 进入 `EditorEventRuntime::invoke_editor_binding(...)` 后会直接调用 `invoke_operation(EditorOperationSource::UiBinding, ...)`，因此扩展菜单、远控 route 和显式 operation invocation 都会写入同一个 `operation_id` journal 记录。

外部控制有两个入口：

- 进程内调用 `EditorEventRuntime::handle_operation_control_request(EditorOperationControlRequest::InvokeOperation(...))`
- 进程内也可以调用 `EditorOperationControlRequest::ListOperations`，返回 `operations[]`，其中包含 `operation_id`、`display_name`、`menu_path`、`callable_from_remote`、`undoable` 和可选 `undo_display_name`
- 进程内可以调用 `EditorOperationControlRequest::QueryOperationStack`，返回 `undo_stack[]` 和 `redo_stack[]`，每条记录包含 operation path、展示名、source 和 sequence，用于远控面板或 Photoshop 风格历史面板先展示命名栈
- `zircon_editor` 命令行使用 `--operation <id> --args <json> --operation-group <id> --headless`，由 app 层构造 headless editor runtime 后走同一 operation control request，并输出结构化 JSON response；该入口会以 `EditorOperationSource::Cli` 派发，journal source 记录为 `EditorEventSource::Cli`，便于远程、脚本和 UI 触发在同一 journal 中区分来源
- CLI 入口现在通过 `EditorOperationInvocation::new(...).with_arguments(...)` 构造请求，并在提供 `--operation-group` 时调用 `with_operation_group(...)`；默认 `operation_group = None`，需要合并连续操作的调用方必须显式提供 group，避免命令行触发因为结构字段新增而绕过默认语义。`--args` 和 `--operation-group` 都必须和 `--operation` 同时出现，即使 `--args null` 也会按“已提供参数”处理；`--operation`、`--args`、`--operation-group`、`--list-operations`、`--operation-stack`、`--headless` 重复出现都会被拒绝，避免外部脚本的歧义输入被后者覆盖。`--operation`、`--list-operations`、`--operation-stack` 三种控制模式互斥；任何控制模式都必须显式携带 `--headless`，反过来单独的 `--headless` 也会被拒绝，避免外部脚本把控制参数传给普通 GUI 启动路径后被静默忽略、被另一个控制模式静默覆盖，或把非 headless 启动意图误变成一次 JSON command
- `zircon_editor --list-operations --headless` 走同一个 `ListOperations` 请求，供外部工具或脚本先发现可调用路径，再选择触发哪个 operation
- `zircon_editor --operation-stack --headless` 走同一个 `QueryOperationStack` 请求，供外部历史面板或自动化脚本读取当前命名 undo/redo 栈

### Menu

- source: `WorkbenchMenuBar/*`
- payload: `MenuAction`
- dispatch entry: `dispatch_editor_host_binding`
- result: `EditorHostEvent::Menu`

menu 继续承载：

- project open/save
- layout save/reset
- undo/redo
- create node
- open view

其中 `MenuAction::OpenProject` 现在不再直接在 runtime 里假定“已有 project path 并立即打开项目”。当前语义是：

- runtime 记录 canonical menu event
- runtime 发出 `EditorEventEffect::PresentWelcomeRequested`
- `SlintDispatchEffects` 把它翻译成 host-level present-welcome 标志
- 宿主再调用现有 `present_welcome_surface(...)`

这样 `WorkbenchMenuBar/OpenProject` 已经回到了统一 template/menu/event 链，而 startup session 的执行权仍保持在宿主。

同一条规则现在也明确约束 `WorkbenchMenuBar/ResetLayout`：即使 builtin template 最终会触发 layout reset，它在 journal 里的 canonical event 仍然必须是 `MenuAction::ResetLayout`，而不是 template bridge 自己提前改写成 `LayoutCommand::ResetToDefault`。这样旧手写 Slint 菜单入口、reflection/nativeBinding 菜单入口、以及新的 template-host 菜单入口，才能在 same-fixture parity 测试里产出同构 `EditorEventRecord`。

### Docking And Tool Window Shell

- source: `ActivityRail/*`, `DocumentTabs/*`, `ToolWindow/*`
- payload: `DockCommand`
- dispatch entry: `dispatch_docking_binding`
- result: `LayoutCommand`

`DockCommand` 当前覆盖：

- focus/close view
- attach view to drawer/document
- detach to window
- activate drawer tab
- activate main page
- set drawer mode
- set drawer extent
- save/load preset
- reset to default

rail click、drawer tab 激活、stack 展开/折叠都应继续走这条 typed docking path。

当前 runtime 里，Slint `Window` 菜单上的 preset 条目仍通过宿主菜单回调进入 `LayoutCommand::SavePreset/LoadPreset`，但稳定协议层已经把它们定成 `DockCommand::SavePreset/LoadPreset`。这保证了 headless / reflection / nativeBinding 可以直接走 typed contract，而不是复制一套字符串分派。
当前 runtime 里，Slint `Window` 菜单上的 preset 条目仍来自 legacy menu callback，因为 builtin template 还不会实例化每个动态 preset item；但字符串归一化已经不再停在 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs)。现在的真实链路是：

- `dispatch_menu_action(...)`
  -> [`slint_menu_action(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs)
  -> `EditorEvent::Layout(core::editor_event::LayoutCommand::SavePreset/LoadPreset)`
  -> `SlintDispatchEffects.active_layout_preset_name`
  -> 宿主只消费 effect，更新当前 preset 选择

因此这里保留下来的只是动态菜单项来源，而不是 `app.rs` 的本地字符串 special-case。稳定协议层仍然是 `DockCommand::SavePreset/LoadPreset`，headless / reflection / nativeBinding 仍可直接走 typed contract，而不用复制一套前缀解析。

同一轮里，scene 空态动作也从宿主 label fallback 收进了同一条 menu/runtime 链：

- `PaneActionModel("Open Scene" / "Create Scene")` 现在直接携带 `menu_action_binding(&MenuAction::OpenScene/CreateScene)`
- `slint_host/ui.rs` 不再需要根据按钮文案硬编码 `"OpenScene"` / `"CreateScene"`
- `dispatch_menu_action(...)` 会把它们和其他 menu action 一样送进 `EditorEventRuntime`
- 当前 runtime 仍只返回占位状态 `"Scene open/create workflow is not wired yet"`，但占位语义已经在 runtime，而不是 `app.rs`

当前真实 Slint tab drag/drop 仍是宿主内部回调，不是反射公开协议：

- Slint tab 释放时先落成内部 `drop_tab(tab_id, target_group)` 回调
- `target_group` 只表达 shell 级别的 `left / right / bottom / document`
- `target_group` 的来源现在是 host-owned `active_drag_target_group`，不再是 Slint 自己算出来的字符串属性
- 宿主再把它映射到现有 `AttachView` / 条件性 `SetDrawerMode(Pinned)` 语义

这样做的原因是：pointer drag 本身是本地桌面手势，不适合作为当前这轮的远控协议表面；真正对外稳定的 attach 语义仍然由 `DockCommand::AttachViewToDrawer` 和 `DockCommand::AttachViewToDocument` 承担。

这里现在多了一条明确的宿主规范：`dispatch_tab_drop(...)` 只会在目标 drawer 当前为 `Collapsed` 时补发 `SetDrawerMode(Pinned)`；如果目标 drawer 已经 `Pinned` 或 `AutoHide`，route dispatcher 只记录 `AttachView` 并保留现有模式。这样 shared pointer route -> normalized drop route -> typed layout dispatch 的 journal 不会因为宿主兼容层而多出冗余 reopen event。

同理，当前 Slint splitters 也先走宿主内部回调：

- `set_drawer_extent(target_group, extent)` 只表达 shell 级别的 `left / right / bottom`
- 宿主把 group fan-out 到对应 drawer slots，再落成 `LayoutCommand::SetDrawerExtent`
- 稳定协议层继续把这类行为定义为 `DockCommand::SetDrawerExtent`，而不是公开桌面手势细节

这样可以把高频 pointer resize 保留在本地宿主里，同时让 headless / reflection / binding roundtrip 仍然对齐同一个 typed docking 语义。

### Selection Sync

- source: hierarchy / scene related controls
- payload: `SelectionCommand`
- dispatch entry: `dispatch_selection_binding`
- apply entry: `apply_selection_binding`
- result: `SelectionHostEvent` or `EditorIntent::SelectNode`

当前已经落地的 typed selection command 是：

- `SelectionCommand::SelectSceneNode`

这条链路的意义是：层级树点击、viewport 点击、headless 绑定调用，最终都收敛到同一条 selection intent，而不是一个地方直接改状态、另一个地方发字符串消息。

### Asset Intent

- source: project/assets related controls
- payload: `AssetCommand`
- dispatch entry: `dispatch_asset_binding`
- result: `AssetHostEvent`

当前已经落地的 typed asset command 是：

- `AssetCommand::OpenAsset`
- `AssetCommand::ImportModel`

这里先锁协议，不强行在这一轮补完完整 asset browser 逻辑。关键约束是：asset open/reveal/import 不能继续走匿名字符串解析。
其中 `AssetCommand::ImportModel` 当前明确走的是“runtime 归一化 + host effect 请求”边界：

- runtime 归一化成 canonical `EditorEvent::Asset(EditorAssetEvent::ImportModel)`
- runtime 只产出 `EditorEventEffect::ImportModelRequested`
- 真正的文件复制、asset import、resource resolve 和 mesh 注入仍然由宿主 effect 消费侧执行

### Welcome Startup Intent

- source: `WelcomeSurface/*`
- payload: `WelcomeCommand`
- dispatch entry: `dispatch_welcome_binding`
- result: `WelcomeHostEvent`

当前 welcome typed command 覆盖：

- project name edit
- location edit
- create project
- open existing project
- open/remove recent project

这一层当前刻意停在 host event，而不是继续伪装成 `EditorEvent`。原因不是协议还没成形，而是 startup session 的执行权还在宿主：

- `WelcomeCommand` 负责稳定 surface 协议
- `WelcomeHostEvent` 负责 typed host dispatch
- `SlintEditorHost` 负责 `EditorStartupSessionDocument` 生命周期与 `EditorManager` 调用

### Inspector Commit

- source: `InspectorView/ApplyBatchButton`
- payload: `InspectorFieldBatch`
- dispatch entry: `dispatch_inspector_binding`
- apply entry: `apply_inspector_binding`

`InspectorFieldBatch` 仍然是 inspector 唯一允许的持久化属性编辑入口。  
selection 改变可以刷新 inspector subject，但真正提交属性改动仍然只允许这一条 batch path。

### Viewport

- source: viewport surface / viewport toolbar
- payload: `ViewportCommand`
- dispatch entry: `dispatch_viewport_binding`
- apply entry: `apply_viewport_binding`
- result: `ViewportInput` or viewport feedback

当前已经落地的 typed viewport path 覆盖：

- pointer move
- left/right/middle press/release
- scroll
- resize

当前桌面宿主里的 viewport pointer/scroll 已经不是直接由 Slint callback 手写映射成 `EditorViewportEvent`：

- Slint callback 先生成 shared `UiPointerEvent`
- `SharedViewportPointerBridge` 在最小 retained `UiSurface` 上完成 target、capture 和 route 派发
- 只有命中 viewport 的结果才继续映射成 editor runtime 的 `EditorViewportEvent`
- release 会复用当前 capture / cursor 状态，即使光标已移出 viewport hit bounds 也不会丢失 `Up`
- callback dispatch 还补齐了 `dispatch_viewport_command(...)` / `viewport_event_from_command(...)` 这条 typed helper path，保证 toolbar/state 级别的 `ViewportCommand` 也会落回同一套 runtime 语义

root shell projection 现在把真实 `PaneSurfaceRoot` 作为 viewport content frame 的优先真源：当 workbench/drawer 可见时，渲染尺寸和 pointer 目标区域从 pane surface 扣掉 viewport toolbar 高度后得到，而不是继续用较早的 document-root/document-tabs fallback。即使 shared Slint 投影已经领先、旧 `WorkbenchShellGeometry` 还没有同步出 drawer frame，也必须以 `PaneSurfaceRoot` 为准。这样 `SlintEditorHost::viewport_size`、host presentation 的 `viewport_content_frame`、以及 `PaneSurface` 中实际显示的 scene canvas 会使用同一套坐标和尺寸。

viewport image polling 也不再在底层 renderer 返回 `None` 时重复发布 `latest_image` 缓存。无新帧时宿主保持静默，避免 UI tick 把上一帧反复推回 Slint，降低静态场景下的无效刷新和卡顿。

viewport toolbar 的 typed command 空间已经固定属于 `ViewportCommand`，即使某些具体 toolbar action 还没完全接入 runtime state，也不能再退回 `Custom("TranslateTool")` 这种匿名协议。

当前 runtime 还承担了 viewport gizmo drag 的 editor 语义状态：

- begin / drag / end 仍由桌面宿主采集 pointer 输入
- 是否进入 gizmo drag 由 runtime 内部 `dragging_gizmo` bookkeeping 决定
- scene 变换、render dirtiness、journal record 都继续走统一 dispatcher path

## Reflection Contract

`zircon_editor/src/ui/workbench/reflection/mod.rs` 现在作为结构入口，把 workbench snapshot 和 view model 投影拆成 activity descriptor、activity collection、typed route registration 与名称映射几个独立子模块。

反射树中固定暴露：

- menu item node
- page node
- drawer node
- floating window node
- activity node

activity node 上暴露的动作必须来自 typed payload：

- menu item -> `MenuAction`
- focus/detach -> `DockCommand`
- inspector apply -> `InspectorFieldBatch`
- inspector live edit -> `DraftCommand.SetInspectorField`
- assets mesh import path edit -> `DraftCommand.SetMeshImportPath`
- assets import submit -> `AssetCommand.ImportModel`
- scene/game pointer actions -> `ViewportCommand`

远控现在可以通过 `CallAction`、`InvokeRoute` 或 `InvokeBinding` 进入这些动作，但执行不再发生在 `EditorUiControlService` 闭包里。  
`register_workbench_reflection_routes` 只注册 stub route metadata，真正执行统一回到 `EditorEventRuntime::handle_control_request(...)`。

## Reflection Rebuild Inputs

runtime 每次事件执行后只允许从三类输入重建 reflection：

- `EditorState` 的稳定 editor 数据快照
- `EditorManager` 的 workbench/layout/view 实例快照
- `EditorTransientUiState` 的 hover/focus/pressed/resizing/drag 状态

这意味着：

- 反射树不再依赖 live Slint tree 查询 transient state
- `CallAction` / `InvokeBinding` 返回的结果来自真实 editor 行为，而不是 preview JSON
- replay session 可以重建同一条 reflection surface，而不是只能回放一堆 widget callback 名字

## Hot-Path Rule

实现约束保持不变：

- 正常 UI 热路径不依赖字符串 parse
- route id 和 `nativeBinding` 最终进入同一 handler
- string formatting/parse 只保留给稳定协议、测试和远控

这也是为什么 shell 相关行为需要拆成 `DockCommand / SelectionCommand / AssetCommand / ViewportCommand`：它们既能提供稳定 wire format，又不会逼迫宿主在高频路径上反复做字符串分派。

## UI Asset Editor Structured Authoring Route

`UI Asset Editor` 这一轮也被钉在同样的 typed session/host seam 上，而不是回退到字符串 callback：

- [`UiDesignerSelectionModel`](/E:/Git/ZirconEngine/zircon_editor/src/ui/reflection.rs) 现在是 Source、Hierarchy、Canvas 共享的 selection payload；[`reconcile_selection(...)`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/session.rs) 会在 parse/tree edit 后按稳定 `node_id` 重建 parent、mount 和 sibling multi-select
- [`UiAssetEditorCommand::TreeEdit`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/command.rs) 已支持附带 `next_selection`；[`UiAssetEditorUndoStack`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/undo_stack.rs) 记录的是 `source + selection` snapshot，因此 undo/redo 恢复的是结构化 authoring state，而不只是 source text
- [`binding_inspector.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/binding_inspector.rs) 现在把 `UiBindingRef` 投影成 `event kind + action kind + payload entries` 三段式 inspector；事件来自 `UiEventKind`，动作和 payload 来自 `UiActionRef`
- manager 和 Slint host 只转发 palette index、binding index、payload key 以及 `canvas.reparent.*` / `palette.insert.*` 这类稳定 action id；真正的 AST 变更始终由 [`UiAssetEditorSession`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/session.rs) 执行
- Source 模式和 Design 模式仍共用同一条 session authority：parse 成功才刷新 preview/reflection，失败时保留 last-good preview 和 roundtrip diagnostics

## Current Coverage

当前已经有自动化覆盖的点：

- `EditorUiBindingPayload` roundtrip
  - `MenuAction`
  - `InspectorFieldBatch`
  - `SelectionCommand`
  - `AssetCommand`
  - `WelcomeCommand`
  - `DraftCommand`
  - `DockCommand`
  - `ViewportCommand`
- representative shell bindings
  - `WorkbenchMenuBar/OpenProject`
  - `ActivityRail/ProjectToggle`
  - `InspectorView/ApplyBatchButton`
- `editor_event` runtime normalization equivalence
  - Slint adapter input
  - `EditorUiBinding`
  - reflection `CallAction`
- live draft dispatch
  - inspector field draft
  - mesh import path draft
- host-effect asset dispatch
  - import model request
- session-local journal serialization / replay
- transient reflection projection
  - hovered
  - focused
  - pressed
  - drawer resizing
- `zircon_editor` host dispatch
  - menu dispatch
  - selection dispatch + apply
  - asset dispatch
  - docking dispatch
  - inspector dispatch + apply
  - viewport dispatch + apply
- desktop Slint callback adapters
  - menu action
  - hierarchy selection
  - asset search
  - viewport pointer/scroll 先走 shared `UiSurface + UiPointerDispatcher` bridge
  - workbench shell pointer route 先走 host-owned `WorkbenchShellPointerBridge`
  - host page activation 与 welcome surface builtin template dispatch 已补 same-fixture parity
- Slint effect bridge
  - `EditorEventRecord.effects` -> `SlintDispatchEffects`
  - layout / render / presentation / asset sync fan-out
- workbench reflection route registration
  - menu
  - docking
  - inspector
  - viewport
  - runtime-backed `CallAction`
- UI asset editor session/host routes
  - source/hierarchy/canvas selection roundtrip
  - structured binding event/action/payload editing
  - palette insert / wrap / unwrap / reparent / outdent
  - source-selected block projection and undo/redo selection snapshots

## Validation Status

这次文档同步时重新跑过的验证命令：

- `cargo test -p zircon_ui shared_core -- --nocapture`
- `cargo test -p zircon_ui --locked`
- `cargo test -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor draft_command_bindings_parse_into_typed_payloads_instead_of_custom_calls --locked`
- `cargo test -p zircon_editor --lib draft_inspector_binding_normalizes_and_updates_live_snapshot --locked`
- `cargo test -p zircon_editor --lib draft_mesh_import_path_binding_normalizes_and_updates_live_snapshot --locked`
- `cargo test -p zircon_editor --lib inspector_draft_field_dispatch_updates_live_snapshot_without_scene_side_effects --locked`
- `cargo test -p zircon_editor --lib mesh_import_path_edit_dispatch_updates_live_snapshot_without_backend_sync --locked`
- `cargo test -p zircon_editor --lib workbench_reflection_model_projects_menu_and_activity_descriptors --locked`
- `cargo test -p zircon_editor --lib workbench_reflection_routes_mark_activity_actions_as_remotely_callable --locked`
- `cargo test -p zircon_editor --lib workbench_reflection_call_action_dispatches_typed_draft_actions --locked`
- `cargo test -p zircon_editor --lib asset_command_binding_roundtrips_for_import_model --locked`
- `cargo test -p zircon_editor --lib asset_import_binding_normalizes_to_runtime_host_request --locked`
- `cargo test -p zircon_editor --lib builtin_asset_surface_import_model_dispatches_host_request_from_template --locked`
- `cargo test -p zircon_editor --lib workbench_reflection_call_action_dispatches_asset_import_action --locked`

后续在同一工作区继续推进后，又额外通过了：

- `cargo test -p zircon_editor --locked`
- `cargo test --workspace --locked`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_`
- `cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_`
- `cargo test -p zircon_editor --lib --locked editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo`
- `cargo test -p zircon_editor --lib --locked tests::host::slint_window::child_window_callback_wiring_tracks_source_window_for_pane_interactions`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_`
- `cargo check -p zircon_editor --lib --locked --jobs 1`
- `cargo test -p zircon_editor --lib root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_collapsed --locked --jobs 1 -- --nocapture`
- `cargo test -p zircon_editor --lib root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_visible --locked --jobs 1 -- --nocapture`
- `cargo test -p zircon_editor --lib apply_presentation_prefers_shared_root_projection_for_visible_drawer_document_region --locked --jobs 1 -- --nocapture`
- `cargo test -p zircon_editor --lib controller_does_not_republish_cached_image_when_no_new_frame_is_available --locked --jobs 1 -- --nocapture`
- `cargo test -p zircon_editor --lib controller_does_not_republish_unchanged_captured_frame --locked --jobs 1 -- --nocapture`
- `cargo check -p zircon_editor --lib --locked --jobs 1` after the shared-drawer-leads-geometry viewport projection correction
- WSL validation setup: installed `pkg-config` and `libfontconfig-dev` in Ubuntu 22.04 so Slint/fontconfig test builds can link under Linux
- `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-wsl-layout RUSTFLAGS='-C debuginfo=0' cargo test -p zircon_editor --lib apply_presentation_prefers_pane_surface_viewport_when_shared_drawer_leads_geometry --locked --jobs 1 -- --nocapture`
- `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-wsl-layout RUSTFLAGS='-C debuginfo=0' cargo test -p zircon_editor --lib apply_presentation_prefers_shared_root_projection_for_visible_drawer_document_region --locked --jobs 1 -- --nocapture`
- `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-wsl-layout RUSTFLAGS='-C debuginfo=0' cargo test -p zircon_editor --lib root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_visible --locked --jobs 1 -- --nocapture`
- `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-wsl-layout RUSTFLAGS='-C debuginfo=0' cargo test -p zircon_editor --lib root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_collapsed --locked --jobs 1 -- --nocapture`
- `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-wsl-layout RUSTFLAGS='-C debuginfo=0' cargo test -p zircon_editor --lib controller_does_not_republish_cached_image_when_no_new_frame_is_available --locked --jobs 1 -- --nocapture`
- `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-wsl-layout RUSTFLAGS='-C debuginfo=0' cargo test -p zircon_editor --lib controller_does_not_republish_unchanged_captured_frame --locked --jobs 1 -- --nocapture`
- Listener-status WSL follow-up: `CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short` used `cargo 1.94.1` / `rustc 1.94.1`; early runs were blocked by unrelated runtime graphics private-field errors or a long editor compile timeout, then the accepted WSL path with `RUSTFLAGS="-C debuginfo=0"` passed both `cargo check -p zircon_editor --lib` and the focused `event_listener_control_reports_listener_status_with_pending_delivery_bounds` regression.

当前重新确认或由现有测试树持续覆盖的关键点包括：

- `UiNavigationDispatcher` 已经能从 focused route 冒泡，并在未聚焦时回退 root handler
- navigation handler 返回 `Focus(UiNodeId)` 时，会把 focus handoff 收口回 shared `UiSurface`
- unhandled navigation 在 shared core 上已经有 canonical fallback，不再要求 editor host 自己维护 tab order 或无焦点方向键起点
- viewport pointer/scroll callback 已经先经过 shared `UiSurface + UiPointerDispatcher`
- visible drawer 下的 viewport content frame 由 `PaneSurfaceRoot` 扣除 toolbar 后统一驱动 host viewport size 与渲染提交尺寸
- shared Slint 投影领先旧 geometry 时，viewport content frame 仍优先使用 `PaneSurfaceRoot`，避免短暂回退到 stale document frame
- renderer 没有新 captured frame 时，viewport controller 不再把缓存图像重复发布给 UI
- shared pointer capture 后移出 viewport hit bounds 仍会把 `Move` / `Up` 派回 viewport
- workbench shell drag target route 已经先经过 shared `UiSurface + UiPointerDispatcher`
- workbench shell splitter route 现在也先经过 `WorkbenchShellPointerBridge` 的 shared `UiSurface + UiPointerDispatcher`
- `UI Asset Editor` 的 Source、Hierarchy、Canvas 选中现在通过稳定 `node_id` 和 `UiDesignerSelectionModel` 做同源恢复
- `UI Asset Editor` 的 bindings inspector 现在走 `UiEventKind + UiActionRef + payload` 的结构化编辑，而不是宿主私有 callback 字符串
- tree-command undo/redo 现在会一起恢复 source block 定位与 inspector subject
- Slint `workbench.slint` 不再本地拥有 `drag_target_group` 公式，只保留 `active_drag_target_group` 展示态和 `update_drag_target(...)` 回调
- Slint callback adapter 通过 runtime dispatch 执行
- `CallAction` / `InvokeBinding` 继续命中真实 editor behavior
- global default layout 的 empty skeleton 现在会在 bootstrap 时被接受，并补回 builtin shell view placement
- `drawer_resize.rs` 与 `callback_dispatch.rs` 的测试 helper 只在 `cfg(test)` 下编译，不再给工作区构建引入死代码告警

## Remaining Work

这一轮已经完成了主线桌面宿主接线：

- `slint_host/app.rs` 的 menu / docking / hierarchy / inspector / asset / viewport callback 已经改成 adapter + runtime dispatch
- `InvokeBinding` / `CallAction` / Slint callback 现在可以对齐到同一个 normalized event path
- reflection rebuild 输入已经收敛到 `EditorState + EditorManager + EditorTransientUiState`

剩下的工作主要是扩展 catalog，而不是继续拆 ownership：

- viewport toolbar / gizmo 的更多 typed action 并入 normalized event catalog
- asset browser / project explorer 的更完整 action catalog
- MCP / network transport adapter 直接消费 runtime journal 和 reflection surface
- 非 scene editor-domain undo 继续在当前 journal / undo metadata 上扩展

也就是说，这一轮已经把“谁拥有执行权”“日志记录什么”“反射动作如何执行”定死了；后续主要是把更多 editor 行为填进同一 dispatcher，而不是再把语义退回 UI 库。
