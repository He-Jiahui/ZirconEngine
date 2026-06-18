---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_alert.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_dropdown.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_icon_button.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_list_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_segmented_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_selection_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_slider.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_toast.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_tooltip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_tree_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_option_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_menu_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-component-contract.mjs
  - zircon_runtime_interface/src/ui/style.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_alert.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_dropdown.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_icon_button.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_list_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_segmented_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_selection_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_slider.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_toast.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_tooltip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_tree_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_option_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_menu_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-component-contract.mjs
plan_sources:
  - user: 2026-06-03 componentized editor UI prototype and native replication request
  - .codex/plans/ZirconEngine 宿主编辑器 UI 基础能力计划.md
  - docs/ui-and-layout/ai-workbench-style/component-prototype/README.md
tests:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-component-contract.mjs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields.rs
  - node verify-native-component-contract.mjs (2026-06-11 passed after the native component contract checker followed runtime feedback state ownership in zircon_runtime/src/ui/surface/render/feedback/state.rs and pinned Workbench Chrome native selector/runtime extract coverage)
  - cargo test -p zircon_editor --lib template_fields --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib template_list_rows --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib template_dropdowns --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib dropdown_loading_state_uses_unavailable_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib slider_loading_state_uses_unavailable_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib template_sliders --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime_interface --lib ui_painter_style_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib selection_controls_loading_state_uses_unavailable_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib selection_control_loading_state_mutes_active_checked_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib template_selection_controls --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_selection_control.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls.rs zircon_runtime_interface/src/ui/style.rs zircon_runtime_interface/src/tests/ui_painter_style_contracts.rs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_slider.rs zircon_runtime_interface/src/ui/style.rs zircon_runtime_interface/src/tests/ui_painter_style_contracts.rs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_dropdown.rs
  - cargo test -p zircon_editor --lib template_popup_rows --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib popup_row_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1 (3 passed, 0 failed, 1879 filtered out; existing warnings only)
  - D:\cargo-targets\zircon-ui-style-selector-0607\debug\deps\zircon_editor-16c136b0ff3b6b9d.exe template_popup_rows --nocapture --test-threads=1 (4 passed, 0 failed, 1878 filtered out)
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs::runtime_component_projection_projects_popup_option_state_metadata_for_native_painter
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs::runtime_component_projection_projects_popup_menu_loading_flags_for_native_painter
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_menu_projection.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_option_projection.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs zircon_editor/src/ui/retained_host/ui/structure_component_tests.rs (2026-06-14: passed after native PopupRow projection parity baseline)
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs::popup_row_style_selector_matches_runtime_extract_state_matrix_for_projected_rows
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs (2026-06-14: passed after native PopupRow selector parity matrix baseline)
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs zircon_editor/src/tests/host/retained_window/native_material_painter.rs zircon_editor/src/tests/host/retained_window/native_material_painter_dialog.rs zircon_editor/src/tests/host/retained_window/native_material_painter_drag_overlay.rs (2026-06-15: passed after resolved button interaction-state convergence)
  - cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-component-showcase-0615 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-15: passed, 49 passed / 0 failed / 1945 filtered)
  - E:\cargo-targets\zircon-editor-ui-component-showcase-0615\debug\deps\zircon_editor-0fea0c836fb2d960.exe native_material_painter_dialog --nocapture --test-threads=1 (2026-06-15: passed, 3 passed / 0 failed / 1992 filtered; covers open Dialog painting, ConfirmDialog error/disabled-confirm styling, and closed no-fallback consumption)
  - cargo test -p zircon_editor --lib native_template_button --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-component-showcase-0615 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-15: passed, 3 passed / 0 failed / 1991 filtered)
  - E:\cargo-targets\zircon-editor-ui-component-showcase-0615\debug\deps\zircon_editor-0fea0c836fb2d960.exe drag_overlay --nocapture --test-threads=1 (2026-06-15: passed, 4 passed / 0 failed / 1991 filtered; covers open DragOverlay preview/drop indicator painting and closed no-fallback consumption)
  - E:\cargo-targets\zircon-editor-ui-component-showcase-0615\debug\deps\zircon_editor-0fea0c836fb2d960.exe notification_center --nocapture --test-threads=1 (2026-06-15: passed, 3 passed / 0 failed / 1992 filtered; covers open NotificationCenter panel/row painting and closed no-fallback consumption)
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs::standalone_dropdown_popup_paints_rows_inside_projected_popup_frame
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs::template_option_popup_frame_within_uses_projected_dropdown_popup_frame
  - rustfmt --edition 2021 --check zircon_editor/src/ui/layouts/views/view_projection.rs zircon_editor/src/ui/retained_host/ui/component_contract_metadata.rs zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/popup_frame.rs zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs (2026-06-14: passed after popup shell role/projected frame geometry baseline)
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row.rs
  - git diff --check -- zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row.rs docs/zircon_editor/ui/retained_host/host_contract/paint_template_nodes/style_selector.md
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_list_row.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_tree_row.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows.rs
  - cargo test -p zircon_editor --lib loading_state_uses_unavailable_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib loading_player_start_tree_row_mutes_special_icon_color --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib list_row_adornment_kind_prefers_disabled_then_selected_then_chevron --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - D:\cargo-targets\zircon-ui-style-selector-0607\debug\deps\zircon_editor-16c136b0ff3b6b9d.exe template_list_rows --nocapture --test-threads=1
  - D:\cargo-targets\zircon-ui-style-selector-0607\debug\deps\zircon_editor-16c136b0ff3b6b9d.exe template_tree_rows --nocapture --test-threads=1
  - D:\cargo-targets\zircon-ui-style-selector-0607\debug\deps\zircon_editor-16c136b0ff3b6b9d.exe template_table_rows --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib template_segmented_controls --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib template_alerts --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib template_tooltips --locked --jobs 1 --message-format short --color never
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_alert.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_tooltip.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_toast.rs
  - cargo test -p zircon_editor --lib alert_loading_state_uses_unavailable_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib tooltip_loading_state_uses_unavailable_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib toast_loading_state_uses_unavailable_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_segmented_control.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field.rs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_icon_button.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons.rs (passed after applying scoped rustfmt to the same files)
  - cargo test -p zircon_editor --lib button_loading_state_uses_unavailable_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1 (foreground command timed out during Windows test-binary compilation with no Rust diagnostics; the same target lane then produced `D:\cargo-targets\zircon-ui-style-selector-0607\debug\deps\zircon_editor-16c136b0ff3b6b9d.exe`)
  - D:\cargo-targets\zircon-ui-style-selector-0607\debug\deps\zircon_editor-16c136b0ff3b6b9d.exe button_loading_state_uses_unavailable_visuals --nocapture --test-threads=1 (2 passed, 0 failed, 1896 filtered out)
  - D:\cargo-targets\zircon-ui-style-selector-0607\debug\deps\zircon_editor-16c136b0ff3b6b9d.exe icon_button_loading_state_uses_unavailable_visuals --nocapture --test-threads=1 (1 passed, 0 failed, 1897 filtered out)
  - D:\cargo-targets\zircon-ui-style-selector-0607\debug\deps\zircon_editor-16c136b0ff3b6b9d.exe disabled_workbench_button_suppresses_declared_style_but_keeps_opacity --nocapture --test-threads=1 (1 passed, 0 failed, 1897 filtered out)
  - cargo test -p zircon_editor --lib segmented_and_tab_loading_state_uses_unavailable_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib text_field_loading_state_uses_unavailable_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-style-selector-0607 --message-format short --color never -- --nocapture --test-threads=1
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels.rs
  - cargo test -p zircon_editor --lib chrome_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-chrome-native-0611 --message-format short --color never -- --nocapture --test-threads=1 (compiled and ran 31 chrome-filtered editor tests; 30 passed, including the Workbench Chrome selector and shell-panel native-paint regressions; one unrelated reference-surface test failed before metric assertions at built-in UI template loading with `UiV2Asset(Io(path not found, os error 3))`)
  - D:\cargo-targets\zircon-ui-chrome-native-0611\debug\deps\zircon_editor-89dfdfd76ef4e2b4.exe workbench_chrome::tests --nocapture --test-threads=1 (3 passed, 0 failed, 1899 filtered out)
  - D:\cargo-targets\zircon-ui-chrome-native-0611\debug\deps\zircon_editor-89dfdfd76ef4e2b4.exe shell_panel_chrome_selector_states_reach_native_paint --nocapture --test-threads=1 (1 passed, 0 failed, 1901 filtered out)
