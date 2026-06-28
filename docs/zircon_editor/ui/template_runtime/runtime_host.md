---
related_code:
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_runtime/src/ui/theme/mod.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/host_nodes.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/host_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/host_projection.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/tests/ui/asset_browser/bootstrap_assets.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/asset_editor/session/v2_authoring.rs
  - zircon_editor/assets/ui/editor/host/editor_main_frame.zui
  - zircon_editor/assets/ui/editor/host/workbench_shell.zui
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - zircon_editor/assets/ui/editor/host/console_body.zui
  - zircon_editor/assets/ui/editor/host/inspector_body.zui
  - zircon_editor/assets/ui/editor/host/hierarchy_body.zui
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
  - zircon_editor/assets/ui/editor/host/floating_window_source.zui
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.zui
  - zircon_editor/assets/ui/editor/host/animation_graph_body.zui
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.zui
  - zircon_editor/assets/ui/editor/host/performance_timeline_body.zui
  - zircon_editor/assets/ui/editor/host/module_plugins_body.zui
  - zircon_editor/assets/ui/editor/host/build_export_desktop_body.zui
  - zircon_editor/assets/ui/editor/project_overview.zui
  - zircon_editor/assets/ui/editor/asset_browser.zui
  - zircon_editor/assets/ui/editor/console.zui
  - zircon_editor/assets/ui/editor/hierarchy.zui
  - zircon_editor/assets/ui/editor/inspector.zui
  - zircon_editor/assets/ui/editor/assets_activity.zui
  - zircon_editor/assets/ui/editor/animation_editor.zui
  - zircon_editor/assets/ui/editor/welcome.zui
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.zui
  - zircon_editor/assets/ui/editor/workbench_menu_popup.zui
  - zircon_editor/assets/ui/editor/workbench_page_chrome.zui
  - zircon_editor/assets/ui/editor/workbench_dock_header.zui
  - zircon_editor/assets/ui/editor/workbench_status_bar.zui
  - zircon_editor/assets/ui/editor/workbench_activity_rail.zui
  - zircon_editor/assets/ui/editor/component_showcase.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_command_toolbar.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_bottom_log.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_category_nav.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_state_panel.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_visual_section.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_input_section.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_selection_section.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_collections_section.zui
  - zircon_editor/src/ui/template_runtime/showcase_demo_state/categories.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state/defaults.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_category.rs
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs
  - zircon_editor/assets/ui/theme/editor_base.zui
  - zircon_editor/assets/ui/theme/editor_unreal_dark.zui
  - zircon_editor/assets/ui/editor/ui_asset_editor.zui
implementation_files:
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/host_nodes.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/host_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/host_projection.rs
  - zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state/categories.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state/defaults.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_category.rs
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/asset_editor/session/v2_authoring.rs
  - zircon_editor/assets/ui/editor/host/editor_main_frame.zui
  - zircon_editor/assets/ui/editor/host/workbench_shell.zui
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - zircon_editor/assets/ui/editor/host/console_body.zui
  - zircon_editor/assets/ui/editor/host/inspector_body.zui
  - zircon_editor/assets/ui/editor/host/hierarchy_body.zui
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
  - zircon_editor/assets/ui/editor/host/floating_window_source.zui
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.zui
  - zircon_editor/assets/ui/editor/host/animation_graph_body.zui
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.zui
  - zircon_editor/assets/ui/editor/host/performance_timeline_body.zui
  - zircon_editor/assets/ui/editor/host/module_plugins_body.zui
  - zircon_editor/assets/ui/editor/host/build_export_desktop_body.zui
  - zircon_editor/assets/ui/editor/project_overview.zui
  - zircon_editor/assets/ui/editor/asset_browser.zui
  - zircon_editor/assets/ui/editor/console.zui
  - zircon_editor/assets/ui/editor/hierarchy.zui
  - zircon_editor/assets/ui/editor/inspector.zui
  - zircon_editor/assets/ui/editor/assets_activity.zui
  - zircon_editor/assets/ui/editor/animation_editor.zui
  - zircon_editor/assets/ui/editor/welcome.zui
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.zui
  - zircon_editor/assets/ui/editor/workbench_menu_popup.zui
  - zircon_editor/assets/ui/editor/workbench_page_chrome.zui
  - zircon_editor/assets/ui/editor/workbench_dock_header.zui
  - zircon_editor/assets/ui/editor/workbench_status_bar.zui
  - zircon_editor/assets/ui/editor/workbench_activity_rail.zui
  - zircon_editor/assets/ui/editor/component_showcase.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_command_toolbar.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_bottom_log.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_category_nav.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_state_panel.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_visual_section.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_input_section.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_selection_section.zui
  - zircon_editor/assets/ui/editor/components/showcase\showcase_collections_section.zui
  - zircon_editor/assets/ui/theme/editor_base.zui
  - zircon_editor/assets/ui/theme/editor_unreal_dark.zui
  - zircon_editor/assets/ui/editor/ui_asset_editor.zui
