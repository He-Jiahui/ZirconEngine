---
related_code:
  - zircon_editor/assets/ui/editor/material_meta_components.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
  - docs/ui-and-layout/material-ui-component-design-matrix.md
  - zircon_editor/assets/ui/editor/welcome.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/inspector_surface_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_base.ui.toml
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs
  - zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs
  - zircon_editor/src/tests/ui/boundary/material_ui_component_design_matrix.rs
  - zircon_runtime/src/ui/template/asset/compiler/value_normalizer.rs
  - zircon_runtime/src/ui/layout/pass/material.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/tests/asset_component_reference_layout.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_runtime/src/ui/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/shaders/ui_material.wgsl
  - zircon_runtime/src/ui/component/catalog/material_foundation
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/inputs.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/selection_inputs.rs
  - zircon_runtime/src/ui/tests/component_catalog/selection_state.rs
  - zircon_editor/assets/ui/editor/material_component_lab.v2.ui.toml
  - zircon_editor/assets/ui/editor/material_components/material_*.zui
  - zircon_editor/src/tests/ui/boundary/material_component_lab/catalog.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/feedback.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inputs/mod.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inputs/checkbox.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inputs/radio.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inventory.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/projection.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/shell.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/support.rs
  - zircon_editor/src/ui/template_runtime/builtin/material_lab_template_bindings.rs
  - tools/ui-profile-capture.ps1
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
implementation_files:
  - zircon_editor/assets/ui/editor/material_meta_components.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
  - docs/ui-and-layout/material-ui-component-design-matrix.md
  - zircon_editor/assets/ui/editor/welcome.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/inspector_surface_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs
  - zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs
  - zircon_editor/src/tests/ui/boundary/material_ui_component_design_matrix.rs
  - zircon_runtime/src/ui/template/asset/compiler/value_normalizer.rs
  - zircon_runtime/src/ui/tests/asset_component_reference_layout.rs
  - zircon_runtime/src/ui/layout/pass/material.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_runtime/src/ui/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/shaders/ui_material.wgsl
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/inputs.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/selection_inputs.rs
  - zircon_runtime/src/ui/tests/component_catalog/selection_state.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/catalog.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/feedback.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inputs/mod.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inputs/checkbox.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inputs/radio.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inventory.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/projection.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/shell.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/support.rs
  - tools/ui-profile-capture.ps1
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter.rs
  - zircon_runtime/src/ui/tests/material_button_style.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/inputs.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/selection_inputs.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/text_inputs.rs
  - zircon_editor/src/ui/template_runtime/builtin/material_lab_template_bindings.rs
  - zircon_editor/assets/ui/editor/material_components/material_button_group.zui
  - zircon_editor/assets/ui/editor/material_components/material_floating_action_button.zui
  - zircon_editor/assets/ui/editor/material_components/material_text_fields.zui
  - zircon_editor/assets/ui/editor/material_components/material_textarea_autosize.zui
  - zircon_editor/assets/ui/editor/material_components/material_selects.zui
  - zircon_editor/assets/ui/editor/material_components/material_autocomplete.zui
  - zircon_editor/assets/ui/editor/material_components/material_checkboxes.zui
  - zircon_editor/assets/ui/editor/material_components/material_radio_buttons.zui
  - zircon_editor/assets/ui/editor/material_components/material_mui_x_date_time_pickers.zui
plan_sources:
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - .codex/plans/Material UI 元组件与 .ui.toml 编辑器布局 Slate 化计划.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - docs/superpowers/plans/2026-05-06-material-layout-foundation.md
  - .codex/plans/Material UI 共享组件风格收束计划.md
  - dev/slint/ui-libraries/material/src/material.slint
  - dev/slint/ui-libraries/material/src/ui/styling/material_style_metrics.slint
  - dev/slint/ui-libraries/material/src/ui/styling/material_palette.slint
  - dev/slint/ui-libraries/material/src/ui/components/buttons/base_button.slint
  - .codex/plans/Editor 基础组件 Material 化视觉优化计划.md
  - docs/superpowers/plans/2026-05-17-mui-all-components-detailed-design.md
  - user: 2026-05-15 optimize retained editor UI styling with Material-like rounded controls and stronger feedback
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/compiler/value_normalizer.rs zircon_runtime/src/ui/tests/asset_component_reference_layout.rs zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs
  - cargo test -p zircon_runtime --lib ui_document_compiler_resolves_nested_material_role_tokens_in_props_and_styles --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m2-material-token-roles --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib material_theme_declares_m2_role_tokens_and_styles_material_classes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m2-material-token-roles --message-format short --color never -- --nocapture
  - zircon_runtime/src/ui/tests/material_layout.rs
  - rustfmt --edition 2021 zircon_runtime_interface/src/ui/surface/render/command.rs zircon_runtime_interface/src/tests/render_contracts.rs
  - cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m2-interface --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib asset_value_nodes_render_as_image_or_icon_not_text --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib runtime_svg_icon_pixels_follow_requested_target_size --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib template_svg_icon_pixels_follow_requested_target_size --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib template_icon_tint_uses_material_state_priority --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib material_meta_component_contracts --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs zircon_runtime/src/ui/tests/asset_component_reference_layout.rs
  - cargo test -p zircon_editor --lib material_meta_component_roots_forward_interaction_accessibility_and_capability_params --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib material_meta_components --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib ui_document_compiler_preserves_reference_instance_bindings_on_expanded_root --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib component_showcase_state --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib component_showcase_selection --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib component_showcase_category --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_runtime --lib material_layout --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never
  - cargo test -p zircon_runtime --lib material_layout --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_app --bin zircon_editor editor_gui_startup_parser --locked --jobs 1 --target-dir E:\cargo-targets\zircon-material-lab --message-format short --color never
  - powershell -NoProfile -Command "$null = [scriptblock]::Create((Get-Content -LiteralPath 'tools/ui-profile-capture.ps1' -Raw))"
  - cargo test -p zircon_editor --lib native_host_welcome_material --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib native_host_pointer_click_routes_projected_material_showcase_button --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib material_meta_component_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-material-shared-style-pass --message-format short --color never
  - cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1 --target-dir E:\cargo-targets\zircon-material-shared-style-pass --message-format short --color never
  - cargo test -p zircon_editor --lib runtime_component_projection_preserves_material_visual_metadata --locked --jobs 1 --target-dir E:\cargo-targets\zircon-material-shared-style-pass --message-format short --color never
  - 2026-05-15 Material visual slice: cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1 --message-format short --color never (3 passed)
  - 2026-05-15 Material visual slice: cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format short --color never (35 passed)
  - 2026-05-15 Material visual slice: cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --message-format short --color never (25 passed)
  - 2026-05-15 Material visual slice: cargo test -p zircon_editor --lib command_stream --locked --jobs 1 --message-format short --color never (7 passed)
  - 2026-05-15 Material visual slice: cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1 --message-format short --color never (2 passed)
  - 2026-05-15 Material visual slice: cargo test -p zircon_editor --lib gpu_surface_commands_preserve_chrome_corner_radius --locked --jobs 1 --message-format short --color never (1 passed)
  - 2026-05-15 Material visual slice: cargo check -p zircon_editor --lib --tests --locked --jobs 1 --message-format short --color never (passed)
  - 2026-05-15 Material visual slice: cargo fmt --all -- --check (passed)
  - 2026-05-16 Material visual live capture: tools/ui-profile-capture.ps1 -ScenarioList startup,idle_hover -OutputRoot .codex/material-ui-capture -SkipBuild -AutoCloseSeconds 5 -AutoInteract -RequireScenarioEvidence (startup 20260516-000244 passed; idle_hover 20260516-000253 recorded redraw/GPU work with zero alerts but missed the strict batch gate)
  - 2026-05-16 Material visual live capture: tools/ui-profile-capture.ps1 -Scenario click -OutputRoot .codex/material-ui-capture -SkipBuild -AutoCloseSeconds 5 -AutoInteract -RequireScenarioEvidence (20260516-000343 passed)
  - 2026-05-16 Material feedback emphasis: rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/painter/theme.rs zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs zircon_editor/src/tests/host/retained_window/native_material_painter.rs (passed)
  - 2026-05-16 Material feedback emphasis: Python tomllib parse for zircon_editor/assets/ui/theme/editor_material.v2.ui.toml, zircon_editor/src/tests/fixtures/ui_legacy/theme/editor_material.ui.toml, and zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml (passed)
  - 2026-05-16 Material feedback emphasis: git diff --check on touched Material feedback files (passed; line-ending warnings only)
  - 2026-05-16 Material feedback emphasis: cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1 --message-format short --color never (blocked before compile by existing Cargo.lock mismatch; lockfile left unchanged)
  - 2026-05-16 Material full-component design matrix: cargo test -p zircon_editor --lib material_ui_component_design_matrix --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=E:\cargo-targets\zircon-mui-design-matrix (2 passed)
  - 2026-05-16 Material foundation catalog split: rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/component_catalog.rs zircon_runtime/src/ui/tests/component_catalog/material_foundation/mod.rs zircon_runtime/src/ui/tests/component_catalog/selection_state.rs and Python static module-boundary validation (passed; 9 modules, max 247 lines; root test file 933 lines)
  - 2026-05-16 Material Lab AppBar toolbar: rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/material_component_lab/shell.rs, Python tomllib AppBar validation (passed: children=5, chips=4, selectors=4, status=success), cargo metadata --locked --no-deps --format-version 1, and git diff --check on the AppBar/docs slice (passed; line-ending warnings only)
  - 2026-05-16 Material Lab section headers: rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/material_component_lab/shell.rs, Python tomllib section-header validation (passed: headers=8, chips=16), cargo metadata --locked --no-deps --format-version 1, and git diff --check on the section-header/docs slice (passed; line-ending warnings only)
  - 2026-05-16 Material Lab side panel: rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/material_component_lab/shell.rs, Python tomllib side-panel validation (passed: rows=5, chips=13, selectors=8), cargo metadata --locked --no-deps --format-version 1, and git diff --check on the side-panel/docs slice (passed; line-ending warnings only)
  - 2026-05-16 Material Lab content surface: rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/material_component_lab/shell.rs, Python tomllib content-surface validation (passed: axis=Vertical, gap=14, selector=1), cargo metadata --locked --no-deps --format-version 1, and git diff --check on the content-surface/docs slice (passed; line-ending warnings only)
  - 2026-05-17 Material Lab Drawer count rows: rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/material_component_lab/shell.rs, Python tomllib drawer-count validation (passed: rows=7, counts=7, selectors=4), cargo metadata --locked --no-deps --format-version 1, and git diff --check on the Drawer count/docs slice (passed; line-ending warnings only)
  - 2026-05-17 Material Lab Drawer count rows focused Rust retry: cargo test -p zircon_editor --lib material_component_lab_shell_keeps_material_lab_layout_regions --locked --jobs 1 --message-format short --color never failed before reaching Zircon tests while compiling `wgpu-hal v29.0.3` DX12; `cargo tree` shows `wgpu-hal` using `windows 0.62.2` directly while `gpu-allocator 0.28.0` brings `windows 0.61.3`, producing incompatible `ID3D12Device`/`ID3D12Heap` types. Lockfile left unchanged.
  - 2026-05-17 typed Button style binding: WSL `cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1`, `cargo test -p zircon_runtime --lib material_button_style --locked --jobs 1`, `cargo test -p zircon_runtime --lib v2_asset --locked --jobs 1`, `cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1`, `cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1`, and `cargo test -p zircon_editor --lib template_assets --locked --jobs 1` passed in the focused Linux target. Windows native focused Cargo remains blocked before Zircon tests by the same `wgpu-hal` DX12/windows dependency mismatch.
  - 2026-05-17 Material Lab section grids: rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/material_component_lab/shell.rs, Python tomllib section-grid validation (passed: sections=8, columns=2), cargo metadata --locked --no-deps --format-version 1, and git diff --check on the section-grid/docs slice (passed; line-ending warnings only)
  - 2026-05-18 Floating Action Button slice: rustfmt --edition 2021 --check zircon_runtime/src/ui/component/catalog/material_foundation/shared.rs zircon_runtime/src/ui/component/catalog/material_foundation/inputs.rs zircon_runtime/src/ui/tests/component_catalog/material_foundation/mod.rs zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs zircon_editor/src/ui/retained_host/host_contract/painter/theme.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs zircon_editor/src/tests/ui/boundary/material_component_lab/feedback.rs zircon_editor/src/tests/ui/boundary/material_component_lab/inventory.rs zircon_editor/src/tests/ui/boundary/material_component_lab/support.rs zircon_editor/src/tests/host/retained_window/native_material_painter.rs (planned/rerun after review fix); git diff --check on the focused FAB asset/theme/docs/catalog/test files passed with LF/CRLF warnings only; Cargo intentionally skipped for milestone-first implementation cadence and known external render compile blocker risk
  - docs-only audit: git diff --check docs/ui-and-layout/material-ui-token-component-audit.md docs/ui-and-layout/index.md .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md .codex/sessions/20260506-2211-slate-milestone-m0-baseline.md