doc_type: module-detail
---

# Workbench Style Selectors

The retained Workbench painter keeps visual state resolution in `style_selector/*` files and leaves the template painters responsible for recognition, geometry, and command emission. Selectors consume the shared `UiPainterState` / `UiPainterResolvedState` priority model, so hover, focus, press, selected, checked, open, disabled, dragging, drop-hover, and loading states do not drift across component families.

## Workbench Chrome

`workbench_chrome.rs` owns shell chrome colors for the retained host Workbench frame: window root, top toolbar, main band, activity rail, side panels, viewport frame, component drawer, status bar, tabs band, and inspector sections. The selector resolves from `UiPainterFamily::Chrome`, matching the runtime render-extract Chrome family so Workbench-level surfaces use the same disabled/loading and active-state priority as other painter families.

- normal shell chrome preserves the existing pixel palette used by `template_shell_panels.rs`;
- disabled and loading chrome mute fills and separators through unavailable surface and border colors before hover, focus, selection, drag, or drop-hover can apply;
- focused, selected, checked, open, drag, and drop-hover chrome use the active selected surface and focus separator, while hover stays in the hot surface lane;
- `template_shell_panels.rs` remains responsible for control-id recognition, pixel alignment, separator geometry, clip propagation, and paint-command ordering.

The selector-local regressions `chrome_selector_preserves_normal_shell_panel_colors`, `chrome_loading_state_uses_unavailable_visuals`, and `chrome_active_state_uses_shared_focus_and_hot_visuals` lock normal palette preservation, loading precedence, and active separator behavior. The shell-panel integration regression `shell_panel_chrome_selector_states_reach_native_paint` verifies loading and focused Chrome selector states reach native pixel output. Existing shell-panel pixel tests keep toolbar, side-panel, drawer, status, and drawer-column geometry anchored to the native painter. The native component contract now also checks `workbench_chrome.rs` and runtime `chrome.rs` so shell surfaces stay represented across the web prototype, `.zui` shell assets, retained host selector, and runtime render extract.