plan_sources:
  - user: 2026-05-11 hard-cut workbench host and core panes to UI v2
  - user: 2026-05-13 migrate UI Asset Editor authoring support to v2 so old schema assets can keep being removed
  - .codex/plans/Zircon Editor Demo 首屏与 .zui 组件陈列计划.md
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
tests:
  - rustfmt --edition 2021 --check zircon_editor\src\ui\template_runtime\runtime\build_session.rs
  - cargo check -p zircon_editor (2026-05-11: passed)
  - cargo test -p zircon_editor builtin_template_compile_cache_is_reused_across_runtime_instances -- --nocapture (2026-05-11: passed)
  - cargo test -p zircon_editor template_assets -- --nocapture (2026-05-11: passed, 10 passed)
  - cargo test -p zircon_editor viewport_toolbar -- --nocapture (2026-05-11: passed, 23 passed)
  - cargo test -p zircon_editor workbench_projection -- --nocapture (2026-05-11: passed, 12 passed)
  - cargo test -p zircon_editor bootstrap_assets -- --nocapture (2026-05-11: passed, 24 passed)
  - cargo test -p zircon_editor --lib asset_browser_projection_maps_bootstrap_asset_into_mount_nodes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26 Asset Browser utility spacing: passed, 1 passed)
  - cargo test -p zircon_editor boundary -- --nocapture (2026-05-11: passed, 72 passed)
  - cargo test -p zircon_editor retained_menu_pointer -- --nocapture (2026-05-11: passed, 21 passed, 4 ignored)
  - cargo test -p zircon_editor retained_activity_rail_pointer -- --nocapture (2026-05-11: passed, 6 passed)
  - cargo test -p zircon_editor component_showcase -- --nocapture (2026-05-11: passed, 19 passed)
  - cargo test -p zircon_editor --lib component_showcase_imported_zui_components_are_single_component_assets --locked --target-dir target/codex-shared-b (2026-05-15: passed, 1 passed)
  - cargo test -p zircon_editor --lib showcase_category_selection_filters_projected_demo_controls --locked --target-dir target/codex-shared-b --message-format short (2026-05-15 category-nav .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_is_hard_cut_to_v2_catalog_components --locked --target-dir target/codex-shared-b --message-format short (2026-05-15 category-nav .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_imported_zui_components_are_single_component_assets --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 state-panel .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_is_hard_cut_to_v2_catalog_components --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 state-panel .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib showcase_demo_state_applies_projected_bindings_to_retained_values_and_log --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 state-panel .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_imported_zui_components_are_single_component_assets --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 visual-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_is_hard_cut_to_v2_catalog_components --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 visual-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_projection_carries_runtime_component_semantics --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 visual-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_imported_zui_components_are_single_component_assets --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 input-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_is_hard_cut_to_v2_catalog_components --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 input-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib showcase_demo_state_applies_projected_bindings_to_retained_values_and_log --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 input-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib showcase_demo_state_exercises_full_component_action_bindings --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 input-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_imported_zui_components_are_single_component_assets --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 selection-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_is_hard_cut_to_v2_catalog_components --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 selection-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_selection --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 selection-section .zui extraction: passed, 2 passed)
  - cargo test -p zircon_editor --lib component_showcase_reference --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 selection-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_imported_zui_components_are_single_component_assets --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 collections-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_is_hard_cut_to_v2_catalog_components --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 collections-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_structure --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 collections-section .zui extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_option --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 collections-section .zui extraction: passed, 2 passed)
  - cargo test -p zircon_editor --lib component_showcase_projection_carries_runtime_component_semantics --locked --target-dir target/codex-shared-b (2026-05-15: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase_template_nodes_preserve_scroll_clip_frames --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase --locked --target-dir target/codex-shared-b --message-format short -- --test-threads=1 (2026-05-15: passed, 24 passed)
  - cargo test -p zircon_editor --lib component_showcase --locked --target-dir target/codex-zui-state-panel --message-format short -- --test-threads=1 (2026-05-15 state-panel .zui extraction: passed, 24 passed)
  - cargo test -p zircon_editor --lib component_showcase --locked --target-dir target/codex-zui-state-panel --message-format short -- --test-threads=1 (2026-05-15 visual-section .zui extraction: passed, 24 passed)
  - cargo test -p zircon_editor --lib component_showcase --locked --target-dir target/codex-zui-state-panel --message-format short -- --test-threads=1 (2026-05-15 input-section .zui extraction: passed, 24 passed)
  - cargo test -p zircon_editor --lib component_showcase --locked --target-dir target/codex-zui-state-panel --message-format short -- --test-threads=1 (2026-05-15 selection-section .zui extraction: passed, 24 passed)
  - cargo test -p zircon_editor --lib component_showcase --locked --target-dir target/codex-zui-state-panel --message-format short -- --test-threads=1 (2026-05-15 collections-section .zui extraction: passed, 24 passed)
  - cargo test -p zircon_editor --lib component_showcase_template_metadata_is_owned_by_rust_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-menu-normalization-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 component showcase Rust action-id markers: passed, 1 passed / 1870 filtered)
  - cargo test -p zircon_editor --lib component_showcase_option_and_action_callbacks_are_rust_wired --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-menu-normalization-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 AssetField action suffix constants: passed, 1 passed)
  - cargo test -p zircon_editor --lib template_nodes --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed, 7 passed)
  - cargo test -p zircon_editor --lib dual_host_parity_preserves_layout_attributes_and_routes_for_representative_documents --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed, 1 passed)
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed)
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 state-panel .zui extraction: passed)
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 visual-section .zui extraction: passed)
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 input-section .zui extraction: passed)
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 selection-section .zui extraction: passed)
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-zui-state-panel --message-format short (2026-05-15 collections-section .zui extraction: passed)
  - git diff --check -- zircon_editor/assets/ui/editor/component_showcase.zui zircon_editor/assets/ui/editor/components/showcase\showcase_input_section.zui zircon_editor/src/tests/ui/boundary/template_assets.rs zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs docs/ui-and-layout/runtime-ui-component-showcase.md docs/zircon_runtime/ui/v2.md docs/zircon_editor/ui/template_runtime/runtime_host.md .codex/sessions/20260515-0832-showcase-zui-input.md (2026-05-15 input-section .zui extraction: passed with only Windows LF-to-CRLF notices)
  - git diff --check -- zircon_editor/assets/ui/editor/component_showcase.zui zircon_editor/assets/ui/editor/components/showcase\showcase_selection_section.zui zircon_editor/src/tests/ui/boundary/template_assets.rs zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs zircon_editor/src/ui/retained_host/ui/reference_component_tests.rs zircon_editor/src/ui/retained_host/ui/structure_component_tests.rs docs/ui-and-layout/runtime-ui-component-showcase.md docs/zircon_runtime/ui/v2.md docs/zircon_editor/ui/template_runtime/runtime_host.md .codex/sessions/20260515-0850-showcase-zui-selection.md (2026-05-15 selection-section .zui extraction: passed with only Windows LF-to-CRLF notices)
  - git diff --check -- zircon_editor/assets/ui/editor/component_showcase.zui zircon_editor/assets/ui/editor/components/showcase\showcase_selection_section.zui zircon_editor/assets/ui/editor/components/showcase\showcase_collections_section.zui zircon_editor/src/tests/ui/boundary/template_assets.rs zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs zircon_editor/src/ui/retained_host/ui/reference_component_tests.rs zircon_editor/src/ui/retained_host/ui/structure_component_tests.rs docs/ui-and-layout/runtime-ui-component-showcase.md docs/zircon_runtime/ui/v2.md docs/zircon_editor/ui/template_runtime/runtime_host.md .codex/sessions/20260515-0850-showcase-zui-selection.md (2026-05-15 collections-section .zui extraction: passed with only Windows LF-to-CRLF notices)
  - python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (2026-05-15: passed)
  - python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (2026-05-15 state-panel .zui extraction: passed and included showcase_state_panel.zui in the editor build output)
  - python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (2026-05-15 visual-section .zui extraction: passed and included showcase_visual_section.zui in the editor build output)
  - python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (2026-05-15 input-section .zui extraction: passed and included showcase_input_section.zui in the editor build output)
  - python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (2026-05-15 selection-section .zui extraction: passed and included showcase_selection_section.zui in the editor build output)
  - python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (2026-05-15 collections-section .zui extraction: passed and included showcase_collections_section.zui in the editor build output)
  - E:\zircon-build\ZirconEngine\zircon_editor.exe --list-operations --headless (2026-05-15: passed, includes window.ui_component_showcase.open)
  - E:\zircon-build\ZirconEngine\zircon_editor.exe --list-operations --headless (2026-05-15 state-panel .zui extraction: passed, includes window.ui_component_showcase.open)
  - E:\zircon-build\ZirconEngine\zircon_editor.exe --list-operations --headless (2026-05-15 visual-section .zui extraction: passed, includes window.ui_component_showcase.open)
  - E:\zircon-build\ZirconEngine\zircon_editor.exe --list-operations --headless (2026-05-15 input-section .zui extraction: passed, includes window.ui_component_showcase.open)
  - E:\zircon-build\ZirconEngine\zircon_editor.exe --list-operations --headless (2026-05-15 selection-section .zui extraction: passed, includes window.ui_component_showcase.open)
  - E:\zircon-build\ZirconEngine\zircon_editor.exe --list-operations --headless (2026-05-15 collections-section .zui extraction: passed, includes window.ui_component_showcase.open)
  - .codex/run-logs/editor-noargs-smoke-polished.png (2026-05-15: no-argument editor smoke screenshot, Component Showcase first screen visible without bottom-log overlap)
  - .codex/run-logs/editor-noargs-smoke-zui-gallery.png (2026-05-15: no-argument editor smoke screenshot after all four center gallery sections moved to `.zui`; Component Showcase first screen visible with category nav, gallery, state panel, and bottom log)
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib pane_body_documents --locked --jobs 1 --message-format=short -- --test-threads=1 (11 passed)
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib drawer_toggle --locked --jobs 1 --message-format=short -- --test-threads=1 (3 passed)
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib repository_assets --locked --jobs 1 --message-format=short -- --test-threads=1 (1 passed)
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib dock_and_welcome --locked --jobs 1 --message-format=short -- --test-threads=1 (5 passed)
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib template_runtime --locked --jobs 1 --message-format=short -- --test-threads=1 (48 passed)
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib retained_callback_dispatch::template_bridge --locked --jobs 1 --message-format=short -- --test-threads=1 (7 passed)
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib floating_window_projection --locked --jobs 1 --message-format=short -- --test-threads=1 (12 passed)
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib shared_resize_target_route_resolves_left_right_and_bottom_splitters --locked --jobs 1 --message-format=short -- --test-threads=1 (1 passed)
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib --locked --jobs 1 --message-format=short -- --test-threads=1 (1298 passed, 4 ignored)
  - cargo test -p zircon_editor builtin_activity_window_documents_are_registered_in_host_runtime -- --nocapture (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_runtime --lib component_catalog -- --nocapture (2026-05-11: passed, 43 passed)
  - cargo test -p zircon_editor --lib ui_asset_editor_v2_authoring_instantiates_imported_component_slots_for_preview --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 1 passed)
  - cargo test -p zircon_editor --lib tests::ui::ui_asset_editor --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 40 passed)
  - cargo test -p zircon_editor --lib global_material_surface_assets_follow_responsive_contracts --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 1 passed)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\runtime_ui\runtime_ui_manager.rs zircon_editor\src\ui\template_runtime\runtime\runtime_host.rs zircon_editor\src\tests\host\template_runtime\shared_surface.rs (2026-06-12 active theme host path: passed)
  - cargo test -p zircon_editor --lib editor_ui_host_runtime_resolves_theme_tokens_for_v2_shared_surface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-host-theme-0612 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-12 active theme host path: timed out after 904 seconds while compiling; matching editor-host cargo/rustc processes were stopped, leaving unrelated runtime_absorption cargo/rustc processes untouched)
  - rustfmt --edition 2021 --check zircon_runtime_interface\src\ui\v2\style.rs zircon_runtime\src\ui\v2\style.rs zircon_runtime\src\ui\v2\surface_tree\node.rs zircon_runtime\src\ui\tests\v2_asset.rs zircon_editor\src\tests\host\template_runtime\shared_surface.rs zircon_runtime\src\ui\style.rs zircon_runtime\src\ui\tests\material_button_style.rs (2026-06-12 v2 style token provenance metadata: passed)
  - git diff --check -- zircon_runtime_interface/src/ui/v2/style.rs zircon_runtime/src/ui/v2/style.rs zircon_runtime/src/ui/v2/surface_tree/node.rs zircon_runtime/src/ui/tests/v2_asset.rs zircon_editor/src/tests/host/template_runtime/shared_surface.rs zircon_runtime/src/ui/style.rs zircon_runtime/src/ui/tests/material_button_style.rs docs/zircon_runtime/ui/v2.md docs/zircon_editor/ui/template_runtime/runtime_host.md .codex/sessions/20260612-0904-editor-ui-architecture-implementation.md (2026-06-12 v2 style token provenance metadata: passed with LF-to-CRLF warnings only)
  - cargo test -p zircon_editor --lib editor_ui_host_runtime_resolves_theme_tokens_for_v2_shared_surface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-host-theme-0612 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-12 style token provenance metadata assertions: not rerun in this slice after the same focused editor-host target timed out during cold compile earlier in the day)
  - cargo test -p zircon_editor --lib surface_backed_retained_projection_exposes_style_overrides_as_effective_properties --locked -- --nocapture (2026-06-16 surface-backed retained projection style overrides: passed, 1 passed / 2040 filtered)
  - cargo test -p zircon_editor --lib componentized_workbench_window_template_bridge_exposes_document_tab_runtime_routes --locked -- --nocapture (2026-06-16 Workbench document tab runtime routes: passed, 1 passed / 2040 filtered)
  - cargo test -p zircon_editor --lib componentized_workbench_window_template_bridge_exports_surface_projection_frames_and_routes --locked -- --nocapture (2026-06-16 Workbench surface projection frame/route baseline: passed, 1 passed / 2040 filtered)
  - python -c "import tomllib, pathlib; paths=[r'zircon_editor/assets/ui/theme/editor_base.zui', r'zircon_editor/assets/ui/theme/editor_material.zui', r'zircon_editor/assets/ui/editor/workbench_activity_rail.zui', r'zircon_editor/assets/ui/editor/workbench_status_bar.zui']; [tomllib.loads(pathlib.Path(p).read_text(encoding='utf-8')) for p in paths]" (2026-06-12 editor_base chrome theme role consumer: passed)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\tests\v2_asset.rs (2026-06-12 editor_base chrome theme role consumer: passed)
  - git diff --check -- zircon_editor/assets/ui/theme/editor_base.zui zircon_runtime/src/ui/tests/v2_asset.rs docs/zircon_runtime/ui/theme.md docs/zircon_runtime/ui/v2.md docs/zircon_editor/ui/template_runtime/runtime_host.md .codex/sessions/20260612-0904-editor-ui-architecture-implementation.md (2026-06-12 editor_base chrome theme role consumer: passed with LF-to-CRLF warnings only)
  - python tomllib parse zircon_editor/assets/ui/editor/components/showcase/showcase_selection_section.zui (2026-06-14 M3.S1 ContextMenu/DropdownPopup showcase visibility baseline: passed)
  - rustfmt --edition 2021 --check zircon_editor/src/tests/host/template_runtime/component_showcase_category.rs zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs zircon_editor/src/ui/template_runtime/showcase_demo_state/categories.rs zircon_editor/src/ui/template_runtime/showcase_demo_state/defaults.rs (2026-06-14 M3.S1 ContextMenu/DropdownPopup showcase visibility baseline: passed after formatting)
  - git diff --check -- zircon_editor/assets/ui/editor/components/showcase/showcase_selection_section.zui zircon_editor/src/ui/template_runtime/showcase_demo_state/categories.rs zircon_editor/src/ui/template_runtime/showcase_demo_state/defaults.rs zircon_editor/src/tests/host/template_runtime/component_showcase_category.rs zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs docs/plans/zircon_editor/editor_ui/06-component-library-mui.md docs/zircon_editor/ui/template_runtime/runtime_host.md .codex/sessions/20260612-0904-editor-ui-architecture-implementation.md (2026-06-14 M3.S1 ContextMenu/DropdownPopup showcase visibility baseline: passed with LF-to-CRLF warnings only)
  - cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked (2026-06-14 M3.S1 ContextMenu/DropdownPopup showcase visibility baseline: deferred because active cargo/rustc lanes were present in the shared Windows workspace)
  - cargo test -p zircon_editor --lib showcase_category_selection_filters_projected_demo_controls --locked (2026-06-14 M3.S1 ContextMenu/DropdownPopup showcase visibility baseline: deferred because active cargo/rustc lanes were present in the shared Windows workspace)
  - python tomllib parse zircon_editor/assets/ui/editor/components/showcase/showcase_visual_section.zui (2026-06-15 M3.S3 CommandPalette showcase visibility baseline: passed)
  - rustfmt --edition 2021 --check zircon_editor/src/ui/template_runtime/showcase_demo_state/categories.rs zircon_editor/src/ui/template_runtime/showcase_demo_state/defaults.rs zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs zircon_editor/src/tests/host/template_runtime/component_showcase_category.rs (2026-06-15 M3.S3 CommandPalette showcase visibility baseline: passed after formatting)
  - cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked (2026-06-15 M3.S3 CommandPalette showcase visibility baseline: deferred because active cargo/rustc lanes were present in the shared Windows workspace)
  - cargo test -p zircon_editor --lib showcase_category_selection_filters_projected_demo_controls --locked (2026-06-15 M3.S3 CommandPalette showcase visibility baseline: deferred because active cargo/rustc lanes were present in the shared Windows workspace)
  - cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 export panel retained projection: passed with existing warnings)
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 export panel retained projection: passed with existing warnings)
  - cargo test -p zircon_editor --lib export_wizard_panel_retained_projection --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --no-run --message-format short --color never (2026-06-15 export panel retained projection: blocked by unrelated zircon_editor/src/tests/editing/state.rs RenderQualityProfile::with_history_resolve compile drift)