doc_type: milestone-detail
---

# Material UI Token And Component Audit

## Purpose

本文件记录 M2.1a/M2.1b/M2.2b：先把 `dev/slint` Material 参考与当前 Zircon `.ui.toml` Material token、style class、meta component、runtime layout 支撑放在同一张表里，再把 token role convergence 和 meta component root forwarding 落到 `editor_material.ui.toml`、`material_meta_components.ui.toml`、shared `.ui.toml` compiler token resolver 与 focused tests。

当前审计保持三个边界：

- Slint Material 是编辑器视觉与组件状态参考，不复制 Slint API。
- Zircon 的行为真源仍是 `.ui.toml` meta component + shared `UiSurfaceFrame` + runtime layout/render/input DTO。
- M2.1b/M2.2b 修改 shared token resolver、Material `.ui.toml` token/style assets、focused tests 和文档；不修改 sibling-owned native host 或 runtime layout implementation。

## 2026-05-07 2D Slate Chain Acceptance

Material UI + `.ui.toml` 的 2D editor/runtime 链路已按 focused boundary 收束：runtime `material_layout` 23 / 0，editor `component_showcase` 19 / 0，Material meta export/component asset coverage 通过，Welcome Material text field keyboard input、Welcome Material button callback、projected showcase button click、Inspector Material roots、native Material painter state palette 均通过。Shared Slate core 同步通过 runtime `hit_grid` 12 / 0 和 `event_routing` 23 / 0，其中包含 same-target mouse move idle/no rebuild、keyboard/text/IME、release-inside/outside click、focus/capture、scroll fallback 和 hit-grid visibility/clip/disabled 语义。

本次验收不声明 workspace-wide clean：当前仍保留既有 unused warning 和 unrelated formatting diffs；world-space UI 只保留进入同一 hit grid 的接口约束，不作为本轮 2D editor/runtime Material 完成阻塞项。

## Slint Reference Inventory

| Reference | Covered Material signal | Zircon use |
|---|---|---|
| `material_style_metrics.slint` | `size_*`、`icon_size_*`、`padding_*`、`spacing_*`、`border_radius_*` | M2.1b 需要把当前零散 `material_*` layout tokens 映射到稳定 density/spacing/radius roles |
| `material_palette.slint` | primary/secondary/tertiary/error/surface/outline/shadow/scrim/inverse/fixed roles、state-layer opacity、disabled opacity | M2.1b 需要补足 color role 与 state-layer opacity token，不只保留 editor 暗色主题色名 |
| `components/buttons/base_button.slint` | button padding 24/10、spacing 8、min 40、icon 18、disabled opacity 38% | 当前 `MaterialButton*` 已基本持有同级 layout attributes；仍缺透明度 token 和更完整状态矩阵 |
| `material.slint` exports | AppBar、Badge、CheckBox、Chip、Dialog、Drawer、DropDownMenu、Divider、Card、Button variants、IconButton variants、ListTile/ListView、Navigation、Progress、RadioButton、SearchBar、ScrollView、Slider、SnackBar、StateLayer/Ripple、SegmentedButton、Switch、TabBar、TextField、TimePickerPopup、ToolTip、MenuItem 等 | 当前 Zircon 已覆盖基础 controls 和 editor showcase 所需组件；navigation/surface/dialog/tooltip/snackbar/chip/radio/card 等还不是 M2 第一批闭环组件 |

## Token Comparison

| Domain | Slint Material roles | Current Zircon tokens/classes | Status | M2.1b target |
|---|---|---|---|---|
| Density and size | `size_32/36/40/48/56` 等固定尺寸阶梯 | Shared style pass fixes `material_density_compact_height=32`、`material_density_default_height=40`、`material_density_prominent_height=56`; aliases map compact controls to 32, standard controls/list rows to 40, and field/prominent controls to 56 | Covered for shared component style pass | 后续可以逐步把旧别名折叠到 role token，但当前 role 已有验收 |
| Padding and spacing | `padding_4..56`、`spacing_2..52` | Shared style pass keeps tool UI padding restrained: button `12/6`、field `10/4`、list `10`、gap `6` | Covered for first controls | 保留 compact tooling values while using role names for button content padding, field content padding, list row gap, and menu item gap |
| Icon sizes | `icon_size_18/24/36/90` | `material_button_icon_size=18`、`material_icon_size=30`；M2.3 DTO/painter tests 已接 frame/DPI target size | Covered for M2.3 | 后续只需按真实组件扩展 decorative icon role |
| Radius | `border_radius_2/4/8/12/16/28` | `material_radius=5`，新增 `material_radius_small/medium/large/pill/control` | Covered for M2.1b | 可在 M8 清理 style 内旧孤立半径 |
| Color roles | `primary/on_primary/surface/surface_container_* /outline/error/shadow/scrim/inverse_*` 等 | 保留旧 editor dark token，并新增 `material_color_*` role aliases | Covered for M2.1b | 真实 scheme 切换可后续接主题工具 |
| State layers | hover/focus/press/disabled/drag opacity roles | 新增 `material_state_layer_opacity_*` 和 `material_disabled_opacity` | Covered for M2.1b | 具体 alpha 混合仍由 painter/theme resolve 逐步消费 |
| Focus ring | focus state and state layer | `material_focus_ring`、`material_focus_ring_width`、`material_focus_ring_offset` | Covered for M2.1b | 视觉 offset 细节进入 M3/M7 screenshot |
| Shadow and elevation | `shadow`、`shadow_15`、`shadow_30`、Elevation component | `material_shadow`、`material_shadow_soft`、`material_elevation_level*` | Covered for token contract | 真实 painter/renderer 阴影仍属 M4/M7 |
| Typography | `MaterialTypography` | `material_font_size` 旧别名和 `material_font_size_body/meta/title` roles | Covered for M2.1b | M6 再接真实 shaping 和 text layout |