## Buttons And Icon Buttons

`workbench_button.rs` owns Workbench command-button colors while `workbench_icon_button.rs` owns toolbar, panel, and rail icon-button colors. The selector follows the Slate `SButton` paint precedence: `SButton::OnPaint` uses `ShouldBeEnabled(...)` before child paint and `SButton::UpdateBorderImage()` selects the disabled brush before pressed or hovered imagery. Zircon maps semantic loading to that same unavailable visual lane so busy buttons do not keep active action colors:

- command buttons resolve from `UiPainterFamily::Button`; disabled and loading now win before pressed, focus, hover, declared button colors, Add Component tones, and label brightness scaling;
- unavailable command buttons mute surface, border, text, and glyph through one disabled-color output while preserving existing opacity handling from `template_buttons.rs`;
- icon buttons resolve from `UiPainterFamily::IconButton`; disabled and loading now win before danger, selected, checked, focus, press, hover, declared background/border/icon colors, and selected icon colors;
- panel icon buttons keep disabled surface and border during loading, while toolbar icon buttons keep their frameless shape but mute the glyph;
- template button painters remain responsible for role recognition, pixel geometry, icon/text layout, opacity, and command ordering.

`resolved_state_for_node` also merges the resolved `TemplatePaneNodeData.button_style.interaction_state` into the shared `UiPainterState`. Resolved `Hover`, `Pressed`, and `Focused` button states therefore reach the same selector priority as node-local hover/press/focus flags, while resolved `Disabled` and `Loading` continue to enter the unavailable lane before active button visuals. This keeps runtime-projected MUI button state aligned with native painter priority instead of requiring each template button path to duplicate resolved-state checks.

