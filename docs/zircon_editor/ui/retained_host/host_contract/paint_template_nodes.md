---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/fallback.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/ordering.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/specialized.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/style/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/exports.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style/surface_roles.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style/colors/border.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style/colors/surface/interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style/colors/surface/variants.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style/dimensions.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_tests/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button/tab_like.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/colors.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/colors/border.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/colors/background.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields/search.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control/chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control/icons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control/signals.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome/separators.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_asset_placeholder_visuals.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_kind/kind.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_kind/mapping.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/chrome/navigation.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/dispatch/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/dispatch/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/files/document.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/files/folder.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/files/save.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/candidates/aliases.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets_tests/runtime.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons_tests/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/chips/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/icons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/signals.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/signals/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls_tests/signals.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/signals/constants.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/signals/icon.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/signals/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_glyphs/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_glyphs/segments.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_glyphs/signals.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_status_bar.zui
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_list_row/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_row_glyphs/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/adornment.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/support.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/fallback.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/ordering.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/specialized.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/style/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/exports.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style/surface_roles.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style/colors/border.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style/colors/surface/interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style/colors/surface/variants.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style/dimensions.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_tests/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button/tab_like.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/colors.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/colors/border.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/colors/background.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields/search.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control/chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control/icons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control/signals.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome/separators.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_asset_placeholder_visuals.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_kind/kind.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_kind/mapping.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/chrome/navigation.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/dispatch/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/dispatch/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/files/document.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/files/folder.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/files/save.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/candidates/aliases.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets_tests/runtime.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons_tests/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_list_row/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_row_glyphs/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/adornment.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/chips/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/icons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/signals.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/signals/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls_tests/signals.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/signals/constants.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/signals/icon.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/signals/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_glyphs/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_glyphs/segments.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_glyphs/signals.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_status_bar.zui
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows/surface/background.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows/surface/row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows/surface/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows_tests/paint.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - cargo test -p zircon_editor --lib template_table_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib componentized_workbench_status_bar --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib template_status_controls --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib componentized_workbench_window_template_bridge_exports_surface_projection_frames_and_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib template_style --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib asset_preview_selected_surface_uses_slate_outline_emphasis --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib template_style --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib asset_placeholder_visual --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib asset_placeholder_visual_uses_single_recessed_well_and_svg_icon --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27: passed, 1 passed)
  - cargo test -p zircon_editor --lib template_asset_placeholder_visuals --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27: passed, 4 passed)
  - cargo test -p zircon_editor --lib asset_placeholder_visual --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0627-thumb-icons --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27: passed, 5 passed)
  - cargo test -p zircon_editor --lib semantic_shell_icon_aliases_load_as_real_pixels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0627-thumb-icons --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27: passed, 1 passed)
  - cargo test -p zircon_editor --lib asset_placeholder_visual --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0627-thumb-plate --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27: passed, 6 passed)
  - cargo test -p zircon_editor --lib semantic_shell_icon_aliases_load_as_real_pixels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0627-thumb-plate --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27: passed, 1 passed)
  - cargo test -p zircon_editor --lib template_fields --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never
  - cargo test -p zircon_editor --lib template_fields --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 15 passed after Quick Import placeholder slice)
  - cargo test -p zircon_editor --lib inactive_workbench_module_tab_keeps_toolbar_surface_clear --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib template_buttons --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib workbench_toolbar --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib template_icon_buttons --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib template_icon_button --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib visual_assets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib close_outline_maps_to_close_glyph --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib table_header_and_tail_use_recessed_table_surface --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: red then passed)
  - cargo test -p zircon_editor --lib workbench_table_row --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 8 passed)
  - cargo test -p zircon_editor --lib template_table_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 17 passed)
  - cargo test -p zircon_editor --lib workbench_table_row_action_stays_hidden_until_marked_or_hot --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27: passed, 1 passed)
  - cargo test -p zircon_editor --lib template_table_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27: passed, 19 passed)
  - cargo test -p zircon_editor --lib workbench_popup_row --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel --message-format short --color never --no-run (2026-06-27: passed)
  - direct D:\cargo-targets\zircon-editor-components-0626-panel\debug\deps\zircon_editor-b22e0a71937e69f5.exe workbench_popup_row --test-threads=1 --nocapture (2026-06-27: 2 passed)
  - direct D:\cargo-targets\zircon-editor-components-0626-panel\debug\deps\zircon_editor-b22e0a71937e69f5.exe template_popup_rows --test-threads=1 --nocapture (2026-06-27: 6 passed)
  - cargo test -p zircon_editor --lib template_list_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27: passed, 9 passed)
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --ignored --test-threads=1 --nocapture
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --ignored --test-threads=1 --nocapture
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-06-27: passed)
  - direct D:\cargo-targets\zircon-editor-components-0626-panel\debug\deps\zircon_editor-b22e0a71937e69f5.exe capture_workbench_component_slate_atlas_visual_artifact --ignored --test-threads=1 --nocapture (2026-06-27: passed)
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel --message-format short --color never (2026-06-27: passed)
  - paint-template-nodes root re-export ownership scan
  - scoped trailing-whitespace scan
  - scoped git diff --check
  - rustfmt --edition 2021 --check zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_layout.rs zircon_editor/src/ui/layouts/views/asset_browser/tests.rs zircon_editor/src/ui/asset_editor/node_projection.rs zircon_editor/src/ui/layouts/views/view_projection.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/style/text.rs (2026-06-28 logical text-align support gate: passed)
  - cargo test -q -p zircon_editor --lib aligned_text_x_resolves_logical_start_end_against_text_direction --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0628-thumb-badge-muted -- --test-threads=1 --nocapture (2026-06-28: passed, 1/1)