## M2.1b Token Role Convergence

M2.1b keeps the current editor dark palette values but gives them stable Material role names. `editor_material.ui.toml` now exposes palette aliases such as `material_color_primary`, `material_color_on_primary`, `material_color_surface_container`, `material_color_outline_variant`, `material_color_shadow`, and `material_color_scrim`. It also carries state-layer opacity roles, disabled opacity, focus ring width/offset, shadow/elevation levels, radius roles, border width roles, and title/body/meta typography sizes.

`material_meta_components.ui.toml` now maps legacy component tokens such as `material_control_height`, `material_field_min_height`, `material_button_padding_x`, `material_icon_size`, `material_radius`, and `material_font_size` through density/spacing/icon/radius/typography role tokens. Existing component props can keep their stable token names while new components can target the role tokens directly.

The compiler support for this is `zircon_runtime/src/ui/template/asset/compiler/value_normalizer.rs`: `$token` and `$param.*` values are resolved recursively with a bounded depth. This lets role aliases survive through component props, layout attributes, and imported style rule declarations without duplicating literal values in every rule.

Focused evidence:

- `ui_document_compiler_resolves_nested_material_role_tokens_in_props_and_styles` proves nested role aliases resolve into component props, layout metrics, palette style values, border width, radius, and typography.
- `material_theme_declares_m2_role_tokens_and_styles_material_classes` proves the editor Material theme declares the M2 role tokens and still styles every Material meta component class.
- `material_meta_components_carry_shared_style_defaults_on_root_nodes` freezes the shared component root contract for state props, variants, radius, border width, font size, and min-height metrics on representative Button, text-button, menu-bar item, and GroupBox surfaces.
- Direct Material theme style literals for common radius, border width, typography, and backdrop scrim have been replaced with token references; actual painter/renderer shadow behavior remains an M4/M7 paint/debug concern.

## Current Meta Component Coverage

| Zircon meta component | Runtime/native root | Covered M2 group | Current strengths | Remaining M2 gap |
|---|---|---|---|---|
| `MaterialButtonBase`、`MaterialButton`、`MaterialTextButton` | `Button` | Button | 已有 padding、spacing、min width/height、icon size、interactive/focusable attrs | 需要统一 variant、disabled/hover/pressed/focus/selected/error state matrix 与 accessibility/callback tests |
| `MaterialIconButton` | `IconButton` | IconButton | 已有 square sizing、icon-only measurement、accessibility label 不参与文本测量的 runtime tests；M2.3a/b 已证明 icon 路径解析、frame/DPI target size 和 native/template SVG resize；M2.3c 已覆盖 selected/error/tone metadata 与 disabled/error/warning/active tint priority | 后续只剩 M2.2b accessibility/callback 禁用态行为测试 |
| `MaterialToggleButton`、`MaterialSwitch` | `ToggleButton` | Button/Navigation | checked/selected 和 list-like metrics 已有 | 需要 pressed/disabled/focus semantic tests 与 switch-specific visual token |
| `MaterialCheckboxRow`、`MaterialCheckBox` | `Checkbox` | List/Field | checked/selected、row metrics、interactive attrs 已有 | 需要 error/disabled/focus state matrix 和 keyboard default action |
| `MaterialOutlinedField`、`MaterialLineEdit`、`MaterialTextEdit` | `InputField` / `TextField` | Field | field padding、min height、value/placeholder measurement 已有 | 需要 text edit callback/binding/accessibility、error/focus/disabled states |
| `MaterialComboBox`、`MaterialSliderField`、`MaterialSlider`、`MaterialSpinBox` | `ComboBox` / `RangeField` / `NumberField` | Field/Menu | scalar/object option and numeric value measurement 已有 runtime tests | 需要 popup anchor、value change callback、disabled/focus matrix |
| `MaterialListItem`、`MaterialTableRow`、`MaterialStandardTableView` | `ListRow` / `TableRow` / `VirtualList` | List/Surface | list row/table row metrics and virtual-list support present | 需要 selected/hover/disabled/error semantics in component expansion tests |
| `MaterialMenuBar`、`MaterialMenuBarItem`、`MaterialMenuFrame`、`MaterialMenuItem` | `HorizontalBox` / `Button` / `ContextActionMenu` / `MenuItem` | Menu | popup anchor fields exist for menu/date/time frames; menu item root is semantic `MenuItem` | 需要 nested popup, close behavior, action callback, focus traversal, disabled owner tests |
| `MaterialTabWidget*`、`MaterialTabImpl`、`MaterialTabBar*` | `VerticalBox` / `Tab` / bar containers | Tabs/Navigation | tab semantic root, selected state, horizontal/vertical bar classes present | 需要 tab activation callback, selected/focus/hover disabled state matrix |
| `MaterialProgressIndicator`、`MaterialSpinner`、`MaterialScrollView`、`MaterialGroupBox` | `ProgressBar` / `Spinner` / `ScrollableBox` / `Group` | Surface | common native roles now accepted by runtime material layout support | 需要 surface/elevation tokens and progress/spinner state visuals |
| `MaterialDatePickerPopup`、`MaterialTimePickerPopup` | `ContextActionMenu` | Dialog/Menu | popup frame class and anchor attrs present | 需要 dialog/popup lifecycle, modal/backdrop, keyboard close and accessibility tests |

2026-05-17 MUI X inventory freeze: Date and Time Pickers are now recorded in the design matrix as a prototype-first MUI X planning row against <https://mui.com/x/react-date-pickers/>. The first Zircon asset is `material_mui_x_date_time_pickers.zui`, a compact field/popup-state placeholder whose implementation may reuse the existing `MaterialDatePickerPopup` / `MaterialTimePickerPopup` popup_open, anchor, and ContextActionMenu semantics before adding dedicated picker lifecycle tests.

## M2.2a State Matrix Baseline

状态矩阵按 Zircon 运行时语义而不是 Slint 控件类型命名：组件根节点必须透传状态属性，style/theme 可以选择如何呈现，但测试需要先证明状态没有在 meta component 展开时丢失。