The selector-local regressions `button_loading_state_uses_unavailable_visuals` and `icon_button_loading_state_uses_unavailable_visuals` lock loading precedence over active and declared visuals. The template regression `disabled_workbench_button_suppresses_declared_style_but_keeps_opacity` keeps disabled command buttons aligned with the same unavailable-color policy while preserving the existing disabled opacity path.

## Tabs And Segmented Controls

`workbench_segmented_control.rs` owns the color and state contract for `WorkbenchSegmentedControl` and `WorkbenchTab` primitives:

- segmented controls use the same `UiPainterFamily::Tab` interactive state priority as tabs, then resolve group fill, border, selected segment fill/border, underline, selected text, idle text, and group-label color;
- tabs keep declared idle background support from `.zui` styles only while available; disabled and loading resolve before hover/focus/pressed/checked/selected and suppress declared tab backgrounds, selected underline colors, active selected text, and group-label colors;
- unavailable segmented controls keep selected/checked structure visible but mute group fill, border, selected segment fill/border, underline, selected text, idle text, and group label through disabled colors;
- selected segment border width, underline height, and underline color remain declaration-driven so icon-toggle segmented controls can suppress the selected border while ordinary segmented controls keep the legacy one-pixel selected border;
- `template_segmented_controls.rs` now delegates state-dependent style choices to the selector while keeping layout offsets, segment splitting, tab underline geometry, text placement, and paint-command ordering local to the painter.

The focused regression `segmented_and_tab_styles_use_shared_state_priority` verifies disabled state wins over pressed/focused/hovered, pressed wins after disabled is removed, and checked tabs still draw selected text while hover controls the tab background. `segmented_and_tab_loading_state_uses_unavailable_visuals` locks the loading-state unavailable treatment for segmented controls and tabs. The browser/native component contract also checks for the selector file and the required pressed/disabled state handling so web-to-native component promotion cannot silently drop this family.

## Selection Controls

`workbench_selection_control.rs` owns Workbench checkbox, radio, and toggle visual resolution. The selector follows the Slate `SCheckBox` split where `OnGetCheckImage()` picks unchecked, checked, or undetermined brushes and then chooses pressed/hovered variants inside that checked lane. Zircon keeps checked/selected as semantic value state, but unavailable paint state is resolved before those active brush branches:

- checkbox, radio, and toggle controls resolve from `UiPainterFamily::Checkbox`, `Radio`, or `Toggle`; disabled and loading now win before pressed, focus, drag, drop-hover, hover, selected, or checked;
- disabled/loading controls mute the mark or track surface, border, toggle thumb, radio dot/accent, row text, and label through one selector output while preserving checked/selected geometry such as checkbox ticks and toggle thumb position;
- unavailable controls suppress declared `.zui` background, border, foreground, label, and value colors so loading cannot render as an active checked toggle or focused checked checkbox;
- pressed, focus, hover, drag, and drop-hover still produce active focus-ring or pressed treatment only when the resolved state is available;
- `template_selection_controls.rs` remains responsible for Workbench role recognition, mark geometry, radio dot size, toggle track/thumb placement, text lanes, and paint-command ordering.

The selector-local regression `selection_controls_loading_state_uses_unavailable_visuals` locks loading-state precedence over checked/selected, press/drop-hover, and declared colors. The template regression `selection_control_loading_state_mutes_active_checked_visuals` keeps the retained painter helper path aligned with the shared selector while preserving existing checked-control geometry tests.

## List Rows

`workbench_list_row.rs` owns the style contract for `WorkbenchListRow` collection rows:

- disabled and loading rows are unavailable: they suppress row background and border while returning disabled text/adornment colors before selected, checked, declared color, hover, or focus branches can apply;
- selected or checked rows keep selected surface and focus-ring adornment semantics independent of hover state only when the resolved state remains available;
- pressed, focused, dragging, and drop-hover states produce the shared focus-ring border through `UiPainterResolvedState` instead of duplicating state branches in `template_list_rows.rs`;
- declared row background, text, and icon colors still win where the template author provided them, preserving the component-drawer list samples.

`workbench_tree_row.rs` and `workbench_table_row.rs` use the same collection-row policy for SceneTree and asset-table rows. Tree rows mute selected text, disclosure, object icon, secondary text, and action colors during disabled/loading; table rows use disabled surface/text/action colors and suppress focus-ring borders during disabled/loading before selected row or declared background colors can apply.

`template_list_rows.rs` now keeps row recognition, label geometry, adornment geometry, and the check/chevron/disabled mark paint commands, including resolving the disabled-diamond mark from selector state so loading selected rows do not keep the active check shape. `template_tree_rows.rs` keeps tree indentation, disclosure, object-icon geometry, action glyphs, and command order, while routing the special PlayerStart icon color through the selector when the row is unavailable. The focused regressions `list_row_style_uses_shared_state_priority`, `list_row_loading_state_uses_unavailable_visuals`, `tree_row_loading_state_uses_unavailable_visuals`, `table_row_loading_state_uses_unavailable_visuals`, and `loading_player_start_tree_row_mutes_special_icon_color` cover disabled/loading precedence over active collection-row visuals, while `verify-native-component-contract.mjs` checks that the native ListRow selector exists and handles pressed/disabled states.

## Popup Rows

`workbench_popup_row.rs` owns Workbench popup-menu and dropdown-option row visuals. Popup rows now project the full shared `UiPainterState` contract from their local `WorkbenchPopupRowState`, including open, dragging, drop-hover, and loading in addition to hover, press, focus, disabled, checked, and selected:

- disabled and loading rows are treated as unavailable, so they suppress selection marks and use disabled text/adornment/shortcut colors;
- open, dragging, and drop-hover rows resolve through `UiPainterFamily::PopupRow` and use the same hot-row treatment as hover/focus/press;
- selected and checked rows still keep the selected surface and focus-ring selection mark unless the resolved state is unavailable;
- danger rows keep their authored danger text/adornment color only after unavailable-state suppression, so a loading or disabled destructive action cannot render as an active action.

`template_popup_rows.rs` remains responsible for popup geometry, menu item flag parsing, right-aligned adornments, shortcut text placement, and paint command ordering. Its option-row path now receives `loading`, selected, focused, hovered, pressed, disabled, and label/id metadata from `pane_option_projection.rs`, including real `DropdownPopup` `selected_options`/`selectedOptions`, `focused_index`, `hovered_option_id`, `loading_options`, and `id|label=...` declarations. Its menu-row path receives `loading` from `pane_menu_projection.rs` as well as the existing raw flag fallback, so `ContextActionMenu` and real context-menu rows reach the same unavailable-state selector lane as runtime render extract. Geometry for option rows is delegated to `template_popup_layout.rs`: ordinary dropdown triggers still open below/above the trigger with bounds handling, while standalone `DropdownPopup` rows are cut directly inside the projected popup frame. `surface_hit_test/template_node.rs`, `native_keyboard.rs`, and `native_popup_dismiss.rs` use the same helper, so painter output, pointer rows, keyboard focus frames, and dismiss containment do not drift. The selector-local regression `popup_row_selector_projects_full_semantic_state` covers open/drag/drop-hover state projection, `popup_row_loading_state_uses_unavailable_visuals` locks the loading-state unavailable treatment, `popup_row_style_selector_matches_runtime_extract_state_matrix_for_projected_rows` pins native projected-row selected/focused/disabled/loading resolved states to the same state names asserted by the runtime render-extract popup tests, and `standalone_dropdown_popup_paints_rows_inside_projected_popup_frame` verifies the native painter uses the projected `DropdownPopup` frame. The host projection regressions `runtime_component_projection_projects_popup_option_state_metadata_for_native_painter`, `runtime_component_projection_projects_popup_menu_loading_flags_for_native_painter`, and the popup anchor assertions in `runtime_component_projection_positions_mui_popups_from_anchor_metadata` cover the editor-native row DTO and frame-projection input paths.