doc_type: module-detail
---

# Paint Template Nodes

`paint_template_nodes/mod.rs` is the retained-host template-node paint entry. It should stay as a structural module declaration surface plus stable re-exports for Workbench painters and test helpers.

`template_node_pipeline.rs` owns the public draw pipeline entry. Its `draw.rs` child iterates node models, applies clipping, orders runtime commands, and reports whether any visible node was painted. Its `clip.rs` child owns node clip resolution, and `test_support.rs` owns image-buffer helpers used by the template paint regression suite.

`template_nodes.rs` owns command emission for a single template node. The `template_nodes/` child owners keep command production separated into command orchestration, fallback rendering, frame geometry, ordering, and specialized component dispatch. Rendering DTO conversion remains in `render_command_conversion.rs`, while `render_commands.rs` owns the runtime command paint harness used by tests.

The 2026-06-28 logical text-align support gate keeps runtime render-command text positioning inside `render_command_conversion/style/text.rs`. Projection code may preserve `UiTextAlign::Start` and `UiTextAlign::End` as semantic strings for metadata, but the paint conversion leaf resolves those logical values against `UiTextDirection` before computing x placement. `Auto`, `LeftToRight`, and `Mixed` use the LTR fallback; `RightToLeft` flips Start and End. The focused regression locks this in the retained paint owner so future runtime-interface enum additions do not reopen non-exhaustive matches or push alignment policy into Asset Browser or view assembly roots.

`style_selector/mod.rs` is now the structural Workbench style-selector entry. `style_selector/exports.rs` owns the selector re-export surface for the child style modules while each `workbench_*` child keeps the family-specific style resolution.

The 2026-06-27 popup/dropdown row styling pass keeps menu row color policy in `style_selector/workbench_popup_row/selection.rs` and popup shell geometry in `template_popup_rows/surface/*`. Selected and checked rows now use a low-emphasis pressed surface with normal text/adornment colors, while the accent color is limited to the left selection marker. The popup shell uses the shared 1 px border metric and a square Slate-like outline; row marker width comes from `METRICS.selection_indicator_width` instead of a local magic value.

The 2026-06-27 selected-only list row follow-up keeps row identity and checked state separate. `template_list_row_glyphs/selection.rs` maps only `checked` rows to the trailing `Check` adornment, while selected-only rows keep the navigation chevron. `style_selector/workbench_list_row/selection.rs` still treats selected and checked rows as marked for row surface/text, but only checked rows mark the trailing adornment color. This keeps selected list rows on a low-emphasis surface with a left 2 px indicator instead of adding a bright right-side checked marker.

## Boundary Rules

- Keep `paint_template_nodes/mod.rs` limited to child declarations and re-exports.
- Keep draw iteration, clipping, and test image buffers in `template_node_pipeline/`; do not reintroduce wrapper functions in the root module.
- Keep per-node command construction in `template_nodes/` and low-level replay/test harness behavior in `render_commands.rs`.
- Keep Workbench pane, menu, dock, and overlay painters as consumers of `draw_template_nodes`/`has_template_nodes`; they should not reach into template-node internals.

## Validation Notes