| M2 group | Zircon components | Required states | Current pass-through | Missing state/test work |
|---|---|---|---|---|
| Button | `MaterialButton`、`MaterialButtonBase`、`MaterialTextButton` | default、hovered、pressed、focused、disabled、error | `MaterialButton` 已透传 hovered/pressed/focused/disabled/validation_level/text_tone；base/text button 还偏 layout/visual attrs | 给 base/text variants 补状态参数或明确只由 concrete button 承担；测试 root props、style state、click callback 和 accessibility label |
| IconButton | `MaterialIconButton` | default、hovered、pressed、focused、disabled、selected when toggle-like | hovered/pressed/focused/disabled 已透传；icon-only label 已作为 accessibility text 不参与测量 | M2.3 证明 icon frame/DPI/tint；M2.2b 证明 accessibility label 和 disabled/focus 不触发 click |
| Toggle/Check/Switch | `MaterialToggleButton`、`MaterialCheckboxRow`、`MaterialCheckBox`、`MaterialSwitch` | default、hovered、pressed、focused、disabled、selected/checked | toggle/checkbox row 已透传 hovered/pressed/focused/disabled/checked/selected；switch/check-box 基础 interactive attrs 已有 | 统一 `checked` -> `selected` 语义，测试 keyboard/default action 与 disabled owner |
| Field | `MaterialOutlinedField`、`MaterialLineEdit`、`MaterialTextEdit`、`MaterialComboBox`、`MaterialSlider`、`MaterialSpinBox` | default、hovered、focused、disabled、error、pressed when range/combo | outlined/text edit 透传 hovered/focused/disabled/validation_level；line edit 只有 validation_level；combo 目前以 popup_open 为主 | 补 line edit/combo/range/number 的 disabled/focused/hovered；测试 value binding、commit callback、error visual role 和 focus ownership |
| List | `MaterialListItem`、`MaterialTableRow`、`MaterialStandardTableView` | default、hovered、pressed、focused、disabled、selected | list item/table row 透传 hovered/pressed/focused/disabled/selected；virtual list 有 interactive attrs | 测试 selected row 与 hover/press 优先级、disabled row 不入 action route、virtual visible-window state 不丢失 |
| Menu | `MaterialMenuFrame`、`MaterialMenuItem`、`MaterialMenuBarItem` | default、hovered、pressed、focused、disabled、selected/checked、popup_open | menu frame 透传 popup_open/popup_anchor_x/y；menu item 透传 hovered/pressed/focused/disabled/checked/selected | 测试 nested popup anchor、disabled menu item、checked menu item、close-on-select、escape/blur close |
| Dialog/Popup | `MaterialDatePickerPopup`、`MaterialTimePickerPopup`、future dialog wrapper | default、focused、disabled descendants、modal/backdrop、popup_open、error when form-like | date/time popup 已透传 popup_open 和 anchor；runtime backdrop/dialog classes exist in theme | 需要 named Dialog meta component 或 explicit popup lifecycle tests；测试 modal focus trap/backdrop close/accessibility role |
| Tabs/Navigation | `MaterialTabImpl`、`MaterialTabWidget*`、future navigation controls | default、hovered、pressed、focused、disabled、selected | tab root 是 semantic `Tab`，透传 hovered/pressed/focused/disabled/selected/checked | 测试 tab activation callback、keyboard nav、selected tab style、disabled tab 跳过 |
| Surface | `MaterialGroupBox`、`MaterialScrollView`、`MaterialProgressIndicator`、`MaterialSpinner`、future Card/Dialog surface | default、hovered when interactive、focused when focusable、disabled descendants、error/warning where relevant | group/scroll/progress/spinner 有 style/layout support，但状态语义弱 | M2.1b 先补 radius/elevation/shadow/surface role tokens；M2.2b 再测 focusability/capability/accessibility |

M2.2b 的 focused tests assert three layers for each covered family: authored params are accepted by the component catalog, expanded root metadata carries the state/binding/callback/accessibility values, and runtime dispatch/layout/render consumers read those values from the shared tree rather than from editor host-specific side tables.

## M2.2b Closure Evidence

M2.2b is now covered by `zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs`. The split keeps the broad `global_material_surface_assets.rs` responsive/import contract small while giving Material root forwarding its own focused module.

| Contract | Test evidence | Covered controls |
|---|---|---|
| Role tokens and style rules | `material_theme_declares_m2_role_tokens_and_styles_material_classes` | every `material-*` class exported by `material_meta_components.ui.toml` |
| Stable state metadata | `material_meta_components_emit_stable_state_metadata` | Button, IconButton, ToggleButton, CheckboxRow, CheckBox, LineEdit/TextEdit/OutlinedField, ComboBox, Slider/SliderField, SpinBox, Switch, ListItem, TableRow, MenuItem, TabImpl |
| Shared root style defaults | `material_meta_components_carry_shared_style_defaults_on_root_nodes` | base Button, text Button, menu-bar item, and GroupBox roots now carry variants, state props, radius, border width, font size, and min-height metrics |
| Input and popup metadata | `material_meta_components_project_input_and_popup_contracts` | all interactive roots plus ComboBox/MenuFrame/DatePicker/TimePicker popup anchors |
| Binding/callback/capability/accessibility forwarding | `material_meta_component_roots_forward_interaction_accessibility_and_capability_params` | Button/IconButton/Field/List/Menu/Dialog-like Popup/Tabs/Navigation/Surface first-slice roots |
| Runtime expanded-root callback | `ui_document_compiler_preserves_reference_instance_bindings_on_expanded_root` | reference component expansion preserves instance binding id, click route, callback action and payload on the expanded root |
| Editor pane/template usage | `global_material_surface_assets` and `material_meta_components` focused gates | real editor/runtime `.ui.toml` asset scanning plus Material component state, input, popup and export contracts |

The 2026-05-07 shared component style pass keeps the public `TemplatePaneNodeData` shape unchanged. `pane_component_projection` already forwards `surface_variant`, `button_variant`, `text_tone`, `validation_level`, `selected`, `hovered`, `pressed`, `focused`, `disabled`, border, radius, and font metadata; `runtime_component_projection_preserves_material_visual_metadata` now asserts that full style/state set so root visual metadata cannot silently fall out of the native host contract.

Native painter state is intentionally centralized in `host_contract/painter/theme.rs` and consumed by `template_nodes.rs`. The current state priority is `disabled > validation error/warning > success/info > pressed > selected/checked/focused > primary/accent hovered/drop-target > primary/accent > hovered/drop-target > default`; focused painter tests cover default, hover, pressed, selected/checked/focused border, disabled, primary/accent, warning, error, success, info, disabled text colors, and rounded-corner pixels.

The 2026-05-16 feedback emphasis uses the generated Material references in `.codex/material-ui-capture/material-style-reference-1.png` and `.codex/material-ui-capture/material-style-reference-2.png` as visual anchors. It raises dark-tooling contrast without changing layout: hover surfaces move to a brighter blue-teal container with accent/focus borders, pressed surfaces move to a deeper teal container, selected and checked states share the saturated selected container, and focused/pressed/selected borders use the 2px focus-ring token. Primary/accent controls also get an explicit hover fill instead of remaining visually flat, so buttons, icon buttons, toggles, rows, tabs, menu items, and field-like controls show immediate feedback through the same retained painter path.

The 2026-05-17 Button visual slice makes `Button` the first single-component polish target after the broad Material Lab pass. The editor retained-host painter now resolves typed `ResolvedButtonStyle` variant/color/interaction state together with legacy `button_variant` and `surface_variant` metadata: contained primary uses the accent container, contained error uses the error container, outlined uses the inset surface, explicit text buttons can render without a fill, pressed/focused states force the focus-ring border, and disabled state mutes both fill and border before any click route is considered. The Component Showcase keeps `ButtonDemo` as the routed primary sample and adds visible outlined, text, danger, and disabled Button rows; `material_buttons.zui` labels its primary sample as a contained Button while preserving the Material Lab card structure. Focused validation added native painter pixel samples, projected hit-frame equality for `ButtonDemo`, disabled Button click rejection, and projection assertions for Button variant/state/input metadata.

The Button Group Milestone 1 slice reuses that Button style contract rather than adding a new grouped-style DTO. `material_button_group.zui` now keeps the strict Material Lab card root but changes the sample into a structural horizontal group with three child `Button` segments. The group records `button_group_orientation`, `button_group_segment_count`, and `button_group_disabled_propagates`; each segment records `button_group_attached_radius = first|middle|last` while retaining `button_variant`, `button_color`, `button_size`, `icon_placement`, and the normal hover/pressed/focus/disabled props. The sample container is intentionally non-dispatchable and has no `MaterialLab/*` feedback event; the child segments carry their own non-lab click routes so Button Group stays structural while segment buttons own click/press/focus behavior. Theme selectors `.material-button-group-*` express the first/middle/last border and radius semantics with one outer group border and zero-radius, borderless segment interiors, so future painter pixel tests can check shared borders without changing Button text/icon layout.

The Floating Action Button Milestone 1 slice also stays on the existing Button typed style contract. `FloatingActionButton` in the runtime catalog now declares `button_color`, `button_size`, `icon_placement`, local `button_shape = circular|extended|pill`, and source-derived `fab_style = small|standard|large` metadata, with defaults of contained primary, medium, icon-only, circular, standard FAB, elevated surface, `corner_radius = 16.0`, zero border, `elevation = 3.0`, and `hover_elevation = 4.0`. `material_floating_action_button.zui` keeps exactly one visible `material-lab-sample` node and one `MaterialLab/FloatingActionButton/Click` feedback route on that sample, but the sample is now a compact `HorizontalBox` containing circular, small circular, and extended child examples. The extended child is a `Button` with leading icon placement and label text; the circular examples are `IconButton` children with FAB radius and icon-only placement. Theme selectors `.material-fab-*` express the shape/size/interaction semantics with Material role tokens. Elevation now reaches the retained host contract as numeric `TemplatePaneNodeData::elevation`, and the native template painter emits a soft offset rounded shadow quad before elevated FAB surfaces using `material_shadow` parity. This is painter-backed editor evidence only; runtime WGPU elevation remains future work.