doc_type: module-detail
---

# Template Runtime Host

`EditorUiHostRuntime` now keeps a v2 prototype store, a compiled v2 document table, and an active `UiThemeRegistry` beside the legacy template registry. Files ending in `.v2.ui.toml` are loaded through `UiV2AssetLoader`, inserted into `UiV2PrototypeStore`, and compiled with `UiV2DocumentCompiler::compile_with_prototype_store`. This makes composite component prototypes resident in heap-backed runtime state instead of reparsing a full recursive tree every time a document is projected.

## Projection Path

`project_document` and `project_pane_body` check the v2 compiled document table before falling back to the legacy template registry. V2 documents are projected from arena handles into retained host projections without re-instantiating the legacy `UiTemplateNode` tree. The arena projection uses an explicit stack, so deep v2 documents do not recurse through editor projection.

Pane payload injection is shared between old and v2 paths. Legacy panes still mutate a temporary `UiTemplateNode` before projection; v2 panes mutate the retained projection root and append any needed `HybridSlotAnchor` projection directly. This keeps existing Rust presenters and route IDs active while the pane body assets move to v2.

`build_shared_surface(...)` uses the same active theme registry when it materializes v2 documents into `UiSurface`. A v2 shared surface can therefore author `$theme.palette.accent`, `theme.palette.surface.1`, or `var(theme.palette.text.primary)` in static rules and runtime pseudo-state rules, and those values are resolved before `UiTemplateNodeMetadata` captures attributes and style overrides. The same build path also preserves `UiTemplateNodeMetadata.style_tokens`, so editor host diagnostics can see whether a final shared-surface color came from a document token, a normalized theme role, or a runtime pseudo-state override. Projection-only calls still use the compiled arena table and remain metadata-oriented until a shared surface is requested.