The 2026-06-21 root re-export split reduced `paint_template_nodes/mod.rs` from 118 lines to an 85-line structural entry. The stable entries now re-export `draw_template_nodes`, `has_template_nodes`, `paint_template_nodes_for_test`, and `paint_template_nodes_for_test_with_background` directly from `template_node_pipeline`, while `paint_runtime_render_commands_for_test` remains re-exported from `render_commands`.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint-template-nodes root re-export ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 style-selector export split reduced `paint_template_nodes/style_selector/mod.rs` from 101 lines to a 20-line structural declaration/re-export entry. `style_selector/exports.rs` owns the restricted selector export surface for state projection and Workbench family selectors. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a style-selector export ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-26 tab-like button pass keeps tab classification in `style_selector/workbench_button/tab_like.rs`, style selection in `workbench_button/selection.rs`, and the selected-state underline paint in `template_buttons/surface.rs`. `template_buttons/identity.rs` only routes recognized tab-like controls into the Workbench button path, including PageTab, DockTab, Asset Browser kind/view/tool controls authored without explicit variants, and the top Workbench module tabs. Module command buttons remain outside that tab-like whitelist so command styling can be tuned separately. This prevents cyan focus-ring rectangles from becoming a fallback behavior while keeping the root `paint_template_nodes` entries structural.

The 2026-06-26 Workbench module-tab clear-toolbar follow-up keeps top module tab styling inside the same Workbench button owners. `style_selector/workbench_button/tab_like.rs` exposes module-tab classification to sibling style code, and `workbench_button/selection.rs` gives inactive module tabs a transparent surface plus muted text while preserving low-emphasis hover/open/active feedback. Asset Browser tab-like controls keep their own low-contrast surface behavior, so this removes top-toolbar blocky tiles without flattening content tabs.

The 2026-06-26 Asset Browser filter/view tab follow-up keeps Content Browser-like filter and view toggles in the Workbench button style owner without treating every inactive item as a filled tile. `style_selector/workbench_button/tab_like.rs` exposes Asset Browser kind/view/utility-tab classification separately from PageTab, DockTab, and Workbench module tabs. `workbench_button/selection.rs` now gives inactive Asset Browser tab-like controls a transparent surface plus muted text, while hover, popup-open, focused, selected, and checked states still use the low-emphasis hover surface and the shared 2 px underline indicator from `template_buttons/surface.rs`. The regression coverage in `template_buttons_tests/style.rs` and `template_buttons_tests/paint.rs` locks transparent inactive tabs, hover feedback, and selected underline behavior.

The 2026-06-26 toolbar icon glyph follow-up maps the Workbench top toolbar to the UE SlimToolbar `Icon20x20` sizing rule. `template_icon_buttons/geometry.rs` now gives Toolbar icon buttons a 20 px glyph cap/default for 30 px buttons while leaving Panel and Rail contexts on their existing density paths. The file fallback glyph owners for document, folder, and save icons draw recognizable document-plus, folder, and floppy outlines, so missing SVG assets no longer collapse into tiny placeholder line fragments.

The 2026-06-26 toolbar icon asset follow-up moves that fidelity step into the icon-button glyph owner without turning the root painter into an asset policy module. `template_icon_button_glyphs.rs` first asks `visual_assets/asset.rs` for an existing SVG/icon raster at the arranged glyph size, applies the current state foreground tint, and emits one image-pixel command when the asset exists. Missing icon paths deliberately return `None` from that existing-asset helper instead of generating the deterministic placeholder image, so the same glyph owner can fall back to the manual line-glyph vocabulary. This keeps UE-like toolbar icons asset-driven while preserving deterministic pixels for deferred or intentionally unmapped icon names.

The 2026-06-26 document-tab close follow-up keeps the dock-tab close affordance in icon-button owners rather than template-specific paint code. `template_icon_buttons/style.rs` treats `DockTabClose`, `PageTabClose`, `DocumentTabClose`, and sibling tab-close ids as Toolbar context buttons, so normal state does not draw a panel surface. `template_icon_button_glyph_kind/mapping.rs` maps close/dismiss names to the `Close` glyph kind, and the chrome glyph dispatch draws the X-shaped close icon. In the same slice, `template_buttons/content/metrics.rs` measures labels with the node-declared font size before `template_buttons/content/text.rs` emits the text command; this keeps 12 px document-tab labels readable instead of under-measuring them with the 10 px body default.

The 2026-06-26 command-button follow-up adds that separate command styling owner in `style_selector/workbench_button/command.rs`. It recognizes Compile and asset import command controls/actions and lets `workbench_button/selection.rs` replace authored accent-filled surfaces with muted Workbench surfaces plus accent foreground. Pixel regressions in `template_buttons` lock this as button style behavior, not root-module or screenshot-only logic.