The Toggle Button Milestone 1 slice continues the same style-contract direction. `ToggleButton` now declares the shared Button typed style fields and a small local `selection_state = exclusive|multiple` enum instead of inventing a separate toggle DTO. The prototype sample is a route-bearing `HorizontalBox` with one `MaterialLab/ToggleButton/Toggle` feedback route, while its child `ToggleButton` examples are non-dispatchable visual states for exclusive selected, exclusive hover, multiple checked, and disabled. `selected` and `checked` intentionally travel together for selected toggle examples so existing state priority paints them as Material selected containers; `selection_state` records group semantics for later runtime exclusivity tests. Theme selectors `.material-toggle-button*` freeze outlined default, hover, pressed, focus, selected, disabled, row, and child-state styling without changing the Button painter surface.

The Text Field Milestone 1 slice follows the MUI `TextField` source split between `Input`, `FilledInput`, and `OutlinedInput` without adding a new shared field DTO yet. `TextField` in the Material foundation catalog now declares `variant = outlined|filled|standard`, `label`, `value_text`, `placeholder`, `helper_text`, `multiline`, and `select_mode`, with text-backed descriptors moved into `material_foundation/text_inputs.rs` so the input catalog family stays under its split-module size budget. `material_text_fields.zui` keeps exactly one route-bearing `material-lab-sample` node and one `MaterialLab/TextFields/Change` route, but the sample is now a compact `HorizontalBox` containing non-dispatchable child fields for outlined focused label, filled helper text, standard underline, error helper, and disabled no-edit. Theme selectors `.material-text-field*` freeze outlined, filled, standard, focused, error, disabled, and row styling through existing Material tokens; helper-label layout and live multiline/select-mode behavior remain runtime support work for later slices.

The Textarea Autosize Milestone 1 slice extends that text-backed catalog path without adding a standalone style DTO. `TextareaAutosize` now declares `variant = outlined|filled|standard`, `value_text`, `placeholder`, `helper_text`, `multiline`, `autosize`, `min_rows`, and `max_rows`; defaults keep it multiline/autosize with a 2-to-8 row clamp. `material_textarea_autosize.zui` keeps one route-bearing `HorizontalBox` sample and one `MaterialLab/TextareaAutosize/Change` route, while child `TextareaAutosize` examples stay non-dispatchable and freeze minimum rows, maximum row clamp, focused autosize, error helper, and disabled no-edit metadata. Theme selectors `.material-textarea*` mirror the TextField tone set but explicitly name row-constraint/autosize states so the later layout pass can treat row changes as layout dirty rather than render-only styling.

The Number Field Milestone 1 slice keeps numeric state typed instead of falling back to string display props. The Material foundation `NumberField` descriptor now declares defaults for numeric `value`, `min`, `max`, `step`, and `large_step`, and exposes the full drag/change event family: `Focus`, `BeginDrag`, `DragDelta`, `LargeDragDelta`, `EndDrag`, `ValueChanged`, and `Commit`. `material_number_field.zui` keeps one route-bearing `HorizontalBox` sample and one `MaterialLab/NumberField/Change` route, while child `NumberField` examples stay non-dispatchable and freeze stepper, clamped max, drag-active chip, error, and disabled metadata. Theme selectors `.material-number-field*` mirror the field tone set and add explicit clamped and drag-active classes so later retained-host/painter work can distinguish numeric drag feedback from text-field focus styling.

The Select Milestone 1 slice records popup and option state without implementing a new popup placement engine. `Select` now carries `variant = outlined|filled|standard`, string `value`, `value_text`, `selected_options`, `options`, `disabled_options`, `focused_options`, `hovered_options`, `pressed_options`, `multiple`, `display_empty`, and `popup_open` metadata, plus `Focus`, `OpenPopup`, `SelectOption`, `ClosePopup`, and `ValueChanged` events. `material_selects.zui` keeps one route-bearing `HorizontalBox` sample and one `MaterialLab/Selects/Change` route, while child `Select` examples stay non-dispatchable and freeze closed placeholder, open popup, selected option, multi-chip, and disabled option states. Theme selectors `.material-select*` mirror the field tone set and add explicit open, selected, multiple, and disabled classes for later popup/option painter tests.

The Autocomplete Milestone 1 slice uses the same selection owner module but adds text-query and chip metadata instead of introducing a separate shared style DTO. `Autocomplete` now carries `query`, string `value`, `value_text`, `selected_options`, `options`, `filtered_options`, `disabled_options`, `focused_options`, `hovered_options`, `pressed_options`, `matched_options`, `multiple`, `free_solo`, and `popup_open`, plus `Focus`, `ValueChanged`, `OpenPopup`, `SelectOption`, `ClosePopup`, and `RemoveElement` events. `material_autocomplete.zui` keeps one route-bearing `HorizontalBox` sample and one `MaterialLab/Autocomplete/Change` route, while child `Autocomplete` examples stay non-dispatchable and freeze query-filtered results, open popup, selected option, multi-chip, and disabled option states. Theme selectors `.material-autocomplete*` mirror the Select tone set and add explicit query, open, selected, multiple, and disabled classes so later popup/filter/chip behavior can distinguish search matching from plain select state.

The Checkbox Milestone 1 slice stays in the basic input descriptor module because it is a local boolean/tri-state toggle, not a popup or grouped-selection owner. `Checkbox` now carries `checked`, `indeterminate`, `label_click_toggles`, and `indeterminate_resolves_to_checked` metadata, plus `Focus` and `ValueChanged`; the recorded transition policy is that clicking an indeterminate checkbox resolves to checked. `material_checkboxes.zui` keeps one route-bearing `HorizontalBox` sample and one `MaterialLab/Checkboxes/Toggle` route, while child `Checkbox` examples stay non-dispatchable and freeze unchecked, checked, indeterminate, error, and disabled states. Theme selectors `.material-checkbox*` define the row, unchecked, checked, indeterminate, error, focus/selected, and disabled states so later glyph/painter work can distinguish the indeterminate dash from a normal checked mark.

The Radio Group Milestone 1 slice also stays in the basic input descriptor module, but records grouped-selection metadata on `Radio` instead of adding a separate catalog id. `Radio` now carries `group_value`, `option_id`, `options`, `disabled_options`, `label_click_selects`, `exclusive_group`, and `keyboard_navigation`, plus `Focus`, `SelectOption`, and `ValueChanged` events. `material_radio_buttons.zui` keeps one route-bearing `HorizontalBox` sample and one `MaterialLab/RadioButtons/Change` route, while child `Radio` examples stay non-dispatchable and freeze selected, unselected, disabled, and error options against the same `group_value`. Theme selectors `.material-radio*` define row, selected, unselected, disabled, error, focus, and selected-state styling so later behavior tests can prove exclusive selection clears siblings and rejects disabled options before painter glyph work lands.

The Switch Milestone 1 slice stays in the same basic input descriptor module because it is also a local checked toggle, but records MUI-specific track/thumb interaction metadata instead of inventing a new renderer DTO. `Switch` now carries `checked`, `switch_size = small|medium`, `switch_color = primary|default|error`, `label_click_toggles`, `track_click_toggles`, and `thumb_draggable`, plus `Focus` and `ValueChanged` events. `material_switches.zui` keeps one route-bearing `HorizontalBox` sample and one `MaterialLab/Switches/Toggle` route, while child `Switch` examples stay non-dispatchable and freeze on, off, small, disabled, and error states. Theme selectors `.material-switch*` define the row, focused, selected, disabled, on, off, small, and error styling so later track/thumb painter work can distinguish local toggle state from generic selected paint state.

The runtime and editor input tests are now folder-backed by component family. Runtime catalog assertions keep `zircon_runtime/src/ui/tests/component_catalog/material_foundation/mod.rs` navigational for broad registry checks and move input-specific assertions into `material_foundation/inputs.rs` next to `selection_inputs.rs`. Editor Material Lab boundary tests live under `zircon_editor/src/tests/ui/boundary/material_component_lab/inputs/`; `mod.rs` keeps only shared helpers, while `toggle.rs`, `text.rs`, `numeric.rs`, `selection.rs`, `checkbox.rs`, `radio.rs`, and `switch.rs` own their component-family assertions. This keeps future Slider/Rating slices from pushing umbrella test files past the repository large-file threshold.

The same slice also introduces the first general typed style binding layer instead of a Button-only string helper. `zircon_runtime_interface::ui::style` owns serializable style values (`UiStyleColor`, `StyleDimension`, `ButtonVariant`, `ButtonColor`, `ButtonSize`, `ButtonIconPlacement`, `ButtonInteractionState`, and `ResolvedButtonStyle`); `zircon_runtime::ui::style` owns `StyleProperty`, `StyleField<P>`, weak `StyleSheetScope` resolution, and `resolve_button_style_from_values(...)` for the narrow editor/runtime bridge. The resolver keeps authored component overrides highest priority, walks near style scopes before far scopes, skips expired weak scopes, normalizes `ButtonVariant::Default` to Material text button semantics, accepts aliases such as `primary`/`filled` for contained and `hovered`/`active` for interaction states, and clamps resolved opacity. The style bridge is intentionally generic so future typed properties can be added beside Button without adding another host-local parser.