## Dropdowns

`workbench_dropdown.rs` owns Workbench dropdown and combo-button trigger visuals. The selector follows the Unreal Slate combo-button shape: `SComboBox` inherits `SComboButton`, `SComboButton` builds the trigger from `SButton`, and `SButton` resolves disabled paint before active button imagery. Zircon keeps that as one retained-host rule instead of letting the dropdown trigger and popup rows disagree:

- dropdown triggers resolve from `UiPainterFamily::Dropdown`, so disabled/loading wins before pressed, focus, open, hover, selected, or checked state can apply;
- disabled and loading triggers mute the whole trigger: surface, border, label text, placeholder text, and chevron all use unavailable colors;
- unavailable triggers also suppress declared `.zui` background, border, value, icon colors, and brightness scaling, preventing a loading dropdown from rendering as a selected/open active control;
- open, pressed, focused, hover, drag, and drop-hover states remain active visual states only when the resolved state is available;
- `template_dropdowns.rs` remains responsible for Workbench dropdown recognition, pixel-aligned geometry, fallback label selection, chevron segments, popup-row handoff, and paint-command ordering.

The selector-local regression `dropdown_loading_state_uses_unavailable_visuals` locks loading-state precedence over open/hover/selected validation and declared colors. The existing template dropdown regressions keep trigger geometry, popup-row emission, declared-color support for available controls, and shared state priority anchored to the native painter.

## Sliders

`workbench_slider.rs` owns Workbench range-field slider visuals. The selector follows the Unreal Slate slider path: `SSlider::OnPaint` derives a disabled draw effect from `ShouldBeEnabled`, while `GetBarImage()` and `GetThumbImage()` select disabled bar/thumb brushes before hover brushes. Zircon maps semantic loading onto the same unavailable lane so busy sliders cannot keep active drag or warning/accent visuals:

- sliders resolve from `UiPainterFamily::Slider`; disabled and loading now win before pressed, focus, drag, drop-hover, and hover;
- disabled/loading sliders mute the track, fill, thumb, thumb outline, value boxes, label text, value text, range-value border, and tick marks through one selector output;
- unavailable sliders suppress declared `.zui` background, border, label, value, icon, and state-layer colors so loading cannot render as a hovered or dragged active range field;
- hover, press, focus, drag, and drop-hover retain halo and active value-border semantics only when the resolved state is available;
- `template_sliders.rs` remains responsible for Workbench slider recognition, label/value geometry, range span math, ticks, thumb placement, and paint-command ordering.

The selector-local regression `slider_loading_state_uses_unavailable_visuals` locks loading-state precedence over press/drop-hover, validation colors, declared colors, and declared state-layer halos. The existing template slider regressions keep track/fill/thumb/value rendering, range minimum, tick count, declared metrics, and available-state declared colors anchored to the native painter.

## Status Controls

`workbench_status_control.rs` owns Workbench status-bar visual resolution for status signals, chips, and icon buttons. The selector follows the Slate disabled-paint precedent from `SButton::OnPaint`: unavailable state is resolved before active visual branches, so disabled/loading output cannot keep an active success, warning, info, or declared override color.

- status signals resolve from `UiPainterFamily::Generic`, then choose icon fill, label text, and inner mark colors from one resolved state;
- disabled and loading status signals mute the entire signal: icon fill, text, and mark all use the disabled text color before declared `.zui` label/value/icon colors or success/warning/info defaults can apply;
- status chips keep their pill surface, border, chevron, and text tied to the same Generic state priority, including unavailable surface/border/text output;
- status icon buttons resolve through `UiPainterFamily::IconButton`, keeping checked/selected/focused/pressed/open/drag/drop-hover glyph focus-ring semantics while disabled/loading mute glyph, border, and surface;
- `template_status_controls.rs` remains responsible for Workbench status-control id recognition, icon geometry, mark stroke width, text gaps, chip chevrons, and paint-command ordering.