The 2026-06-27 generic editor primary-button follow-up keeps the same button owner boundary but clamps legacy Material class colors before they can override Workbench tokens. `template_buttons/identity.rs` still routes generic editor variants such as `button_variant=primary` into the Workbench button path, but `style_selector/workbench_button/selection.rs` now applies declared background, border, and foreground colors only when `uses_workbench_visual_language(node)` is true. As a result, authored Workbench controls can still tune their local surface and border, while ordinary editor actions such as `OpenAssetsView` use the canonical low-emphasis primary surface instead of inheriting the old purple `.primary` rule from projection. The `editor_variant_button_ignores_legacy_declared_material_colors` regression captures that boundary.

The 2026-06-26 table selected-row follow-up keeps Workbench table row selection split between style and paint owners. `style_selector/workbench_table_row` owns the selected fill decision, rejects authored selected backgrounds for marked rows, and now suppresses the focus-ring border for rows that are selected/checked while still letting unmarked focused rows show the keyboard focus border. `template_table_rows/surface.rs` paints only the 2 px accent selection indicator. The row still uses the normal table text/action command path, so the selected state is visible without turning the full row into a cyan block or cyan outline.

The 2026-06-26 table recessed-surface follow-up keeps Workbench table list chrome in the same table-row owner. `style_selector/workbench_table_row/palette.rs` now maps header and tail/empty-fill backgrounds to the same recessed row surface used by ordinary rows, matching the Unreal Slate `TableView` pattern and removing extra black bands around Asset Browser tables. `style_selector/workbench_table_row/tests.rs` locks the behavior before wider table composition changes.

The 2026-06-27 table row action follow-up keeps inline row affordance visibility local to the table-row action leaf owner. `style_selector/workbench_table_row/state.rs` remains the single source for hot row states, and `template_table_rows/actions/entry.rs` now paints data-row `more-horizontal.svg` only for selected, checked, pressed, or hot rows. Table headers still paint their settings gear unconditionally. The regression in `template_table_rows_tests/paint.rs` locks neutral rows to zero action image-pixel commands while hovered and selected rows keep the shared shell SVG affordance.

The 2026-06-26 status-bar flat-control follow-up keeps status chip/icon behavior inside the status-control owners. `style_selector/workbench_status_control` makes normal chip/icon surfaces transparent while retaining interaction feedback, and `template_status_controls` skips transparent quads instead of painting button-like blocks. Status chips no longer draw the down-chevron; text spacing now uses `METRICS.gap_s`, while `template_status_controls/chips/text.rs` continues to own label/value splitting and right alignment.

The 2026-06-26 status-signal marker follow-up keeps bottom-left status signals as one inline `METRICS.gap_m` round marker plus one text run. Signal geometry ignores legacy per-asset `icon_size`, `layout_icon_size`, vertical icon offsets, and stroke/mark widths; `template_status_glyphs/signals.rs` now paints the same circle for Ready, Success, Warning, and Info. The old success/check, warning-triangle, and info-circle signal glyph modules were removed so the status bar cannot drift back to large composite icons through a compatibility path.

The 2026-06-26 status-bar spacing follow-up keeps horizontal density authored in `workbench_status_bar.zui` without adding another paint owner. The status bar root gap is zero, left signal and right chip/icon slots have compact fixed widths, and the stretch filler is a transparent `Space` with no control id. Bridge tests assert the filler and idle task slot do not project host contract nodes, so the painter cannot accidentally render the middle stretch as an empty bordered control.

The 2026-06-26 Asset Browser placeholder follow-up keeps empty preview and secondary details surfaces low-emphasis. `template_style/colors/surface/variants.rs` now maps `asset-placeholder` and `asset-placeholder-visual` to the inset surface family without a border, and `template_style_tests/surface.rs` locks that behavior. This lets the selected table row and selected asset preview remain visible without painting cyan selection boxes around the right-side details placeholders.

The 2026-06-26 Asset Browser selected-preview follow-up keeps selected preview cards in the same low-emphasis selection language. `template_style/surface_roles.rs` centralizes the `asset-preview` and `asset-preview-visual` classification, `template_style/colors/surface/interaction.rs` resolves their selected/focused fill to `PALETTE.surface_pressed` instead of `PALETTE.surface_selected`, `template_style/dimensions.rs` preserves the authored 1 px border, and `template_style/colors/border.rs` keeps interactive preview-card borders on the normal muted border rather than the focus ring. `template_style_tests/surface.rs` locks the dark fill, muted outline, focus override, and validation override so the card no longer becomes a full cyan block or outline.