UI v2 cascade now separates authored inline style from static selector cascade. Static non-pseudo `resolve_static(...)` resolves selector rules without merging inline `node.style`, so `UiTemplateNodeMetadata.attributes` carries the baseline cascade and `style_overrides` carries authored inline/final overrides. Render extraction and editor typed projection read `style_overrides` first, then fall back to attributes, which keeps `.zui` inline `style = { self = ... }` as the highest-priority authored value while allowing runtime pseudo-state restyle to overlay selector-driven visual state. `editor_material.v2.ui.toml` now declares typed button fields on shared `.material-button`, `.material-button-primary`, `.material-text-button`, and `.material-icon-button` rules: contained/text variants, primary color, medium/small sizes, icon-only placement, and hover/pressed/focused/disabled `button_interaction_state` values are present in the same rules that already carry the existing dark Material colors. Focus rules for these button classes intentionally apply the focus-ring border without a fill change, preserving the stronger pressed background when pointer down also focuses the control.

Windows focused validation is no longer blocked by the `wgpu-hal` DX12/windows crate mismatch for this slice. The root cause was the dirty lockfile resolving `gpu-allocator 0.28.0` to `windows 0.61.3` while `wgpu-hal 29.0.3` uses `windows 0.62.2`; restoring only that `gpu-allocator` edge to `windows 0.62.2` made `cargo metadata --locked` and the Button gates compile on Windows. 2026-05-17 evidence: `ui_contract_spine` passed `6`, runtime `material_button_style` passed `4`, runtime `component_catalog` passed `45`, editor `native_material_painter` passed `5`, editor `native_host_contract` passed `40`, editor `pane_body_documents` passed `11`, and `cargo check -p zircon_app --features target-editor-host --locked --jobs 1` passed with only the existing `RuntimeSession::create` dead-code warning.

Live Button evidence is recorded in two places. The strict Material Lab click profile at `target/zircon-profiles/20260517-175506-material_lab_click` reported `1324` visible draw items batched into `146` GPU draw calls, `batch_success_rate=0.890`, `draw_reduction_ratio=9.068`, `dependency_density=0.015`, `layer_density=12.259`, `hit_consistency_samples=15 failed=0`, and `software_fallback_present_count=0`. Direct `UI Component Showcase` screenshots at `target/button-visual-slice/button-showcase-visual/` show primary, outlined, text, danger, and disabled Button rows in the first viewport. Screenshot parity is still not accepted: the GPU screenshot is globally brightness-shifted against forced softbuffer (`differing_sample_ratio=0.9992`, `average_channel_delta=68.03`), while the softbuffer capture remains visually aligned with the reference style.

The 2026-05-15 retained Material slice makes rounded corners a real paint contract instead of a token-only promise. `HostPaintCommand` carries `corner_radius`, `HostRecordedPaintKind::{Quad, Border}` preserves it in the recorded stream, `ChromeCommandKind::{Quad, Border}` forwards it to the GPU presenter, and `UiSurfaceCommandKind::{Quad, Border}` keeps the radius for runtime WGPU geometry. The software painter fills rounded surfaces and borders by pixel coverage, while WGPU emits rounded solid vertices for quad and border commands so the GPU presenter no longer draws Material controls as square quads.

The same slice shifts the editor Material palette from the older blue/orange-dark scheme to a darker teal/blue tool palette: default controls use roughly 10px radii, panels use 12px radii, primary/action controls can use pill radii, and success/info/warning/error tones are available as reusable surface variants. The first representative consumers are the workbench menu/activity controls, Inspector fields/actions, welcome controls, and component showcase shell; this is intentionally a visual/style convergence pass and does not change business routing or restore any Slint host path.

Implementation follow-up in this slice added root-level forwarding params for the first-slice Button/IconButton/Field/List/Menu/Dialog-like Popup/Tabs/Navigation/Surface representatives, and keeps `MaterialComboBox` forwarding `popup_anchor_x` and `popup_anchor_y` so popup positioning does not need an editor-host side table.

The 2026-05-16 full-component design matrix moves the next Material UI pass from ad hoc component additions to a coverage-driven inventory. `docs/ui-and-layout/material-ui-component-design-matrix.md` maps every public directory under `dev/material-ui/docs/data/material/components` to a Zircon retained UI shape, source reference, state contract, feedback expectation, and verification strategy. It deliberately classifies utility-only entries such as `click-away-listener`, `portal`, `no-ssr`, and `use-media-query` as behavior/utility rows instead of forcing them into drawn controls, while rows such as `masonry` and `popper` remain explicit `needs support` items until Zircon has the corresponding layout or placement capability.

## 2026-05-16 Material Component Lab Contract

The Material Component Lab slice turns the design matrix into executable retained assets. The shared style contract is still dark Material rather than a pixel copy of MUI light theme: default controls use 10px corner radii, surfaces use 12px radii, high-emphasis actions may use pill radii, and state priority remains `disabled > error/danger > warning > success/info > pressed > selected/checked/focused/open > hover > default`.

The `.zui` prototype rule is intentionally narrow: every component gets one `material_<component>.zui` file with `asset.kind = "component"`, a single `[components.<Name>]` root, Material state props (`hovered`, `pressed`, `focused`, `selected`, `checked`, `disabled`, `open`, `popup_open` where relevant), and a representative event route for interactive rows. Behavior utilities such as Portal, NoSsr, ClickAwayListener, CssBaseline, InitColorSchemeScript and useMediaQuery render compact placeholder rows that name their behavior owner instead of pretending to be visible controls.

Static and utility prototypes may still show Material state styling for visual reference, but they must not expose dispatchable input flags unless they also define a `MaterialLab/*` feedback route. `material_component_lab_non_route_prototypes_are_not_dispatchable_controls` locks this so automated click evidence cannot target a no-feedback placeholder. The inverse guard, `material_component_lab_route_prototypes_are_dispatchable_controls`, requires every route-bearing prototype to keep at least one dispatchable input flag so click, hover, focus, and press evidence can still reach the visual feedback sample.

`material_component_lab_feedback_routes_live_on_dispatchable_sample_nodes` tightens that route contract to the actual rendered node. Route-bearing prototypes must expose exactly one `MaterialLab/*` feedback node, that node must be the visible `material-lab-sample`, and the sample's `input_interactive`, `input_clickable`, `input_hoverable`, and `input_focusable` props must all be `true`. Non-route prototypes must have no hidden feedback node. This keeps automated hover/click evidence aligned with the component sample the user can see in the demo.

`material_component_lab_feedback_route_inventory_matches_expected_interactions` freezes the representative event semantics for all 48 route-bearing Material Lab prototypes. Button-like and surface-like samples stay on `Click`; form/navigation selectors stay on `Change`; Checkbox, Switch, ToggleButton, Accordion, and Tree View stay on `Toggle`; chart and Tooltip samples stay on `Hover`; Slider stays on `DragUpdate`; AgentChat, Chat Composer, and Date and Time Pickers stay on `Submit`. Button Group is the explicit structural exception: it has no sample-level `MaterialLab/*` feedback route because its child Button segments own the non-lab click routes. This prevents a future asset edit from accidentally downgrading specialized interaction examples into generic click samples.

`material_component_lab_interactive_inventory_matches_route_bearing_prototypes` keeps the test whitelist honest by requiring `INTERACTIVE_PROTOTYPES` to exactly match the prototype files that actually declare `MaterialLab/*` events. That makes additions and removals explicit instead of letting stale interactive inventory drift away from the `.zui` asset set.

`material_component_lab_places_every_prototype_once_in_visible_sections` closes the last demo-shell gap: imports are not enough. Every `material_*.zui` component must also have exactly one `prototype_*` node, and that node must be mounted under one of the visible Material Lab family sections. This prevents invisible regressions where a component file remains parseable but disappears from the demo window. `material_component_lab_prototype_nodes_match_material_file_stems` adds the stricter filename link: `material_buttons.zui` must be surfaced by `prototype_buttons` and that node must point at `MaterialButtonsPrototype`. The shell guard also freezes the component order inside Data Display, Feedback, Inputs, Layout, MUI X, Navigation, Surfaces, and Utils/Lab sections so the demo keeps the same MUI-family browsing shape as the design matrix.

`material_component_lab_shell_keeps_material_style_contract` freezes the demo chrome's shared Material look: the lab imports `editor_material.v2.ui.toml`, keeps the root on `material-lab-shell`, keeps AppBar/Drawer/status panel/section titles on `material-lab-card`, and preserves the dark shell/card colors, border tone, and 12px panel radius used by the generated visual reference.

The top AppBar is now a structured Material toolbar rather than a single title label. `appbar` is a `HorizontalBox` with `appbar_title`, a selected/focused `appbar_scope` chip for `MUI + MUI X`, a `74 prototypes` inventory chip, a success-toned `Static contracts` chip, and a `Capture ready` chip. Each chip keeps the same pill radius and 1px outline contract as the rest of the lab status UI, while the shell guard freezes the `material-lab-appbar-*` selectors and dark teal/success colors so the first viewport immediately shows scope, count, validation state, and capture readiness.