The selector-local regression `status_signal_unavailable_states_mute_icon_text_and_mark` locks the whole-signal disabled/loading suppression. The existing template regressions keep geometry, declared active colors, chip state priority, and icon-button state priority anchored to the native status-bar painter.

## Text Fields

`workbench_text_field.rs` owns Workbench text-field visual resolution:

- field surface, border, text, placeholder, and stepper glyph colors resolve from `UiPainterFamily::TextField` through the shared interactive state priority;
- disabled and loading state win over pressed/focused/hovered and mute field surface, border, text, placeholder, and stepper glyph colors;
- unavailable text fields suppress `.zui` declared background/border colors and validation borders so a loading field cannot render as focused or error-active, while available fields still honor declared background and border colors;
- `template_fields.rs` now keeps Workbench text-input recognition, field frame offsets, half-pixel height preservation, placeholder fallback label, text geometry, and stepper drawing.

The focused regression `workbench_field_selector_uses_shared_text_field_state_priority` covers disabled-over-pressed/focused/hovered priority, pressed before focused, the focused visual sample path, and placeholder color selection. `text_field_loading_state_uses_unavailable_visuals` locks loading precedence over focus, press, validation, and declared colors. The browser/native component contract now checks that the native TextField selector exists and handles pressed/disabled states.

## Feedback Controls

`workbench_alert.rs` owns Workbench inline alert visual resolution:

- alert surface, border, mark, and text colors resolve from `UiPainterFamily::Alert` through the shared interactive state priority;
- disabled and loading alerts use unavailable surface, border, mark, and text colors before tone, focus, pressed, hover, icon, label, or declared style colors can apply;
- `template_alerts.rs` still owns inline alert tone detection, glyph geometry, title/message/action placement, and paint-command ordering.

`workbench_tooltip.rs` owns Workbench tooltip visual resolution:

- tooltip surface, border, title, body, arrow, icon, and shadow colors resolve from `UiPainterFamily::Tooltip` through the shared interactive state priority;
- disabled and loading tooltip state mutes all text/icon/arrow output and lowers shadow strength, while pressed/focused state routes border and icon color through the focus-ring semantics used by the other retained controls only when the resolved state remains available;
- author-declared `.zui` style colors still override surface, border, title, body, icon, and arrow colors for available states, but unavailable state suppresses those declarations before paint commands are emitted;
- `template_tooltips.rs` now keeps tooltip detection, bubble placement, arrow geometry, text layout, info-icon drawing, and paint-command ordering.

`workbench_toast.rs` owns Workbench toast visual resolution:

- toast surface, border, text, status mark, action text, and close mark resolve from `UiPainterFamily::Toast`, so disabled, pressed, focused, hovered, dragging, drop-hover, open, and loading state use the same priority model as the rest of the retained-host painter;
- pressed and focused toasts use focus-ring border/action styling, hovered and drag/drop states use a hotter toast surface, and disabled/loading state suppresses declared surface, border, text, action, and mark colors in favor of disabled surface/border/text colors;
- `template_alerts.rs` still owns inline alert tone detection and glyph geometry, but standalone Workbench toast visuals now come from the selector instead of local Toast constants and helper branches.

The focused regressions `alert_loading_state_uses_unavailable_visuals`, `tooltip_loading_state_uses_unavailable_visuals`, and `toast_loading_state_uses_unavailable_visuals` lock loading-state precedence over pressed/focused/hovered state and declared colors for feedback controls. The older `workbench_tooltip_style_uses_shared_state_priority` and `workbench_toast_style_uses_shared_state_priority` regressions cover disabled-over-pressed/focused/hovered precedence plus the pressed/focused fallthrough used by the shared selector model. `verify-native-component-contract.mjs` keeps feedback runtime extract coverage aligned with the renderer split by checking component recognition and suppression in `feedback.rs` while checking Alert, Tooltip, and Toast family ownership in runtime `feedback/state.rs`.