The 2026-06-26 Asset Browser thumbnail-placeholder follow-up introduced a dedicated image placeholder owner. The 2026-06-27 refinement keeps that owner but removes the noisy hand-drawn dot, ridge, shadow, and baseline vocabulary. `template_asset_placeholder_visuals.rs` now recognizes `asset-placeholder-visual` and `asset-preview-visual`, paints one recessed thumbnail well with no inner border, and centers the real `image` SVG asset through the existing `visual_assets` pipeline. The fallback pipeline still invokes this owner after the base surface and before optional image commands, so real image resources can replace the placeholder while plain `asset-placeholder` cards remain simple inset containers. Pixel and command regressions in the same module lock the single-well contract, real SVG pixels, selected preview visuals, and plain placeholders separately.

The later 2026-06-27 asset-type icon refinement keeps that same placeholder owner boundary. `template_asset_placeholder_visuals.rs` treats nodes with `component_role=asset-thumbnail-visual` as typed thumbnail wells and uses the node `component_variant` as the icon name before falling back to `icon_name` or `image`. The semantic names are resolved in `visual_assets/candidates/aliases.rs`, so `asset-texture`, `asset-material`, `asset-scene`, `asset-mesh`, `asset-shader`, and sibling variants load real SVG pixels through the existing visual-asset path. This keeps asset-type selection out of the root painter while allowing Asset Browser thumbnail and preview nodes to share one typed visual contract.

The follow-up typed-thumbnail plate refinement keeps the same split. `template_asset_placeholder_visuals.rs` now paints typed thumbnail visuals as one recessed well, one centered low-emphasis icon plate, and one tinted 28 px SVG icon; plain generic `image` placeholders keep the smaller 20 px icon and do not receive a plate. Type tinting stays local to the placeholder visual owner, while `visual_assets/candidates/aliases.rs` adds `asset-ui-layout`, `asset-ui-widget`, and `asset-ui-style` aliases so UI resources do not collapse to script/file symbols. The command regressions lock the extra plate command, icon size, generic fallback size, and real-pixel alias resolution without adding a root painter branch.

The follow-up thumbnail type-badge pass keeps tile metadata styling in the shared template-style owner instead of drawing a custom Asset Browser badge. `thumbnail_nodes.rs` emits an `asset-type-badge` panel plus a separate type label and muted status label, while `thumbnail_layout.rs` sizes the badge from the type text and information-band width. `template_style/colors/surface/variants.rs` maps `asset-type-badge` to the low-emphasis hover layer and `template_style_tests/surface.rs` locks that it draws without a border. This gives thumbnail tiles a Content Browser-like type/status row while keeping the root painter free of asset-browser-specific branches.

The 2026-06-26 text/search-field follow-up keeps input-field visual density in the text-field owner rather than authored per-instance pixels. `style_selector/workbench_text_field/palette.rs` now resolves normal fields to the central Slate-like panel surface, hover/focus to the hover surface, borders to the stronger separator role, and placeholders to muted text instead of disabled text. `template_fields/search.rs` owns search-specific compact paint geometry using `METRICS.row_height` plus border-derived padding, so over-tall authored search fields center a stable 28 px control while normal fields keep their authored height. `template_fields/geometry.rs` only delegates the search-specific clamp after pixel alignment and layout offsets. Focused regressions in `template_fields_tests/style.rs` lock the palette role, placeholder role, and compact search geometry before the Asset Browser search and Quick Import fields are re-composed into the wider Workbench window.

The Quick Import placeholder follow-up keeps the empty path field in that same field-text path instead of drawing ad-hoc overlay text from the Asset Browser layout. `template_fields/text.rs` treats `AssetBrowserImportPathField` with empty `value_text` and non-empty projected label as a placeholder label, so the shared Workbench text-field style paints it with `PALETTE.text_muted`. The regression in `template_fields_tests/style.rs` also asserts that a non-empty path value returns to the normal field text color.

The 2026-06-27 content-panel surface follow-up keeps content container semantics inside the template-style owners. `template_style/surface_roles.rs` classifies `content-panel` and `asset-content`, `template_style/colors/surface/variants.rs` maps them to the recessed content layer, `template_style/colors/surface/interaction.rs` prevents hover/focus/selected state from repainting them as command or input controls, and `template_style/dimensions.rs` plus `template_style/colors/border.rs` preserve a 1 px muted outline. The regression in `template_style_tests/surface.rs` locks that selected/focused content panels remain low-emphasis containers instead of focus-ring boxes.