The content section headers now follow the same visible Material pattern as the AppBar and Drawer. Each family title node is a `HorizontalBox` with a title label, a count chip, and a status chip: Data Display `11 components / display`, Feedback `11 components / overlay`, Inputs `14 components / interactive`, Layout `5 components / structure`, MUI X `11 prototypes / mockups`, Navigation `9 components / routing`, Surfaces `5 components / chrome`, and Utils/Lab `8 utilities / utility`. The `material-lab-section-*` selectors reuse dark card, compact pill, and teal info tones so users can scan the component families before reading individual prototype cards.

The scrollable content region now has its own `material-lab-content-surface` class rather than blending directly into the shell background. It remains a vertical `ScrollableBox` with `gap = 14.0` and automatic scrollbars, but the added surface gives the prototype list a distinct `#151c22` container, 1px outline, and 12px panel radius. This keeps the center column visually grouped without changing the visible section order or individual component prototype assets.

Inside that surface, each component family section is now a two-column `GridBox` rather than a long vertical list. The section title spans both columns, and prototype cards fill row-major slots with 10px column/row gaps while preserving the official component order already frozen by the shell test. The resulting layout is still one vertical scroll stream, but the first viewport reads as a Material component gallery instead of a plain inventory list. The shell guard freezes the `GridBox` component kind, two-column container, expected row count per family, title `column_span = 2`, and every prototype card's row/column slot.

The left Drawer is now a structured Material navigation prototype instead of a plain multiline label. `drawer` is a `VerticalBox` with `drawer_title` and seven `material-lab-nav-item` rows for Inputs, Data Display, Feedback, Surfaces, Navigation, Layout / Utils, and MUI X. Each row is a `HorizontalBox` with a family label plus a compact count chip: Inputs `14`, Data Display `11`, Feedback `11`, Surfaces `5`, Navigation `9`, Layout / Utils `13`, and MUI X `11`. `drawer_inputs` carries the selected/focus example with `material-lab-nav-active` and `material-lab-nav-count-active`; `drawer_data_display` carries the hover example with `material-lab-nav-hover` and `material-lab-nav-count-hover`; every row forwards `selected`, `hovered`, `focused`, `disabled`, `corner_radius`, and `border_width` props. The shell test freezes the Drawer node order, horizontal label/count layout, count-chip text/classes, active/hover sample props, and the dark Material nav colors so future visual changes keep the AppBar + Drawer composition visible in the first viewport.

The right-side status panel is now structured instead of a multiline text label. `side_panel` is a `VerticalBox` with an `Interaction Contract` title, `Variant Chips` rows for family/response/appearance/layout, `Interaction Feedback` rows for Hover, Pressed, Focus, Selected, Disabled, and Error, and a `Capture Evidence` row for startup/hover/click. Each row uses `material-lab-side-row`; each chip uses `material-lab-side-chip` plus info/success/error emphasis classes where appropriate. This keeps the demo self-describing while making the feedback legend visually consistent with the AppBar, Drawer, section headers, prototype meta chips, and state pills.

The Material Lab boundary tests are now folder-backed under `zircon_editor/src/tests/ui/boundary/material_component_lab/`: `inventory.rs` owns import/prototype placement and parsed sample-node props, `feedback.rs` owns route and dispatchability contracts, `shell.rs` owns demo chrome and capture-script wiring, `projection.rs` owns retained-host projection, and `support.rs` owns shared fixture discovery. This keeps future component-family assertions from accumulating in one near-1000-line test file.

`material_component_prototype_sample_nodes_carry_typed_material_props` closes a weaker text-search gap in the `.zui` contract. Each prototype must have exactly one rendered `material-lab-sample` node, that node must keep the shared `material-control` class, and its parsed `props` table must carry typed Material variants (`surface_variant`, `button_variant`, `text_tone`, `validation_level`), boolean state/input flags (`hovered`, `pressed`, `focused`, `selected`, `checked`, `disabled`, `open`, `popup_open`, `input_*`), plus numeric `corner_radius = 10.0` and `border_width = 1.0`. The older state-presence guard remains useful as a broad file-level smoke check, but the parsed-node guard is the contract that proves the visible sample is actually renderable with the intended Material state metadata.

`material_component_prototype_roots_keep_card_layout_contract` freezes the card shell around that sample. Every prototype component must keep default classes `material-lab-prototype` and `material-surface`, a `VerticalBox` root with `material-lab-card`, `material-surface`, and `material_<component>`-derived class, fixed `104/120/140` height, stretch width, `VerticalBox` container with `6px` gap, and the stable `title`, `meta`, `sample`, `state_strip` child order. `title` remains a fixed-height `Label`; `meta` is now a fixed-height `HorizontalBox` with `material-lab-meta-strip`; `state_strip` is a fixed-height `HorizontalBox` with `material-lab-state-strip`. This keeps variant and state examples visual without destabilizing the card grid.

`material_component_prototype_meta_strips_cover_variants_and_layout_modes` turns the old descriptive meta label into a parsed visual contract. Every `material_*.zui` prototype now exposes four compact chips under `meta`: `meta_group` for the MUI component family, `meta_response` for response mechanism examples such as `click / press`, `change / commit`, `toggle / checked`, `hover / preview`, `drag / update`, `submit / send`, `static / display`, or `utility / none`, `meta_variant` for appearance variants such as `filled / outlined / text / tonal / elevated` or `outlined / filled / standard`, and `meta_layout` for layout mode examples such as `inline controls`, `overlay / inline`, `surface stack`, or `data viewport`. The response, variant, and layout chips mirror their text into typed `response_mechanism`, `variant`, and `layout_mode` props, carry `density = "compact"`, and keep `corner_radius = 999.0` plus `border_width = 1.0`, so the demo row is both visible and machine-checkable. The lab stylesheet gives the meta chips a subdued Material dark surface, green response emphasis, teal variant emphasis, and amber layout emphasis.

`material_component_prototype_state_strips_cover_core_feedback_examples` makes the state coverage visible in the demo rather than leaving it implied by props alone. Every `material_*.zui` prototype now includes eight compact state pills: `Default`, `Hover`, `Pressed`, `Focus`, `Disabled`, `Selected`, `Open`, and `Error`. Each pill is a parsed `Label` node with `material-lab-state-pill` plus a state-specific class, typed `hovered`/`pressed`/`focused`/`selected`/`checked`/`disabled`/`open`/`popup_open` booleans, `validation_level`, `text_tone`, `border_width = 1.0`, and `corner_radius = 999.0`. The lab stylesheet gives the strip an inset dark row and gives each pill its own Material dark state treatment: hover and pressed teal state layers, focus with a 2px teal ring, disabled muted colors, selected/open high-emphasis containers, and error coral text/border. `material_component_lab_shell_keeps_material_style_contract` freezes those selectors and color tokens next to the shell/card style contract.

The runtime catalog is now folder-backed under `zircon_runtime/src/ui/component/catalog/material_foundation/` so Inputs, Data Display, Feedback, Surfaces, Navigation, Layout and MUI X descriptors can grow independently. Every descriptor carries the Material default classes plus `density`, `surface_variant`, `button_variant`, `text_tone`, `validation_level`, `corner_radius`, and `border_width`, which gives the retained host, component lab, and painter tests a shared interface surface. `material_editor_foundation_catalog_stays_folder_backed_by_family` freezes that there is no restored monolithic `material_foundation.rs`, that the planned `mod/shared/inputs/data_display/feedback/surfaces/navigation/layout/mui_x` modules remain present, that non-shared modules are aggregated into the registry, and that each split file stays below the 300-line size budget. The matching tests now live in `zircon_runtime/src/ui/tests/component_catalog/material_foundation/`; input-control descriptor assertions are split into `inputs.rs` and selection-input descriptor assertions are split into `selection_inputs.rs` so the module root does not absorb every NumberField/Checkbox/Radio/ToggleButton or Select/Autocomplete schema detail while the generic selection/state coverage remains in `selection_state.rs`.

`material_component_lab_prototypes_map_to_foundation_catalog_descriptors` now ties the visible demo assets back to runtime component roles. Every `material_*.zui` prototype except the demo-only `material_about_the_lab.zui` must map to one or more descriptors in `UiComponentDescriptorRegistry::material_editor_foundation()`, and each mapped descriptor must expose the shared Material props and core state props used by the prototype samples. The companion MUI X guard freezes Tree View, Data Grid, Date and Time Pickers, Charts, chart subtype, Gauge, AgentChat, and Chat Composer mappings so the special requested prototypes cannot drift away from the runtime catalog while still appearing in the demo.