Shared-surface layout failures now flow through `EditorUiHostRuntimeError::UiTree`. That keeps retained host callers such as the desktop export panel projection on the same error boundary as template parsing, v2 asset loading, and template build failures when they compute layout before building a `RetainedUiHostProjection`.

The workbench chrome path now benefits from that same theme-aware materialization through `editor_base.zui`. Menu, page, dock, status, and activity-rail assets still author local chrome classes, but their ordinary base aliases delegate to central palette roles. The runtime guard loads `workbench_activity_rail.zui` and `workbench_status_bar.zui` as real imported documents and checks the resulting `style_tokens`, so host diagnostics can trace chrome colors back through chains such as `token.panel_bg -> theme.palette.surface.2` instead of seeing only legacy hex literals.

## Current Hard Cut

The builtin registry now routes these critical editor shell assets to v2:

- `editor_main_frame.zui`
- `workbench_shell.zui`
- `workbench_window.zui` (owns the componentized Workbench window and real bottom drawer shell)
- `floating_window_source.zui`
- `scene_viewport_toolbar.zui`
- `animation_sequence_body.zui`
- `animation_graph_body.zui`
- `runtime_diagnostics_body.zui`
- `module_plugins_body.zui`
- `build_export_desktop_body.zui`
- `console_body.zui`
- `inspector_body.zui`
- `hierarchy_body.zui`

The view projection layer now routes these top-level pane assets to v2:

- `project_overview.zui`
- `asset_browser.zui`
- `console.zui`
- `hierarchy.zui`
- `inspector.zui`
- `assets_activity.zui`
- `animation_editor.zui`
- `welcome.zui`

The Asset Browser v2 pane remains fully authored in `asset_browser.zui`. Its bottom utility region now keeps Preview/References/Metadata/Plugins as a compact retained template composition: the tab row, selection locator, preview visual, references split, metadata stack, and utility content/panel heights are all frame-locked by `asset_browser_projection_maps_bootstrap_asset_into_mount_nodes` before the retained host paints them. This keeps local container spacing in the authored asset instead of adding host-side layout exceptions for a single pane.

The shared workbench chrome projection now routes these root chrome assets to v2:

- `workbench_menu_chrome.zui`
- `workbench_menu_popup.zui`
- `workbench_page_chrome.zui`
- `workbench_dock_header.zui`
- `workbench_status_bar.zui`
- `workbench_activity_rail.zui`

The runtime component showcase is also now routed through `component_showcase.zui`. It no longer imports the old recursive `component_widgets.ui.toml#ShowcaseSection` or `material_meta_components.ui.toml#Material*` references on the builtin path. The v2 asset uses flat arena shell nodes while the demo-control sections live in imported `.zui` components, retaining existing control ids, event route ids, and Material measurement props so Rust callback dispatch and retained host projection continue to work. The showcase now imports `showcase_command_toolbar.zui#ShowcaseCommandToolbar`, `showcase_category_nav.zui#ShowcaseCategoryNav`, `showcase_visual_section.zui#ShowcaseVisualSection`, `showcase_input_section.zui#ShowcaseInputSection`, `showcase_selection_section.zui#ShowcaseSelectionSection`, `showcase_collections_section.zui#ShowcaseCollectionsSection`, `showcase_state_panel.zui#ShowcaseStatePanel`, and `showcase_bottom_log.zui#ShowcaseBottomLog`, proving the builtin runtime host can load `.zui` component prototypes from `res://` imports while the showcase root remains in the deprecated root-document suffix until the editor main asset M3 batch migrates it. The category nav prototype keeps the `UiComponentShowcase.SelectCategory.*` event routes inside the reusable component asset, the Visual/Feedback prototype keeps display-control ids inside a reusable gallery section, the Input/Numeric prototype keeps button/text/numeric/vector event routes inside a reusable gallery section, the Selection/References prototype keeps option and reference-field routes inside a reusable gallery section, the Collections/Inspector prototype keeps collection/tree/menu/paging routes inside a reusable gallery section, and the state-panel prototype keeps the retained diagnostic `PropertyRow` control ids inside a reusable component asset. `ShowcaseSelectionSection` now also owns real `ContextMenuDemo` and `DropdownPopupDemo` nodes with opened popup rows; `showcase_demo_state` keeps them visible only under Selection/All, maps them back to `ContextMenu`/`DropdownPopup`, and the retained host projection test verifies the resulting structured menu-item and option rows before native popup painters consume them. `ShowcaseVisualSection` now also owns an opened `CommandPaletteDemo` sample with workbench command-source defaults, `build` query filtering, selected/focused command state, and a disabled command row; `showcase_demo_state` keeps it visible only under Feedback/All while mapping it back to the real `CommandPalette`, and retained host tests verify the resulting popup role, query, options text, and structured command rows. Category filtering, visual component projection, input/numeric event projection, selection/reference projection, collection/structure projection, and state projection therefore still pass through the generic template runtime path.