The UI surface shader contract follows the same split. `ui_material.wgsl` keeps separate solid, image, and blit entry points while centralizing UI-specific decisions: solid and image outputs pass through tint-ready helpers, those fragment outputs are premultiplied before the shared UI blend state, final blit preserves the already-rendered offscreen UI texture without double premultiplication, future rounded-SDF parameters have a named helper path, and clip/mask softness terminology matches the Unity Canvas and Unreal Slate references. Current rounded fill and border geometry still comes from `geometry.rs`; the shader file is the stable handoff point for moving more Material primitives to GPU-side SDF without changing `UiSurfaceCommandKind`.

MUI X prototypes are Community-visible structural mocks: Tree View, Data Grid, Date and Time Pickers, Charts, and AgentChat record state, layout, and feedback semantics without implementing commercial-only behaviors such as server data adapters, advanced virtualized editing, full picker lifecycle, or full chart engines. The lab orders this section as Tree View, Data Grid, Date and Time Pickers, aggregate Charts, Line, Bar, Pie, Sparkline, Gauge, AgentChat, and Chat Composer so the special components requested in the plan stay visible before generic chart subtypes. Those gaps stay explicit as `needs support` until the retained host has the required data, picker, and rendering capability.

The visual evidence route is direct rather than menu-driven. `zircon_app` accepts `--builtin-view editor.material_component_lab`, and `tools/ui-profile-capture.ps1` exposes `material_lab_startup`, `material_lab_hover`, and `material_lab_click` scenarios that use that route. The intended capture command is:

```powershell
tools/ui-profile-capture.ps1 -ScenarioList material_lab_startup,material_lab_hover,material_lab_click -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 4 -SkipBuild
```

These scenarios keep the default no-argument editor startup on `editor.ui_component_showcase` while allowing the Material Component Lab to generate startup, hover, click, GPU batching, hit-consistency, and screenshot artifacts without manual menu navigation.

The 2026-05-16 closeout connects the Material Lab prototype event routes to retained feedback. All `MaterialLab/*` bindings are registered as builtin template bindings, dispatched as `MaterialComponentLab` feedback events, and converted to `PAINT_ONLY` host effects so clicks update visual feedback without changing editor business state. `material_component_lab_feedback_events_use_consistent_ids_routes_and_kinds` requires each `.zui` event to keep `id`, dotted `route`, and event kind tail in sync, and `material_component_lab_feedback_events_are_registered_as_builtin_bindings` freezes the source-level registry link and `EditorUiEventKind` so a new `.zui` feedback event cannot be added with a missing or mismatched builtin binding row. The capture script now accepts comma-separated `-ScenarioList` values, limits `material_lab_click` auto-interaction to live `template_controls`, and treats dependency-bound hover patches as valid evidence when draw-call reduction is not possible. Evidence from one command passed `-RequireScenarioEvidence` for all three lab scenarios: `target/zircon-profiles/20260516-123734-material_lab_startup`, `target/zircon-profiles/20260516-123745-material_lab_hover`, and `target/zircon-profiles/20260516-123756-material_lab_click`. The click capture reported `dirty_paint_only_count=1`, `redraw_region_count=2`, `presentation_rebuild_count=0`, `dirty_layout_count=0`, `dirty_presentation_count=0`, `software_fallback_present_count=0`, and `alerts=0`; startup and hover also reported zero alerts and zero software fallback.

Focused validation for this closeout passed on 2026-05-16: editor `material_ui_component_design_matrix`, `material_component_lab`, `material_meta_component_contracts`, `native_material_painter`, and `component_showcase`; runtime `ui_surface`, `component_catalog`, `material_layout`, and `event_routing`; `cargo fmt --all -- --check`; and PowerShell parser validation for `tools/ui-profile-capture.ps1`.

A follow-up guard now freezes the Material Lab shell itself: `material_component_lab_shell_keeps_material_lab_layout_regions` asserts the root AppBar/body split, body Drawer/content/status-panel split, vertical content scroll axis, official component-family section order, and visible interaction-feedback legend. Date and Time Pickers raises the expected inventory to 74 prototypes, 74 imports, 63 MUI Core rows, 11 MUI X rows, and 48 authored `MaterialLab/*` interaction routes plus the structural Button Group child-route exception. The newest Material Lab route is `MaterialLab/MuiXDateTimePickers/Submit`, which brings field commit evidence into the same MUI X feedback contract as the existing Tree, Data Grid, Charts, Gauge, AgentChat, and Chat Composer prototypes. `material_component_lab_mui_x_prototypes_define_feedback_routes` now locks this expectation so every MUI X prototype has a Material Lab feedback route.

The design matrix now expands the MUI X aggregate chart/chat entries into searchable subtype rows for Line Chart, Bar Chart, Pie Chart, Sparkline, Gauge, and Chat Composer, with Date and Time Pickers integrated as the eleventh explicit MUI X row. The 2026-05-17 WSL matrix validation covered all 63 local MUI Core docs rows, 11 explicit MUI X rows, and the required response, appearance, layout, `.zui`, and validation terminology: `cargo test -p zircon_editor --lib material_ui_component_design_matrix --locked --jobs 1` passed 4 tests. The matrix `.zui` references now use actual prototype filenames; the Rust boundary test `material_ui_component_design_matrix_names_existing_zui_prototypes` verifies explicit `material_*.zui` references resolve to existing files.

The 2026-05-17 WSL Material Lab validation for this repair passed after one contract repair: an initial `cargo test -p zircon_editor --lib material_component_lab --locked --jobs 1` reached assertions and failed because `material_buttons.zui` had Button state-strip pills while the documented Material Lab contract requires non-dispatchable Label state pills. Restoring only those eight state-strip nodes to `Label` kept the routed `buttons_Sample` as a real Button and made the rerun pass 27 tests. The runtime catalog gate then passed after a lower shared asset aggregate export fix: `zircon_runtime/src/asset/assets/mod.rs` now exposes the child material/shader DTOs already expected by the crate facade (`MaterialTextureSlotValue`, `ShaderTextureSlotAsset`, `ZMaterialDocument`, and `ZShaderTextureSlotDocument`), and `cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1` passed 45 tests in WSL.

## Slint Export Gap

Current Zircon first-class Material meta components cover the M2 first slice around Button, IconButton, Field, List, Menu, Tabs and basic Surface controls. The following Slint-exported families are still intentionally outside first-slice coverage or only represented by a lower-level Zircon primitive:

| Slint family | Current Zircon position | Suggested owner |
|---|---|---|
| AppBar, BottomAppBar, NavigationBar, NavigationDrawer, ModalNavigationDrawer, NavigationRail | Not first-class Material meta components; editor chrome currently comes through host/template paths | M2.2 for navigation semantics, M3 for editor host cutover |
| Dialog, FullscreenDialog, Modal, ModalBottomSheet, Snackbar, ToolTip | Popup/backdrop classes exist, but dialog lifecycle and modal focus contract are not closed | M2.2 + M5 |
| Badge, Chip, SegmentedButton, RadioButton, RadioButtonTile | Not in current first-slice meta component coverage | M2 follow-up after base controls |
| ElevatedCard, FilledCard, OutlinedCard, Elevation | Elevation/shadow tokens now exist, but no first-class card/elevation component behavior yet | M2.2 surface follow-up, M4/M7 paint/debug support |
| SearchBar, ListTile, Avatar, Divider | Can be composed from fields/list rows/basic primitives, but not frozen as named components | M2.2 or M3 according to real pane usage |

## Runtime Support Boundary

`zircon_runtime/src/ui/layout/pass/material.rs` currently treats a node as Material-layout-aware when it has authored layout attributes and a supported component role. The supported role set already includes:

- controls: `Button`, `IconButton`, `ToggleButton`, `Checkbox`, `InputField`, `TextField`, `ComboBox`, `RangeField`, `NumberField`, `Switch`
- lists and menus: `ListRow`, `MenuItem`, `ContextActionMenu`, `TableRow`, `VirtualList`, `Tab`
- status and generic roles: `ProgressBar`, `Spinner`, `Label`
- editor value roles: `ColorField`, `Vector2Field`, `Vector3Field`, `Vector4Field`

This is enough for M2.1 token unification and M2.2 focused behavior tests without adding another layout path. M2.3 now continues through the existing render/visual asset command path instead of teaching layout about icon pixels: render commands carry `UiVisualAssetRef`, paint conversion writes the frame+DPI target `pixel_size`, and native/template SVG tests prove the actual editor raster path can resize the same SVG at multiple target frames.

## Next Execution Targets

M2.1b is accepted: role tokens now exist in the theme/meta component TOML and are checked by `material_meta_component_contracts`. M2.2b is accepted for state/input/popup root forwarding plus binding, callback, focusable, capability, and accessibility params on the first-slice Material roots. M2.3a/b/c are accepted for SVG/icon path, frame/DPI target sizing, cache-size resource state, and Material state tint priority.

Next execution target is M3.1a: move menu, drawer, toolbar, document pane and floating panel entry points toward `.ui.toml + shared surface` data ownership while preserving the M2 contracts as the component-layer floor.