`ShowcaseVisualSection` also owns an opened `NotificationCenterDemo` sample with two visible structured notification rows, unread state, selected and focused row metadata, and `NotificationCenter` component mapping. `showcase_demo_state` keeps it visible only under Feedback/All, and retained host tests verify the resulting notification-center role, popup state, options text, tone/description/unread metadata, selected row, and focused row before the native notification painter consumes the same structured row projection.

The top-level showcase asset also carries explicit Rust contract action-id markers for generated input actions such as `ui_component_showcase.number_field_drag_update` and `ui_component_showcase.input_field_changed`. Those markers keep the owned Rust metadata visible in the source bundle even though the routed `.zui` components still author their human-readable PascalCase event ids. `showcase_event_inputs.rs` keeps exact `AssetFieldClear`, `AssetFieldLocate`, and `AssetFieldOpen` suffix constants beside the normalized matching helper, so the callback-wiring guard can verify the asset-field controls stay Rust-wired after action id normalization.

The UI Asset Editor bootstrap layout is now `ui_asset_editor.zui`. UI Asset Editor sessions detect v2 source through `UiV2AssetLoader`, keep the last valid v2 document resident on the session, and serialize edited/canonical source back as v2 instead of downgrading authoring output to the old recursive schema. The deleted `ui_asset_editor.ui.toml` path is covered by the asset boundary guard, so UI authoring can no longer quietly reopen the old bootstrap asset.

The UI Asset Editor authoring preview now mirrors the runtime v2 prototype path for registered imports. `v2_authoring.rs` builds a `UiV2PrototypeStore` from the current v2 document plus registered component/style imports, compiles through `compile_with_prototype_store`, and leaves the current asset source as a flat v2 view with import references. That gives authoring preview the same external component expansion, named slot fill, and props/state patch behavior as the runtime v2 path without re-entering the old recursive template builder.

`pane_data_conversion` now builds a shared surface and computes layout before building the host model for template pane bodies. This lets v2 pane bodies contribute frame, clip, z-order, component metadata, and event bindings through the same host model path as older shared surfaces.

The 2026-05-15 continuation closed the stale demo-front/template blocker that had been hiding behind the GPU command-stream validation. `workbench_shell.zui` now treats Hierarchy and Assets as the current activity-rail drawer entries, with `ActivityRail/HierarchyToggle` and `ActivityRail/AssetsToggle` resolving to the matching view instances through `template_bindings.rs`. Pane body assets that host Rust-projected content declare explicit root slots: hierarchy tree, animation sequence timeline, animation graph canvas, module plugins list, build/export targets, and the new performance timeline frame list. `PerformanceTimelinePaneBody/RefreshSnapshot` is a registered focus-view command, so the body asset and binding table no longer drift.

The retained adapter maps v2 `HorizontalGroup` and `VerticalGroup` containers to the retained host box kinds, so v2 host-window and pane-body tests assert the current authored components rather than legacy `HorizontalBox` / `VerticalBox` source names. Repository asset tests now load the builtin v2 document table before registering `workbench_shell.zui`, which keeps v2 host-window projection, route IDs, and document tree IDs under one runtime host path. Full `zircon_editor --lib` validation now passes after these corrections.

`build_host_model_with_surface(...)` consumes the arranged surface tree as the spatial authority whenever layout has been computed. Host nodes use `UiArrangedNode.frame` and `UiArrangedNode.clip_frame`, so the retained host sees the same effective clip chain as shared rendering and hit testing. Metadata-only callers that pass an uncomputed surface still fall back to `UiTreeNode.layout_cache`, preserving the older route/property projection path. The arranged path is required for scroll panes such as the Component Showcase center gallery: descendants that are arranged below the visible `ScrollableBox` viewport still project their original frames for layout/debugging, but their `TemplatePaneNodeData.clip_frame` is bounded to the scroll viewport before native painting or template-node hit testing runs.

Workbench shell and viewport toolbar bridges now keep their `UiSurface` instances resident after initial load. Recompute marks the surface root dirty and calls `rebuild_dirty(...)` before projecting the updated retained host model, so these high-frequency bridge layouts no longer rebuild a fresh shared surface for every pointer-adjacent host refresh.

Surface-backed retained projection now carries both `metadata.attributes` and `metadata.style_overrides`. `RetainedUiHostNodeProjection` stores the style override map separately, `runtime/projection.rs` preserves it when host nodes are collected from a materialized surface, and `retained_adapter.rs` folds overrides over attributes when building effective retained properties. This mirrors runtime render extraction precedence: selector/style defaults remain visible for diagnostics, but inline or instance-level style values such as Workbench viewport gizmo foreground colors are the values consumed by retained painters, hit-test metadata, and template bridge assertions.

The componentized Workbench bridge uses that path for its current runtime route baseline. `workbench_viewport_panel.zui` mounts `DocumentTabsRoot` before the viewport toolbar/surface and binds activate/close events to dock commands. `workbench_component_drawer.zui` now relies on the v2 default-slot contract for `WorkbenchLabsTabs`, so the concrete `WorkbenchLabsTabOne/Two/Three` nodes are projected into the host contract rather than being lost inside the reusable tab strip. The bridge regression checks both behaviors in one retained path: document tab Change/Submit bindings normalize to dock commands, Labs tab click dispatch selects the expected tab, and the component drawer's disabled input, list, table, and tab samples all expose projected frames and runtime routes.

## Remaining Scope

The runtime host still has tree-template support for assets kept only as migration/test inputs. `ui_asset_editor.ui.toml` is no longer an exception and has been deleted; the remaining historical inputs are Material meta-component and additional authoring fixtures, such as `editor_widgets.zui`, `material_meta_components.zui`, `asset_browser.zui`, `binding_browser.zui`, `layout_workbench.zui`, `preview_state_lab.zui`, and `theme_browser.zui`. These fixtures now live under `zircon_editor/src/tests/fixtures/ui_zui/**`, outside the deployable asset roots. Staged `ZirconEngine/assets/ui/**` includes `.zui` UI templates only, and the guard test `packaged_ui_asset_roots_contain_only_v2_schema_files` prevents historical `.ui.toml` files from returning to active editor/runtime asset roots.
