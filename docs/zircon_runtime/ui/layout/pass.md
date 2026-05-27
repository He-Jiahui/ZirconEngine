---
related_code:
  - zircon_runtime_interface/src/ui/component/descriptor/slot_schema.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/layout/linear_sizing.rs
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/tests/ui_layout.rs
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
  - zircon_runtime/src/ui/template/build/slot_contract.rs
  - zircon_runtime/src/ui/template/build/container_inference.rs
  - zircon_runtime/src/ui/template/build/parsers.rs
  - zircon_runtime/src/ui/v2/surface_tree/layout.rs
  - zircon_runtime/src/ui/v2/surface_tree/slot.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/pass/engine.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/layout/pass/responsive_mui.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/layout/taffy_bridge.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/workbench/debug_reflector/mod.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/workbench/debug_reflector/tests.rs
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_editor/src/tests/host/retained_window/ui_debug_reflector.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/layout_routes.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/template_grid_flow.rs
  - zircon_runtime/src/ui/tests/mui_responsive_layout.rs
  - zircon_runtime/src/ui/tests/taffy_bridge.rs
  - zircon_runtime/src/ui/tests/taffy_layout_diagnostics.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass.rs
  - zircon_runtime/src/ui/tests/taffy_visual_verification.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/tests/runtime_ui_layout_routes.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
implementation_files:
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime/src/ui/template/build/slot_contract.rs
  - zircon_runtime/src/ui/template/build/container_inference.rs
  - zircon_runtime/src/ui/template/build/parsers.rs
  - zircon_runtime/src/ui/v2/surface_tree/layout.rs
  - zircon_runtime/src/ui/v2/surface_tree/slot.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/engine.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/layout/pass/responsive_mui.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/layout/taffy_bridge.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/workbench/debug_reflector/mod.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/workbench/debug_reflector/tests.rs
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_editor/src/tests/host/retained_window/ui_debug_reflector.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/layout_routes.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/template_grid_flow.rs
  - zircon_runtime/src/ui/tests/mui_responsive_layout.rs
  - zircon_runtime/src/ui/tests/taffy_bridge.rs
  - zircon_runtime/src/ui/tests/taffy_layout_diagnostics.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass.rs
  - zircon_runtime/src/ui/tests/taffy_visual_verification.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/tests/runtime_ui_layout_routes.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/template.rs
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
plan_sources:
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SPanel.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SBoxPanel.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedWidget.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedChildren.h
  - .codex/plans/ZirconEditor MUI Web Parity Plan.md
  - .codex/plans/UI Layout 架构评审与 Taffy 收敛计划.md
  - docs/superpowers/plans/2026-05-26-editor-ui-use-media-query-responsive.md
tests:
  - cargo test -p zircon_runtime --lib taffy_layout_docs_keep_visual_profile_gate --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-visual-doc-gate-20260527 --message-format short --color never (2026-05-27: passed, 1 passed; 0 failed; 2098 filtered out, after widening the shared accessibility binding diagnostic helper to the action-module boundary so the runtime test crate compiles)
  - tools/ui-profile-capture.ps1 -ScenarioList material_lab_startup,material_lab_hover,material_lab_click,drawer_resize -AutoInteract -RequireScenarioEvidence -CaptureSoftbufferScreenshot -AutoCloseSeconds 4 -SkipBuild (visual verification gate; run after building the editor profiling target; inspect `screenshot_gpu.png`, `screenshot_softbuffer.png`, `screenshot_diff.json`, `ui_hotspots.json`, and Runtime Diagnostics `Layout Engine` rows for Taffy/Zircon routing)
  - cargo build -p zircon_app --bin zircon_editor --profile profiling --features "target-editor-host profiling profiling-chrome" --locked (2026-05-27: passed, produced the profiling editor executable with existing warning noise)
  - tools/ui-profile-capture.ps1 -ScenarioList material_lab_startup -AutoInteract -RequireScenarioEvidence -CaptureSoftbufferScreenshot -AutoCloseSeconds 4 -OutputRoot target/zircon-profiles/layout-visual-20260527 -SkipBuild (2026-05-27: passed; artifacts under `target/zircon-profiles/layout-visual-20260527/20260527-193940-material_lab_startup`; `software_fallback_present_count=0`; GPU-vs-softbuffer `differing_sample_ratio=0.1014`, `average_channel_delta=6.7884`)
  - tools/ui-profile-capture.ps1 -ScenarioList material_lab_hover,material_lab_click,drawer_resize -AutoInteract -RequireScenarioEvidence -CaptureSoftbufferScreenshot -AutoCloseSeconds 4 -OutputRoot target/zircon-profiles/layout-visual-20260527 -SkipBuild (2026-05-27: hover and click passed; artifacts under `20260527-194117-material_lab_hover` and `20260527-194140-material_lab_click`; drawer_resize produced artifacts but failed the strict evidence gate because GPU-vs-softbuffer average channel delta exceeded 10.0)
  - tools/ui-profile-capture.ps1 -ScenarioList drawer_resize -AutoInteract -RequireScenarioEvidence -CaptureSoftbufferScreenshot -AutoCloseSeconds 4 -OutputRoot target/zircon-profiles/layout-visual-20260527 -SkipBuild (2026-05-27: after preserving primary interaction evidence across the softbuffer follow-up, drawer resize still failed strict screenshot parity; artifacts under `20260527-194629-drawer_resize` and `20260527-194735-drawer_resize`; geometry changed and `software_fallback_present_count=0`, but GPU-vs-softbuffer average channel delta stayed at 10.7813 then 10.8045 above the 10.0 threshold)
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/layout/pass/responsive_mui.rs zircon_runtime/src/ui/layout/pass/mod.rs zircon_runtime/src/ui/layout/pass/layout_tree.rs zircon_runtime/src/ui/layout/pass/incremental.rs zircon_runtime/src/ui/tests/template_grid_flow.rs zircon_runtime/src/ui/tests/mui_responsive_layout.rs zircon_runtime/src/ui/tests/mod.rs (2026-05-26: passed after MUI responsive breakpoint pre-pass)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zirconeditor-mui-breakpoints-20260526 --message-format short --color never (2026-05-26: passed with existing runtime warning noise)
  - cargo test -p zircon_runtime --lib ui_v2_mui_responsive_layout_recomputes_from_viewport_breakpoints --locked --jobs 1 --target-dir D:\cargo-targets\zirconeditor-mui-breakpoints-20260526 --color never -- --nocapture (2026-05-26: passed, 1 passed; 0 failed; 2067 filtered out, now includes responsive display/visibility/visible assertions)
  - cargo test -p zircon_runtime --lib template_mui_responsive_layout_recomputes_from_viewport_breakpoints --locked --jobs 1 --target-dir D:\cargo-targets\zirconeditor-mui-breakpoints-20260526 --color never -- --nocapture (2026-05-26: passed, 1 passed; 0 failed; 2067 filtered out, now includes responsive display/visibility/visible assertions)
  - git diff --check -- zircon_runtime/src/ui/layout/pass/responsive_mui.rs zircon_runtime/src/ui/layout/pass/mod.rs zircon_runtime/src/ui/layout/pass/layout_tree.rs zircon_runtime/src/ui/layout/pass/incremental.rs zircon_runtime/src/ui/tests/template_grid_flow.rs zircon_runtime/src/ui/tests/mui_responsive_layout.rs zircon_runtime/src/ui/tests/mod.rs docs/zircon_runtime/ui/layout/pass.md .codex/sessions/20260525-2142-editor-mui-render-layout-input.md (2026-05-26: passed with Windows LF/CRLF notices only)
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/layout/pass/responsive_mui.rs zircon_runtime/src/ui/tests/mui_responsive_layout.rs zircon_runtime/src/ui/tests/template_grid_flow.rs (2026-05-26 UseMediaQuery responsive gate: passed)
  - cargo test -p zircon_runtime --lib mui_responsive_layout --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=F:\cargo-targets\zircon-use-media-query-20260526 (2026-05-26 UseMediaQuery responsive gate: passed, 2 passed; 0 failed; existing warning noise)
  - cargo test -p zircon_runtime --lib template_mui_responsive_layout_recomputes_from_viewport_breakpoints --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=F:\cargo-targets\zircon-use-media-query-20260526 (2026-05-26 UseMediaQuery responsive gate: passed, 1 passed; 0 failed; existing warning noise)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=F:\cargo-targets\zircon-use-media-query-20260526 (2026-05-26 UseMediaQuery responsive gate: passed; existing warning noise)
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/layout/pass/responsive_mui.rs zircon_runtime/src/ui/tests/mui_responsive_layout.rs zircon_runtime/src/ui/tests/template_grid_flow.rs zircon_runtime/src/ui/component/catalog/material_foundation/layout_utilities.rs zircon_runtime/src/ui/tests/component_catalog/material_foundation/layout.rs (2026-05-27 UseMediaQuery breakpoint shorthand: passed)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-use-media-query-20260527 cargo test -p zircon_runtime --lib mui_responsive_layout --locked --jobs 1 --message-format short --color never (2026-05-27 UseMediaQuery breakpoint shorthand: passed, 2 passed; 0 failed; existing warning noise)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-use-media-query-20260527 cargo test -p zircon_runtime --lib template_mui_responsive_layout_recomputes_from_viewport_breakpoints --locked --jobs 1 --message-format short --color never (2026-05-27 UseMediaQuery breakpoint shorthand: passed, 1 passed; 0 failed; existing warning noise)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-use-media-query-20260527 cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never (2026-05-27 UseMediaQuery breakpoint shorthand: passed; existing warning noise)
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/layout/scroll.rs zircon_runtime_interface/src/ui/layout/mod.rs zircon_runtime_interface/src/ui/layout/slot.rs zircon_runtime/src/ui/layout/pass/slot.rs zircon_runtime/src/ui/layout/pass/arrange.rs zircon_runtime/src/ui/layout/pass/measure.rs zircon_runtime/src/ui/template/build/container_inference.rs zircon_runtime/src/ui/template/build/parsers.rs zircon_runtime/src/ui/template/build/slot_contract.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/layout_slots.rs zircon_runtime/src/ui/tests/template_grid_flow.rs zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/layout/scroll.rs zircon_runtime_interface/src/ui/layout/mod.rs zircon_runtime_interface/src/ui/layout/engine.rs zircon_runtime_interface/src/tests/layout_engine_contracts.rs zircon_runtime/src/ui/template/build/container_inference.rs zircon_runtime/src/ui/template/build/parsers.rs zircon_runtime/src/ui/template/build/slot_contract.rs zircon_runtime/src/ui/v2/surface_tree/layout.rs zircon_runtime/src/ui/v2/surface_tree/slot.rs zircon_runtime/src/ui/layout/pass/slot.rs zircon_runtime/src/ui/layout/pass/measure.rs zircon_runtime/src/ui/layout/pass/arrange.rs zircon_runtime/src/ui/tests/layout_slots.rs zircon_runtime/src/ui/tests/template_grid_flow.rs zircon_runtime/src/ui/tests/taffy_bridge.rs zircon_editor/src/tests/ui/boundary/material_component_lab/layout_theme.rs (2026-05-24: passed after formatting)
  - cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-masonry-interface --message-format short --color never (2026-05-24: passed)
  - cargo test -p zircon_runtime_interface --lib ui_layout_engine_request_maps_current_container_contracts_to_engine_families --locked --jobs 1 --target-dir D:\cargo-targets\zircon-masonry-interface --message-format short --color never (2026-05-25: passed, 1 passed; 0 failed; 114 filtered out)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-masonry-runtime --message-format short --color never (2026-05-25: passed; existing warning noise only)
  - cargo test -p zircon_runtime --lib masonry_shortest_column_layout_feeds_arranged_render_hit_from_one_surface_frame --locked --jobs 1 --target-dir D:\cargo-targets\zircon-masonry-runtime --message-format short --color never (2026-05-25: passed, 1 passed; 0 failed; 1983 filtered out)
  - cargo test -p zircon_runtime --lib masonry_sequential_layout_preserves_ordered_column_assignment --locked --jobs 1 --target-dir D:\cargo-targets\zircon-masonry-runtime --message-format short --color never (2026-05-25: passed, 1 passed; 0 failed; 1985 filtered out)
  - cargo test -p zircon_runtime --lib template_builder_maps_grid_and_flow_slots_into_shared_runtime_layout_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-masonry-runtime --message-format short --color never (2026-05-25: passed, 1 passed; 0 failed; 1990 filtered out)
  - cargo test -p zircon_editor --lib material_component_lab_masonry_sample_uses_runtime_descriptor_and_theme_selectors --locked --jobs 1 --target-dir D:\cargo-targets\zircon-masonry-editor --message-format short --color never (2026-05-25: timed out after 15 minutes while compiling dependencies without Rust diagnostics; matching target-dir Cargo/Rust processes were stopped)
  - Python tomllib static validation for zircon_editor/assets/ui/editor/material_components/material_masonry.zui and zircon_editor/assets/ui/theme/editor_material.v2.ui.toml (2026-05-25: passed; sample_children=8, columns=4, gap=8.0, sequential=true, `.MuiMasonry-item` and `.material-masonry-tile` selectors present)
  - cargo test -p zircon_runtime --lib layout_slots --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib template_grid_flow --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib taffy_bridge --locked --target-dir target/codex-shared-b (2026-05-11: passed, 2 passed)
  - cargo test -p zircon_runtime taffy -- --nocapture (2026-05-11: passed, 5 passed)
  - cargo test -p zircon_runtime size_box_contain_aspect_ratio_stays_zircon_owned -- --nocapture (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_runtime taffy -- --nocapture (2026-05-11: passed, 6 passed)
  - D:\cargo-targets\zircon-layout-impl\debug\deps\zircon_runtime-eb36ca4a90c1b648.exe taffy_layout_pass --nocapture (2026-05-20: passed, 7 passed)
  - D:\cargo-targets\zircon-layout-impl\debug\deps\zircon_runtime-eb36ca4a90c1b648.exe surface_frame_authority --nocapture (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime_interface layout_engine_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface --message-format short --color never (2026-05-20: passed, 3 passed)
  - cargo test -p zircon_runtime_interface layout_engine_contracts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-reason-counts-interface-20260522 --message-format short --color never (2026-05-22: passed, 5 passed after fallback_reason_counts update; --locked remained blocked by unrelated Cargo.lock sound/cpal drift and lock was restored)
  - cargo test -p zircon_runtime_interface layout_engine_contracts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-report-deser-20260522 --message-format short --color never (2026-05-22: passed, 6 passed after deserialization recomputes stale or missing aggregate counts; lock restored)
  - cargo test -p zircon_runtime_interface ui_surface_debug_snapshot_legacy_layout_report_recovers_fallback_reason_counts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-snapshot-deser-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after full surface debug snapshots recovered nested legacy layout-report counts; lock restored)
  - cargo test -p zircon_runtime_interface ui_layout_engine_selection_report_counts_unsupported_routes_separately --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface-unsupported-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime_interface ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface-contract --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_editor --lib ui_debug_reflector_model_projects_snapshot_rows_and_sections --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after route-detail model update)
  - cargo test -p zircon_editor --lib ui_debug_reflector_model_projects_snapshot_rows_and_sections --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-debug-tree-stats-20260523 --message-format short --color never (2026-05-23: deferred after Editor Debug Reflector Taffy tree-build line addition because unrelated active Cargo/rustc processes remain present)
  - cargo test -p zircon_editor --lib ui_debug_reflector_model --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-22: passed, 6 passed after fallback-reason summary display update; lock restored)
  - cargo test -p zircon_editor --lib ui_debug_reflector_model_displays_unsupported_layout_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_editor --lib runtime_diagnostics_payload_uses_active_ui_debug_snapshot_when_available --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_editor --lib runtime_diagnostics_payload_uses_active_ui_debug_snapshot_when_available --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after warmed rebuild)
  - cargo test -p zircon_editor --lib editor_ui_host_runtime_projects_pane_body_payload_metadata_into_root_attributes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after route-detail projection update)
  - cargo test -p zircon_editor --lib runtime_diagnostics_host_conversion_keeps_payload_reflector_text_and_overlay --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after route-detail host update)
  - cargo test -p zircon_editor --lib runtime_diagnostics_body_refresh_preserves_active_payload_reflector --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after route-detail refresh update)
  - cargo test -p zircon_editor --lib runtime_diagnostics_live_body_surface_populates_debug_reflector_rows_and_overlays --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_editor --lib builtin_editor_host_templates_export_layout_engine_route_reports --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-host-routes-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after real host-template route, frame-stability, render, hit, pointer-route, and no-silent-fallback coverage update)
  - cargo test -p zircon_runtime --lib taffy_native_flex_surface_frame_feeds_render_hit_and_pointer_dispatch --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_grid_slot_frame_policy_feeds_render_hit_and_pointer_dispatch --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-authority-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime --lib zircon_size_box_fallback_feeds_render_hit_and_pointer_dispatch --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-authority-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_wrap_surface_frame_feeds_render_hit_and_pointer_dispatch --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after WrapBox render/hit/pointer authority coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_flex_slot_policy_fallback_feeds_render_hit_and_pointer_dispatch --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after Taffy-eligible flex SlotFramePolicy fallback authority coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_flex_linear_slot_sizing_feeds_render_hit_and_pointer_dispatch --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after Taffy-native linear slot sizing authority coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_vertical_flex_linear_slot_sizing_feeds_render_hit_and_pointer_dispatch --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: passed, 1 passed after Taffy-native vertical linear slot sizing authority coverage; lock restored)
  - cargo test -p zircon_runtime --lib surface_frame_authority --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: passed, 8 passed after Taffy-native vertical linear slot sizing authority coverage joined the module gate; lock restored)
  - cargo test -p zircon_runtime --lib runtime_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-runtime-fixture-20260521 --message-format short --color never (2026-05-21: passed, 282 passed including both runtime fixture route tests after no-silent-fallback coverage update)
  - cargo test -p zircon_runtime --lib runtime_ui_layout_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-runtime-fixture-20260521 --message-format short --color never (2026-05-21: passed, 2 passed after real runtime fixture render, hit, pointer-route, and public-frame render-extract authority update)
  - cargo test -p zircon_runtime --lib runtime_ui_layout_routes --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-runtime-fixture-20260521 --message-format short --color never (2026-05-22: passed, 2 passed after recomputed fallback_reason_counts assertion; lock restored)
  - cargo test -p zircon_editor --lib builtin_editor_host_templates_export_layout_engine_route_reports --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-host-routes-20260521 --message-format short --color never (2026-05-22: passed, 1 passed after recomputed fallback_reason_counts assertion; lock restored)
  - cargo test -p zircon_runtime --lib surface_dirty_layout_preserves_unvisited_layout_engine_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-incremental-routes-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after incremental layout-engine report merge update)
  - cargo test -p zircon_runtime --lib surface_dirty_layout --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-incremental-routes-20260521 --message-format short --color never (2026-05-21: passed, 6 passed after incremental layout route replacement, stale route removal, and debug snapshot/JSON export coverage)
  - cargo test -p zircon_runtime --lib surface_dirty_layout --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-incremental-reason-counts-20260522 --message-format short --color never (2026-05-22: passed, 6 passed after incremental fallback_reason_counts merge coverage; lock restored)
  - cargo test -p zircon_runtime --lib runtime_inventory_fixture_reports_virtualized_list_zircon_fallback --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-runtime-fixture-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime --lib runtime_quest_log_fixture_exports_layout_engine_route_report --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-runtime-fixture-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after JSON export roundtrip update)
  - cargo test -p zircon_runtime --lib surface_debug_snapshot_json_exports_layout_engine_route_report --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib surface_debug_snapshot_json_exports_zircon_fallback_route_reason --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-diagnostics-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime --lib surface_debug_snapshot_json_exports --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-diagnostics-20260522 --message-format short --color never (2026-05-22: passed, 2 passed after fallback_reason_counts JSON assertions; lock restored)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_arranges_linear_wrap_and_grid_containers --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_uses_measured_text_and_image_desired_sizes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_maps_linear_slot_padding_without_fallback --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_maps_linear_slot_padding_and_cross_axis_alignment_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after linear slot padding/alignment Taffy-native coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_maps_vertical_linear_slot_padding_and_cross_axis_alignment_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after vertical linear slot padding/alignment Taffy-native coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_maps_wrap_slot_padding_and_cross_axis_alignment_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after WrapBox slot padding/alignment Taffy-native coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_maps_grid_slot_span_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after grid slot row/column span Taffy-native coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_expands_grid_tracks_for_out_of_bounds_slot_span_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after grid track expansion for out-of-bounds span coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_reports_non_finite_slot_padding_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after non-finite slot padding fallback coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_maps_grid_slot_padding_and_alignment_without_fallback --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_reports_grid_slot_alignment_without_fixed_extent_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after GridBox non-fixed alignment fallback coverage; lock restored)
  - cargo test -p zircon_runtime --lib layout_pass_reports_taffy_native_and_zircon_fallback_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime --lib layout_pass_reports_taffy_native_and_zircon_fallback_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-route-20260521 --message-format short --color never (2026-05-21: passed, 1 passed)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_aggregates_fallback_reason_counts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-taffy-reason-counts-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after layout-pass fallback_reason_counts aggregation coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_aggregates_distinct_fallback_reason_counts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after distinct fallback-reason aggregation coverage; lock restored)
  - cargo test -p zircon_runtime --lib slot_alignment --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 2 passed after linear main-axis and non-fixed cross-axis slot alignment fallback coverage; lock restored)
  - cargo test -p zircon_runtime --lib slot_sizing --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 3 passed after Auto, Stretch, and StretchContent linear slot sizing coverage; lock restored)
  - cargo test -p zircon_runtime --lib vertical_linear_slot_sizing --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: passed, 1 passed after VerticalBox Stretch linear slot sizing coverage; lock restored)
  - cargo test -p zircon_runtime --lib linear_slot_sizing_bounds --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after linear slot min/max Taffy-native bounds coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_maps_vertical_linear_slot_sizing_bounds_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: passed, 1 passed after VerticalBox linear slot min/max bounds coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_reports --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-22: passed, 2 passed after collapsed-child and child-placement fallback reason coverage; lock restored)
  - cargo test -p zircon_runtime --lib size_box_contain_aspect_ratio_stays_zircon_owned --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-route-20260521 --message-format short --color never (2026-05-21: passed, 1 passed after final route-test update)
  - cargo test -p zircon_runtime --lib taffy_layout_pass --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: passed, 29 passed after vertical main-axis linear slot sizing and vertical linear slot min/max bounds joined the Taffy pass gate alongside Auto/Stretch/StretchContent slot sizing, horizontal linear slot min/max bounds, grid slot span/track expansion, non-finite padding fallback, horizontal/vertical linear slot padding/alignment, unsupported slot-alignment fallback, WrapBox slot padding/alignment, fallback-reason, and distinct reason aggregation coverage; lock restored)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_ignores_flow_slot_linear_sizing_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: deferred after source/test addition because unrelated active Cargo processes are still present; Cargo.lock was clean when checked)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_reports_axis_constraint_priority_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: deferred after AxisConstraintPriority fallback reason addition because unrelated active Cargo/rustc processes restarted)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_reports_non_finite_axis_constraint_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: deferred after InvalidLayoutValue fallback guard addition because unrelated active Cargo/rustc processes remain present)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_reports_non_finite_linear_slot_sizing_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: deferred after InvalidLayoutValue fallback guard addition because unrelated active Cargo/rustc processes remain present)
  - cargo test -p zircon_runtime --lib taffy_layout_pass_reports_non_finite_container_config_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: deferred after container-level InvalidLayoutValue fallback guard addition because unrelated active Cargo/rustc processes remain present)
  - cargo test -p zircon_runtime --lib taffy_bridge_rejects_non_finite_style_inputs --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: deferred after Taffy bridge finite style input guard addition because unrelated active Cargo/rustc processes remain present)
  - cargo test -p zircon_runtime_interface layout_engine_contracts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never (2026-05-23: deferred after AxisConstraintPriority and InvalidLayoutValue fallback reason serialization coverage because unrelated active Cargo/rustc processes restarted)
  - cargo test -p zircon_runtime_interface layout_engine_contracts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-tree-stats-20260523 --message-format short --color never (2026-05-23: passed, 7 passed after Taffy tree-build stats contract coverage)
  - cargo test -p zircon_runtime --lib taffy_layout_report_exports_transient_tree_build_stats --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-tree-stats-runtime-20260523 --message-format short --color never (2026-05-23: timed out after 10 minutes in compile phase with no Rust diagnostics while unrelated Cargo/rustc processes were active)
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/layout/engine.rs zircon_runtime_interface/src/ui/layout/mod.rs zircon_runtime_interface/src/tests/layout_engine_contracts.rs zircon_runtime/src/ui/layout/pass/engine.rs zircon_runtime/src/ui/layout/pass/taffy_arrange.rs zircon_runtime/src/ui/tests/taffy_layout_diagnostics.rs zircon_runtime/src/ui/tests/mod.rs (2026-05-23: passed after Taffy tree-build report stats)
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/layout/pass/taffy_arrange.rs zircon_runtime/src/ui/tests/taffy_layout_pass.rs zircon_runtime/src/ui/tests/surface_frame_authority.rs (2026-05-23: passed after Flow slot linear sizing parity guard)
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/layout/engine.rs zircon_runtime_interface/src/tests/layout_engine_contracts.rs zircon_runtime/src/ui/layout/pass/taffy_arrange.rs zircon_runtime/src/ui/tests/taffy_layout_pass.rs zircon_runtime/src/ui/tests/surface_frame_authority.rs (2026-05-23: passed after AxisConstraintPriority fallback reason addition)
  - git diff --check -- zircon_runtime/src/ui/layout/pass/taffy_arrange.rs zircon_runtime/src/ui/tests/taffy_layout_pass.rs zircon_runtime/src/ui/tests/surface_frame_authority.rs docs/zircon_runtime/ui/layout/pass.md .codex/sessions/20260523-0907-layout-taffy-convergence.md (2026-05-23: passed with only CRLF conversion warnings)
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/layout/taffy_bridge.rs zircon_runtime/src/ui/tests/taffy_bridge.rs zircon_runtime/src/ui/layout/pass/taffy_arrange.rs zircon_runtime/src/ui/tests/taffy_layout_pass.rs zircon_runtime/src/ui/tests/surface_frame_authority.rs zircon_runtime_interface/src/ui/layout/engine.rs zircon_runtime_interface/src/tests/layout_engine_contracts.rs (2026-05-23: passed after InvalidLayoutValue style input guard)
  - git diff --check -- zircon_runtime/src/ui/layout/taffy_bridge.rs zircon_runtime/src/ui/tests/taffy_bridge.rs zircon_runtime/src/ui/layout/pass/taffy_arrange.rs zircon_runtime/src/ui/tests/taffy_layout_pass.rs zircon_runtime/src/ui/tests/surface_frame_authority.rs zircon_runtime_interface/src/ui/layout/engine.rs zircon_runtime_interface/src/tests/layout_engine_contracts.rs docs/zircon_runtime/ui/layout/pass.md .codex/sessions/20260523-0907-layout-taffy-convergence.md (2026-05-23: passed after InvalidLayoutValue style input guard, with only CRLF conversion warnings)
  - cargo test -p zircon_runtime --lib taffy_layout_pass --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never (2026-05-20: passed, 11 passed)
  - cargo test -p zircon_runtime --lib taffy_bridge_keeps_block_display_explicit_and_container_zircon_owned --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime_interface ui_layout_engine_block_is_explicit_not_implied_by_current_container_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface --message-format short --color never (2026-05-20: passed, 1 passed)
  - cargo test -p zircon_runtime template_tree_builder_parses_size_box_container_contract -- --nocapture (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_runtime_interface ui_layout_engine_request_maps_current_container_contracts_to_engine_families -- --nocapture (2026-05-11: passed, 1 passed)
doc_type: module-detail
---

# Runtime UI Layout Pass Slots

## Purpose

The runtime layout pass turns retained `.ui.toml` tree data into concrete frames, clip frames, scroll windows, and slot-driven child placement. This is the M1 shared Slate core layer for panel geometry: editor hosts and runtime renderers must consume the arranged `UiSurfaceFrame` that comes out of this pass instead of creating local coordinate tables.

Unreal Slate keeps panel-owned placement in slot objects and emits `FArrangedWidget` records during `OnArrangeChildren`. Zircon keeps the authored source as `.ui.toml`, preserves parent-owned slot policy in neutral DTOs, and then derives `UiArrangedTree`, render extract, and hit-grid entries from the same runtime layout cache.

## Slot Inventory

`UiSlotSchema` is component-level authoring metadata. It names slots such as `content`, `row`, or `page`, and records whether they are required or repeatable. It does not describe placement.

`UiSlot` is the runtime parent-child placement DTO. Current fields cover:

- `kind`: `Free`, `Container`, `Overlay`, `Linear`, `Grid`, `Flow`, `Canvas`, `Scrollable`, `Splitter`, `Scale`.
- `padding` and `alignment`: consumed by `Free`, `Container`, `Overlay`, `HorizontalBox`, `VerticalBox`, `WrapBox`/`FlowBox`, and `GridBox` layout through `layout/pass/slot.rs`, `axis.rs`, `measure.rs`, `arrange.rs`, and `child_frame.rs`.
- `linear_sizing`: consumed by linear panels and mapped into the Taffy flex path. `Auto` stays content/preferred sized, `Stretch` grows by slot value from a zero/min basis, and `StretchContent` grows from content/preferred basis. The Zircon axis solver also consumes the same slot sizing so fallback and Taffy paths keep the same main-axis intent for the supported growth cases.
- `canvas_placement`: preserved for Free/Canvas-like parents by template compilation, but `Free` arrangement still primarily consumes child `anchor`, `pivot`, and `position` unless a slot frame policy is active.
- `order`: consumed for stacked and linear child ordering, including wrap rows.
- `grid_placement`: consumed by `GridBox` for per-child row, column, row-span, and column-span placement. Missing values fall back to stable row-major placement. The Taffy path maps this metadata into explicit CSS grid lines; finite non-negative grid slot padding maps to child margin, and fixed-axis `Center`/`End` alignment maps to `justify_self`/`align_self` while unsupported stretch-axis alignment still falls back to Zircon.
- `z_order`: serialized and parsed for overlay slots, but not yet promoted into `UiTreeNode.z_index`; current arranged/render/hit z-order still comes from the node.
- `dirty_revision`: preserved as slot mutation metadata and not yet a standalone rebuild trigger.

`UiContainerKind` currently has runtime arrange support for `Free`, `Container`, `Overlay`, `Space`, `SizeBox`, `HorizontalBox`, `VerticalBox`, `ScrollableBox`, `WrapBox`, and `GridBox`. Template inference maps authored `FlowBox` and `FlexBox` to `WrapBox`, group aliases to the matching horizontal/vertical/grid containers, and `CanvasBox` to `Free`, so v2 component names can stay stable while the shared runtime keeps a compact container enum.

## Taffy Bridge

`zircon_runtime::ui::layout::taffy_bridge` converts only the Taffy-owned subset of `UiContainerKind` into `taffy::style::Style`: horizontal and vertical flex, grid, and wrap. The shared engine capability also exposes an explicit `Block` family and maps that family to Taffy `Display::Block`, but no current `UiContainerKind` produces a `Block` request. Plain `Container`, `Space`, and `SizeBox` remain Zircon-owned until a real block container contract exists. The bridge copies min/preferred/max constraints into Taffy size fields and maps panel gaps to Taffy gap values.

Overlay, scroll, virtualized list, popup-like, canvas/free, `SizeBox`, and editor docking semantics remain Zircon-owned. The bridge returns `None` for these families, and `UiLayoutEngineSelection` reports a Taffy-to-Zircon fallback with `ZirconOwnedSemantics` when Taffy is requested for those containers. Runtime `CanvasBox` authoring currently compiles to `UiContainerKind::Free`, so the route diagnostic for canvas-like parents is the `Free` family plus the separate `SlotCanvasPlacement` fallback for slot-authored canvas placement. `SizeBox` deliberately maps to the `Container` family but still requires Zircon semantics because its child frame is a contain-fit content rectangle, not a flex/grid track.

`layout/pass/engine.rs` records the backend decision for every arranged container that crosses the layout-engine boundary. Full and incremental layout both finish with a `UiLayoutEngineSelectionReport`; `UiSurface` stores it, `UiSurfaceFrame` carries it, and the debug snapshot mirrors it, including JSON export through `UiSurface::debug_snapshot_json(...)`. Reports count native Taffy selections, legacy Zircon selections, fallbacks, unsupported requests, node ids for each selection, `fallback_reason_counts` for a compact reason summary, and Taffy tree-build totals (`taffy_tree_build_count` / `taffy_tree_node_count`) derived from per-selection `UiLayoutEngineTaffyTreeBuildStats`. `fallback_reason_counts` also counts fallback/unsupported routes whose reason is missing as `reason=None`, so a silent fallback becomes visible in aggregate diagnostics instead of hiding behind the total fallback count. Editor/runtime diagnostics therefore no longer have to infer silent legacy fallback, scan the full selection list just to see which fallback reason dominated a frame, or guess how many transient Taffy trees the current non-cached pass built. Report deserialization recomputes every aggregate field from `selections`, so older or hand-edited surface debug snapshots that omit `fallback_reason_counts` or tree-build stats still recover the same route summary after import.

Incremental layout produces route selections only for visited dirty subtrees. `UiSurface::rebuild_dirty(...)` merges that partial report with the previous surface-level report by dropping selections for visited nodes and stale deleted nodes, then appending the new subtree selections. This keeps diagnostics surface-scoped even when a leaf under a non-auto parent is the only visited layout node, so unvisited container route decisions do not disappear from `UiSurfaceFrame`.

The report treats `Fallback` and `Unsupported` as separate states. Runtime containers currently route to either native Taffy or a known Zircon fallback, but the shared DTO can also represent a request that neither the preferred backend nor fallback backend supports. Interface tests cover this path without requiring a live runtime container that intentionally has no solver.

Editor Debug Reflector consumes the same report through a `Layout Engine` section. The section is intentionally read-only: it summarizes request counts, selected backend counts, Taffy transient tree-build totals, aggregated fallback-reason counts, and a bounded per-node routing/fallback preview including each route's Taffy tree-build/node counts, but does not recompute frames, hit entries, or render commands. Runtime Diagnostics carries those section lines through `RuntimeDiagnosticsPanePayload`, pane-template attributes, and retained-host generated text rows, so Taffy-vs-Zircon routing and the current non-cached Taffy tree-build cost are visible in the workbench instead of being limited to serialized snapshots.

`taffy_arrange.rs` is the first runtime pass integration point. `arrange.rs` asks it to solve `HorizontalBox`, `VerticalBox`, `WrapBox`, and `GridBox` before falling back to the legacy Zircon arrange code. The helper accepts template metadata because it carries render/event descriptors. It now accepts linear slot sizing (`Auto`, `Stretch`, and `StretchContent`) only for true `HorizontalBox`/`VerticalBox` linear slots, grid placement, finite non-negative slot padding, grid fixed-axis alignment, and flex/wrap cross-axis alignment where the child constraint can preserve Zircon semantics. `WrapBox`/`FlowBox` keeps the Zircon wrap contract: Flow slots can affect ordering, padding, and cross-axis alignment, but Flow slot `linear_sizing` is ignored instead of being treated as flex growth. It still rejects collapsed children, canvas placement, non-default child anchor/pivot/position, non-zero main-axis `AxisConstraint.priority`, non-finite parent frame or container style values, non-finite child constraint/desired-size values, non-finite consumed linear slot sizing values, negative or non-finite slot padding values, flex/wrap main-axis per-child `Center`/`End` alignment, and cross-axis `Center`/`End` alignment when the child extent is not fixed, all with explicit fallback reasons (`UnsupportedChildVisibility`, `ChildPlacementPolicy`, `AxisConstraintPriority`, `InvalidLayoutValue`, `SlotFramePolicy`, `SlotCanvasPlacement`). This lets v2 template-authored component subtrees take the Bevy-style Taffy path while retaining Zircon ownership for absolute placement and parent-specific frame policies.

The Taffy bridge preserves Zircon's explicit stretch semantics. A child with `StretchMode::Stretch` and an authored preferred extent participates in main-axis `flex_grow` using its constraint weight; on the cross axis it leaves size as `auto` so the parent's stretch alignment can fill the available extent while min/max constraints still clamp the result. Default content-driven children with measured desired size can remain content-sized unless their preserved stretch axis asks to fill.

MUI responsive layout runs as a pre-pass before both full and incremental measurement. `responsive_mui.rs` resolves authored Material UI breakpoint props against the current root logical width using the default MUI keys (`xs=0`, `sm=600`, `md=900`, `lg=1200`, `xl=1536`), including table-shaped values such as `{ xs = ..., md = ... }`, array values, and scalar fallbacks. The pre-pass first evaluates retained `UseMediaQuery` utility nodes against the same viewport and writes a runtime `matches` boolean into template metadata. Supported queries are intentionally narrow: `(min-width: Npx)`, `(max-width: Npx)`, and range strings that combine both constraints in one query (for example `(min-width: 600px) and (max-width: 959px)`), plus numeric `min_width` / `max_width` props and breakpoint shorthand props `up`, `down`, `between`, and `breakpoint`. `up` and `breakpoint` use inclusive lower bounds, while `down` and the upper half of `between` use the target breakpoint as an exclusive upper bound. Unsupported query strings fall back to `defaultMatches`, `default_matches`, authored `matches`, then `false`. The pre-pass then recomputes implicit `Grid` container columns/gaps, `Stack` direction/spacing, `Masonry` columns/gap, and child `Grid` item `size`/`offset` placement into the existing Zircon `UiContainerKind` and `UiSlot` DTOs. It also resolves responsive visibility metadata for authored `display`, `visibility`, and legacy `visible`: `display=none` maps to collapsed layout participation, while `visibility` and `visible` drive render/hit/input behavior through `UiVisibility` and `UiStateFlags::visible`. It deliberately preserves explicit `attributes.layout.container` and explicit `slot_attributes.layout` authoring, so low-level runtime layout remains the source of truth when a template opts out of MUI inference. Changed media-query matches, containers, slots, or visibility state are marked into layout/render/hit/input dirty domains before the normal solver runs, which lets resize and multi-resolution editor surfaces share the same Taffy/Zircon arrange path instead of creating a parallel MUI coordinate path.

## Behavior Model

The pass runs in two phases. `measure.rs` walks children first, computes desired content size, and includes slot padding for stacked, linear, and wrap containers. `arrange.rs` then writes each node's `layout_cache.frame`, `clip_frame`, `content_size`, and `virtual_window`.

Stacked panels (`Free`, `Container`, `Overlay`) use `free_child_frame(...)`. When a matching slot carries padding or alignment, the child is arranged inside the padded parent frame. Without a slot frame policy, the legacy node-owned anchor, pivot, and position fields remain the placement source.

`SizeBox` measures stacked child content and, when `aspect_ratio` is positive and finite, expands the desired content box to preserve that ratio. During arrange it computes a centered contain-fit content frame inside the parent frame; children are then placed through the normal container slot path inside that content frame. Invalid or zero ratios degrade to plain container behavior.

Linear panels order children by slot `order`, solve main-axis extents from constraints plus slot padding and `UiLinearSlotSizing`, and use slot alignment to place each child inside its allocated outer frame. Taffy-native linear panels map the same slot sizing into flex basis/grow/shrink: `Auto` keeps the desired-content basis without growing into leftover space, `Stretch` grows from the authored minimum basis, and `StretchContent` grows from the measured or preferred content basis. Slot `min`/`max` is mapped onto Taffy min/max size for the main axis, so a capped stretch child can clamp and leave remaining space for its sibling. Taffy does not model Zircon's staged priority solver, so a non-zero main-axis `AxisConstraint.priority` forces a Zircon fallback with `AxisConstraintPriority` instead of silently flattening priorities into flex weights. Non-finite parent frame, container gap, `AxisConstraint`, desired-size, or consumed linear slot sizing values force `InvalidLayoutValue` before the Taffy tree is built. They also map finite non-negative slot padding to child margin; flex/wrap cross-axis alignment can stay native when constraints make the aligned extent explicit. Flex main-axis `Center`/`End` remains a Zircon fallback because Taffy flex does not provide per-child main-axis self alignment. `WrapBox`/`FlowBox` reuses the linear child-frame logic per row after grouping children by available width, horizontal gap, vertical gap, item minimum width, and slot padding, but it does not consume `UiLinearSlotSizing`; the Taffy-native wrap path follows the same rule so Flow slot sizing metadata cannot create backend-specific growth.

`GridBox` divides the parent frame into configured rows and columns, subtracts row/column gaps once, and places children from `UiGridSlotPlacement`. Span values expand the outer cell frame before normal slot padding/alignment is applied, so render extraction and hit testing see the same child frame that layout measured. Taffy-native grid panels use the same zero-based placement metadata to emit one-based CSS grid line starts/ends, expand the template rows/columns to cover explicit spans, map finite non-negative slot padding to margin, and map fixed-size slot alignment to `justify_self`/`align_self`; non-fixed `Center`/`End` grid alignment remains a `SlotFramePolicy` fallback.

`ScrollableBox` computes content extent, clamps scroll offset, records `UiScrollState`, and stores `UiVirtualListWindow` when virtualization is configured. Off-window children are hidden from hit testing by zeroing layout frames through `hide_subtree_layout(...)`; visible children keep frames and clips that feed the surface frame.

`UiSurfaceFrame` remains the single spatial authority regardless of backend. Taffy and Zircon both write the same retained tree layout cache, then `build_arranged_tree`, render extraction, hit-grid rebuild, pointer routing, and debug reflection consume that output. The layout-engine report is diagnostic metadata attached to this same frame, not a second coordinate source.

Viewport changes feed MUI breakpoint props through the same frame authority. `apply_mui_responsive_layout(...)` runs before `measure.rs`, so a `Grid` whose columns or gaps change at `md`, a `Stack` whose direction flips between column and row, and a `Masonry` whose column count changes all arrive at measurement as ordinary runtime containers. Grid item `size` and `offset` props become ordinary grid slot placement, then the existing Taffy grid bridge and Zircon grid fallback consume that placement exactly like explicit layout authoring.

`UseMediaQuery` is treated as a behavior utility instead of a visual widget. Its `matches` value is recomputed before visibility, container, and slot resolution, so editor assets can bind viewport-dependent state through ordinary retained attributes while keeping runtime layout as the single source of truth.

Runtime layout invalidation uses structured dirty domains. `mark_layout_dirty(...)` bubbles layout invalidation through content-driven or auto-layout ancestors, marking layout, hit-test, and render dirty on affected nodes without setting the legacy `state_flags.dirty` compatibility bit. This keeps `UiSurface::dirty_flags()` diagnostics precise and avoids reporting input dirtiness for pure layout changes.

## Shared Frame Contract

`zircon_runtime/src/ui/tests/layout_slots.rs` now covers four M1.3 focused authority cases:

- `overlay_slot_geometry_feeds_arranged_render_hit_and_z_order_from_one_surface_frame` arranges overlapping overlay children with slot padding/alignment and node z-index, then proves `UiSurfaceFrame.arranged_tree`, `render_extract`, `hit_grid`, and `hit_test_surface_frame(...)` agree on frame, clip, z-order, stacked hit order, and bubble route.
- `scrollable_virtual_window_uses_visible_arranged_child_for_render_and_hit_entries` arranges a virtualized scroller at an offset, verifies the visible window, and proves the visible arranged child is the same frame consumed by render and hit-grid entries while off-window children do not enter hit testing.
- `wrap_flow_slot_padding_alignment_feeds_shared_surface_frame` proves `WrapBox`/flow slot order, padding, and alignment feed arranged, render, and hit evidence from one frame.
- `grid_slot_cell_placement_feeds_arranged_render_hit_from_one_surface_frame` proves configured grid rows/columns/gaps and per-child grid placement feed arranged, render, and hit evidence from one frame.

`zircon_runtime/src/ui/tests/template_grid_flow.rs` adds the matching template compile contract. It proves authored `GridBox` and `FlowBox` nodes produce the expected runtime container config, child slot kinds, grid placement/span metadata, and flow ordering before the layout pass runs.

`zircon_runtime/src/ui/tests/taffy_layout_pass.rs` now covers the runtime Taffy route contract. It proves simple flex/wrap/grid containers select Taffy natively, free/canvas-like, generic container, space, overlay, scrollable, virtual-list, and `SizeBox` containers report Zircon-owned fallback, flex main-axis alignment, non-fixed cross-axis alignment, GridBox non-fixed alignment, negative/non-finite slot padding values, collapsed children, non-default child anchor/position, non-zero main-axis constraint priority, non-finite container config, non-finite axis constraints, non-finite consumed linear slot sizing, and canvas placement report explicit fallback reasons, grid placement/span plus out-of-bounds track expansion and `Auto`/`Stretch`/`StretchContent` linear slot sizing with min/max bounds can remain on the Taffy-native path, linear slot padding plus fixed-size cross-axis alignment can stay Taffy-native for both horizontal and vertical boxes, wrap slot padding plus fixed-size cross-axis alignment can stay Taffy-native, Flow slot `linear_sizing` is ignored by WrapBox without forcing a Zircon fallback, grid slot padding/alignment can stay Taffy-native for fixed-size cells, and measured text/image desired sizes from the measure pass feed Taffy-native child frames. `zircon_runtime/src/ui/tests/taffy_layout_diagnostics.rs` keeps the new tree-build metric coverage out of the already-large route file and proves a two-child Taffy-native flex pass reports one transient tree and three Taffy nodes. The route tests also guard the current Block boundary: runtime arrange routing must not claim `UiLayoutEngineFamily::Block`, `Display::Block`, or generic `Container` native arrange support until a real Block container contract exists.

`zircon_runtime/src/ui/tests/surface_frame_authority.rs` now includes backend-specific authority coverage beyond the original overlapping free-frame case. It proves Taffy-native flex, horizontal and vertical linear slot sizing, wrap, and grid containers write the expected frames into `UiSurfaceFrame`; it proves a Taffy-eligible flex container that falls back for `SlotFramePolicy` still writes its Zircon-arranged frame into the same surface authority; and it proves a Zircon-owned `SizeBox` fallback writes its contain-fit slot-aligned frame into the same authority surface. In each case render extract, hit-grid entries, frame hit testing, and pointer dispatch consume that same arranged frame instead of rebuilding local geometry.

`zircon_runtime/src/ui/tests/diagnostics.rs` covers JSON/debug snapshot export for both sides of the route report. The Taffy-native snapshot test proves `layout_engine_report` survives JSON roundtrip with a native Taffy backend; the Zircon fallback snapshot test proves `legacy_zircon`, `fallback`, and `zircon_owned_semantics` are serialized and deserialize back to the same `UiLayoutEngineSelection` for a `SizeBox` root.

These tests are intentionally runtime-only. They do not touch Material visual templates, editor host rectangles, or native painter code.

## Visual Verification Gate

The runtime Taffy route tests are not sufficient for visual acceptance. Every layout/Taffy closeout that changes routing, slot sizing, fallback behavior, or frame authority must also run the editor profile capture path so the result is visually inspectable through screenshots and live diagnostics:

```powershell
tools/ui-profile-capture.ps1 -ScenarioList material_lab_startup,material_lab_hover,material_lab_click,drawer_resize -AutoInteract -RequireScenarioEvidence -CaptureSoftbufferScreenshot -AutoCloseSeconds 4 -SkipBuild
```

Accepted artifacts are the generated profile directory's `screenshot_gpu.png`, `screenshot_softbuffer.png`, `screenshot_diff.json`, `ui_hotspots.json`, and `summary.md`. The visual check must inspect Material Lab startup/hover/click plus drawer resize for no overlapping or clipped rows, stable spacing after resize, visible hover/click state changes, nonblank GPU and softbuffer screenshots, and no unexpected software fallback alerts. Runtime Diagnostics must also expose the read-only `Layout Engine` section with representative `selected=Taffy` native routes and `selected=LegacyZircon` fallback routes, so the screenshot can be correlated with the same Taffy-vs-Zircon routing that unit tests assert.

If `-CaptureSoftbufferScreenshot` is not available in the local run, record that limitation explicitly and still keep the GPU screenshot plus `ui_hotspots.json` evidence. A code-only or JSON-only route report is no longer enough to close a layout/Taffy visual change.

## Accepted Follow-Ups

M1 accepts the shared panel authority for the current retained-tree model. These items remain explicit follow-ups rather than hidden M1 gaps:

- Overlay slot `z_order` should either be promoted into arranged z-order or removed from the runtime DTO if node `z_index` remains the only z authority.
- `canvas_placement` needs a runtime cutover decision for Free/Canvas panels so parent-owned placement can replace child-owned anchor/pivot/position where intended. Until then, Canvas-like parent routing is visible as `UiLayoutEngineFamily::Free` with Zircon-owned fallback, and slot-authored canvas placement stays an explicit `SlotCanvasPlacement` fallback.
- Scroll virtualization currently keeps hidden children in the retained tree and arranged tree with zeroed frames; M1 accepts the hit-grid boundary, but later render/performance work should decide whether zero-area render commands should be suppressed.
- A true `BlockBox` or equivalent container should be introduced before claiming runtime Block arrangement. The interface can already represent an explicit `Block` request for backend selection, but the current retained tree deliberately does not reinterpret generic `Container` as block layout.
- Parser/import validation should eventually reject or sanitize non-finite authored layout scalars before they reach the retained tree. Runtime Taffy eligibility now guards the values it consumes, but earlier validation would make authoring errors easier to report at source.

## Test Coverage

Formatting passed for the changed runtime-interface layout DTOs, runtime layout pass files, template parser/build files, and focused tests.

Focused runtime validation passed with `7 passed; 0 failed; 920 filtered out` for:

```powershell
cargo test -p zircon_runtime --lib layout_slots --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
```

Template Grid/Flow validation passed with `1 passed; 0 failed; 926 filtered out` for:

```powershell
cargo test -p zircon_runtime --lib template_grid_flow --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
```

The first cold run exceeded the tool timeout while compiling on a machine with concurrent sibling Cargo validations. Warmed focused reruns above completed and are the accepted evidence for this slice. Existing `zircon_runtime` warning noise remains.

The v2 Taffy cutover guard passed with `6 passed; 0 failed; 1251 filtered out` for:

```powershell
cargo test -p zircon_runtime taffy -- --nocapture
```

That run includes `taffy_layout_pass_accepts_template_metadata_from_v2_assets`, proving v2-authored nodes no longer fall back merely because they carry template metadata. It also includes `size_box_contain_aspect_ratio_stays_zircon_owned`, proving `SizeBox` uses Zircon contain-fit semantics and remains outside the Taffy-owned arrange path.

The 2026-05-20 layout-engine report slice passed by running the already-built focused runtime test harness after the Cargo invocation completed:

```powershell
D:\cargo-targets\zircon-layout-impl\debug\deps\zircon_runtime-eb36ca4a90c1b648.exe taffy_layout_pass --nocapture
D:\cargo-targets\zircon-layout-impl\debug\deps\zircon_runtime-eb36ca4a90c1b648.exe surface_frame_authority --nocapture
```

The first command passed `7 passed; 0 failed; 1657 filtered out`, covering backend reports, explicit fallback reasons, grid placement, and linear slot sizing. The second passed `1 passed; 0 failed; 1663 filtered out`, preserving the arranged/render/hit/pointer authority contract.

The backend authority continuation adds `taffy_native_flex_surface_frame_feeds_render_hit_and_pointer_dispatch`, `taffy_flex_linear_slot_sizing_feeds_render_hit_and_pointer_dispatch`, `taffy_vertical_flex_linear_slot_sizing_feeds_render_hit_and_pointer_dispatch`, `taffy_flex_slot_policy_fallback_feeds_render_hit_and_pointer_dispatch`, `taffy_wrap_surface_frame_feeds_render_hit_and_pointer_dispatch`, `taffy_grid_slot_frame_policy_feeds_render_hit_and_pointer_dispatch`, and `zircon_size_box_fallback_feeds_render_hit_and_pointer_dispatch` to `surface_frame_authority.rs`. The Taffy tests prove native flex, horizontal and vertical linear slot sizing, wrap, and grid-slot frames are the same frames seen by arranged tree, render extract, hit grid, surface hit testing, and pointer dispatch while the layout-engine report marks the root container as native Taffy. The horizontal linear slot sizing case uses a 2:1 stretch split in a 180px parent, requiring `UiFrame::new(0.0, 0.0, 120.0, 40.0)` and `UiFrame::new(120.0, 0.0, 60.0, 40.0)` to feed render, hit, and pointer routing. The vertical case uses the same 2:1 split in a 180px-tall parent, requiring `UiFrame::new(0.0, 0.0, 60.0, 120.0)` and `UiFrame::new(0.0, 120.0, 60.0, 60.0)` to feed the same authorities. The flex slot-policy fallback test proves a Taffy-eligible `HorizontalBox` can report `Flex` + `SlotFramePolicy` + `LegacyZircon` while still feeding `UiFrame::new(10.0, 19.0, 40.0, 16.0)` through the same arranged/render/hit/pointer authority. The Zircon-owned fallback test proves `SizeBox` reports `Container` + `ZirconOwnedSemantics`, keeps its contain-fit slot-aligned frame, and still feeds the same downstream authorities. The grid-slot focused command passed on 2026-05-21 with `1 passed; 0 failed; 1750 filtered out` after a `20m 42s` cold build in `D:\cargo-targets\zircon-layout-authority-20260521`. The SizeBox fallback focused command passed on 2026-05-21 with `1 passed; 0 failed; 1760 filtered out` after a `4m 48s` warmed rebuild in the same target dir. The WrapBox focused command passed on 2026-05-22 with `1 passed; 0 failed; 1855 filtered out`; the flex slot-policy fallback focused command passed on 2026-05-22 with `1 passed; 0 failed; 1862 filtered out`; the horizontal linear slot sizing focused command passed on 2026-05-22 with `1 passed; 0 failed; 1880 filtered out`; the vertical linear slot sizing focused command passed on 2026-05-23 with `1 passed; 0 failed; 1900 filtered out`; and the warmed module rerun passed `8 passed; 0 failed; 1894 filtered out` in `D:\cargo-targets\zircon-layout-wrap-authority-20260522`, covering the original free-frame authority case plus Taffy flex, Taffy horizontal/vertical linear slot sizing, Taffy flex fallback, Taffy wrap, Taffy grid, and Zircon SizeBox fallback. The 2026-05-22 and 2026-05-23 runs used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`. These runs reported existing `zircon_runtime` warning noise.

The incremental route-report continuation adds `surface_dirty_layout_preserves_unvisited_layout_engine_routes`, `surface_dirty_layout_replaces_visited_layout_engine_routes`, and `surface_dirty_layout_drops_removed_layout_engine_routes` to `surface_dirty_domains.rs`. The first test starts from a `Free` root whose route report is a Zircon-owned fallback, then marks only a leaf child layout-dirty under a non-auto parent. The accepted rebuild visits one node, skips two, preserves the root fallback route in `UiSurface.layout_engine_report`, and confirms `UiSurfaceFrame.layout_engine_report` carries the same surface-level report. The replacement test mutates a visited child container from a reported `Flex` route that can be a LegacyZircon `AxisConstraintPriority` fallback into Zircon-owned `Overlay`, requiring the old flex selection to disappear and exactly one overlay fallback selection to remain for that node while the unvisited root fallback stays present. The removal test detaches the previously reported flex subtree and requires the merged report to drop the stale node selection while retaining the root fallback. These incremental tests also verify the merged report is exported through `UiSurfaceFrame`, `UiSurface::debug_snapshot()`, and `UiSurface::debug_snapshot_json(...)` roundtrip, so diagnostics and JSON export do not regress to a partial subtree report. The 2026-05-22 continuation also asserts `fallback_reason_counts` during preserve, replace, and stale-route removal: the preserved root fallback stays at `ZirconOwnedSemantics=1`, replacement recomputes the merged report to `ZirconOwnedSemantics=2`, and removal drops back to `ZirconOwnedSemantics=1`. The 2026-05-27 M5 runtime-gate rerun keeps the preserve case on root-only `ZirconOwnedSemantics=1`, records the initial visited child fallback as `AxisConstraintPriority=1`, then verifies replacement rises to `ZirconOwnedSemantics=2` and removal returns to `ZirconOwnedSemantics=1`. The focused command passed on 2026-05-21 with `1 passed; 0 failed; 1797 filtered out` after a `17m 07s` cold build in `D:\cargo-targets\zircon-layout-incremental-routes-20260521`. The broader `surface_dirty_layout` rerun passed `6 passed; 0 failed; 1794 filtered out` after a `3m 15s` rebuild in the same target dir, covering the existing skip/revisit/marking dirty-domain cases plus route preservation, replacement, and stale-route removal. The debug-export rerun passed the same `6 passed; 0 failed; 1794 filtered out` after a `2m 06s` rebuild in the same target dir. The 2026-05-22 reason-count rerun passed `6 passed; 0 failed; 1849 filtered out` after a cold build in `D:\cargo-targets\zircon-layout-incremental-reason-counts-20260522`. The 2026-05-27 exact test-binary rerun passed `surface_dirty_layout_replaces_visited_layout_engine_routes` and `surface_dirty_layout_drops_removed_layout_engine_routes`, followed by `cargo test -p zircon_runtime --lib --locked --target-dir F:\cargo-targets\zircon-platform-m5-workspace --message-format short --color never -- --format terse` passing with 2102 passed and 0 failed. These accepted runs reported existing `zircon_runtime` warning noise; the 2026-05-22 rerun used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`.

The runtime fixture route-report continuation adds `runtime_quest_log_fixture_exports_layout_engine_route_report` and `runtime_inventory_fixture_reports_virtualized_list_zircon_fallback`. The quest log test loads the real `runtime.ui.quest_log_dialog` fixture through `RuntimeUiManager`, computes layout, and verifies that the authored `VerticalBox` dialog and `HorizontalBox` actions route to native Taffy while the overlay root remains a Zircon fallback with `ZirconOwnedSemantics`. The inventory test loads `runtime.ui.inventory_list` and verifies that the real virtualized `ScrollableBox` reports `UiLayoutEngineFamily::VirtualizedList` with `LegacyZircon`, `Fallback`, and `ZirconOwnedSemantics`. Both tests assert the report is identical on `UiSurface`, `UiSurfaceFrame`, the debug snapshot, and a JSON-exported debug snapshot roundtrip, so runtime `.v2.ui.toml` assets do not get a separate diagnostic path. They also recompute route counts from `selections`, require every route to identify its node, require native routes to be Taffy without fallback reasons, and require every LegacyZircon fallback or unsupported route to carry a diagnostic reason. This mirrors the Editor host no-silent-fallback contract for real runtime fixture assets. The runtime fixture authority continuation now asserts the same arranged frames feed render command frame/clip/z-index for `QuestLogDialog`, `QuestLogActions`, `InventoryList`, and the first visible virtualized inventory row. It also dispatches pointer events through `RuntimeUiManager` for `TrackQuestButton`, `CloseQuestLogButton`, and `InventoryRow00`, requiring matching hit-grid entries, identical `surface.hit_test(...)` and `hit_test_surface_frame(...)` results, and a successful `UiPointerDispatcher` route through the same hit path. The same tests then call `RuntimeUiManager::build_frame()` and require the public runtime frame `ui` extract to equal both `UiSurfaceFrame.render_extract` and `UiSurface.render_extract`, with non-empty render commands, so the runtime submission boundary consumes the same surface output. The first quest-log focused command passed `1 passed; 0 failed; 1791 filtered out` after a `26m 50s` cold build in `D:\cargo-targets\zircon-layout-runtime-fixture-20260521`. The JSON roundtrip rerun passed `1 passed; 0 failed; 1793 filtered out` after a `5m 19s` warmed rebuild. After the inventory test was added, the broader `runtime_` filter passed `282 passed; 0 failed; 1515 filtered out` after a `3m 58s` warmed rebuild, and the exact inventory and quest-log focused reruns each passed `1 passed; 0 failed; 1796 filtered out` in the same target dir. The final no-silent-fallback rerun passed `282 passed; 0 failed; 1515 filtered out` after a `4m 02s` rebuild in the same target dir. The authority-focused module rerun passed `2 passed; 0 failed; 1795 filtered out` after a `3m 37s` rebuild in the same target dir. The public-frame authority rerun passed `2 passed; 0 failed; 1795 filtered out` after a `3m 06s` rebuild in the same target dir. These runs reported existing `zircon_runtime` warning noise.

The Zircon fallback diagnostics continuation passed with:

```powershell
cargo test -p zircon_runtime --lib surface_debug_snapshot_json_exports_zircon_fallback_route_reason --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-diagnostics-20260521 --message-format short --color never
```

The final rerun passed `1 passed; 0 failed; 1764 filtered out` after a `3m 06s` warmed rebuild. An earlier run failed because the test asserted compact JSON while `UiSurface::debug_snapshot_json(...)` intentionally uses pretty JSON; the assertion now matches the exported pretty JSON and still deserializes the snapshot to verify the structured `LegacyZircon`/`Fallback`/`ZirconOwnedSemantics` selection. The passing run used `--locked` and reported existing `zircon_runtime` warning noise.

The 2026-05-22 diagnostics JSON rerun broadened the same export checks to the aggregate `fallback_reason_counts` field:

```powershell
cargo test -p zircon_runtime --lib surface_debug_snapshot_json_exports --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-diagnostics-20260522 --message-format short --color never
```

The focused run passed `2 passed; 0 failed; 1852 filtered out`. It verifies that a Taffy-native route serializes an empty `fallback_reason_counts` list and a Zircon fallback route serializes and round-trips `ZirconOwnedSemantics=1`. The run used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`.

The unsupported route-report contract is covered by:

```powershell
cargo test -p zircon_runtime_interface ui_layout_engine_selection_report_counts_unsupported_routes_separately --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface-unsupported-20260521 --message-format short --color never
```

This test constructs explicit empty preferred/fallback capabilities and verifies that a `Block` request records `UnsupportedFamily`, `UiLayoutEngineSupport::Unsupported`, and `unsupported_count=1` while leaving `fallback_count=0`. It is an interface DTO contract, not a claim that any current runtime container is intentionally unsupported.

The focused run passed `1 passed; 0 failed; 102 filtered out` after a `2m 35s` cold compile in `D:\cargo-targets\zircon-layout-interface-unsupported-20260521`.

The runtime-interface DTO and debug snapshot contracts passed with:

```powershell
cargo test -p zircon_runtime_interface layout_engine_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface --message-format short --color never
cargo test -p zircon_runtime_interface ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface-contract --message-format short --color never
```

Those runs passed `3 passed; 0 failed; 97 filtered out` and `1 passed; 0 failed; 99 filtered out`.

The 2026-05-22 fallback-reason count continuation extends `UiLayoutEngineSelectionReport` with `fallback_reason_counts`, recomputed from each route's `fallback_reason` and serialized beside the existing aggregate counts. Interface tests now cover both repeated `ZirconOwnedSemantics` fallbacks and a single `UnsupportedFamily` route. Runtime fixture and Editor host route tests recompute the same grouped counts from `selections` and compare them with the report field, so the public DTO cannot drift from per-node route details. A later interface continuation makes deserialization recompute all aggregate fields from `selections` as well; stale JSON aggregate values are ignored, and legacy snapshots that omit `fallback_reason_counts` recover the grouped reason summary from the per-node selections.

```powershell
cargo test -p zircon_runtime_interface layout_engine_contracts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-reason-counts-interface-20260522 --message-format short --color never
cargo test -p zircon_runtime --lib runtime_ui_layout_routes --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-runtime-fixture-20260521 --message-format short --color never
cargo test -p zircon_editor --lib builtin_editor_host_templates_export_layout_engine_route_reports --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-host-routes-20260521 --message-format short --color never
```

Those accepted reruns passed `5 passed; 0 failed; 98 filtered out`, `2 passed; 0 failed; 1852 filtered out`, and `1 passed; 0 failed; 1421 filtered out`. A `--locked --offline` interface attempt failed before compilation because the repository lockfile currently has unrelated sound/cpal dependency drift; the accepted offline runs temporarily allowed Cargo to update the lockfile, restored `Cargo.lock` from backup immediately afterward, and left the lockfile clean.

The deserialization continuation passed `cargo test -p zircon_runtime_interface layout_engine_contracts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-report-deser-20260522 --message-format short --color never` with `6 passed; 0 failed; 98 filtered out`. It feeds `UiLayoutEngineSelectionReport` JSON with deliberately stale aggregate counts and with legacy-missing `fallback_reason_counts`, then verifies the deserialized report recomputes request/backend/support/reason totals from `selections`.

The editor reflector projection for the same report passed with:

```powershell
cargo test -p zircon_editor --lib ui_debug_reflector_model_projects_snapshot_rows_and_sections --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor --message-format short --color never
cargo test -p zircon_editor --lib ui_debug_reflector_model_displays_unsupported_layout_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never
```

The native/fallback projection rerun passed `1 passed; 0 failed; 1416 filtered out` after a `6.30s` warmed rebuild and reported existing `zircon_runtime`/`zircon_editor` warning noise. The unsupported-route continuation passed `1 passed; 0 failed; 1416 filtered out` after a `4m 03s` warmed rebuild; it constructs a report where neither preferred nor fallback capability supports `Block` and requires the Editor `Layout Engine` section to show `fallbacks: 0 unsupported: 1`, `support=Unsupported`, and `reason=UnsupportedFamily`.

The 2026-05-22 Editor reflector rerun passed:

```powershell
cargo test -p zircon_editor --lib ui_debug_reflector_model --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-editor-route-payload-20260521 --message-format short --color never
```

That run passed `6 passed; 0 failed; 1416 filtered out` and proves the `Layout Engine` section now displays the grouped fallback-reason summary (`fallback reasons: ...`) in addition to the bounded per-node route preview. The first attempt timed out during rebuild and left a temporary lockfile update; the accepted rerun restored `Cargo.lock` from backup afterward.

The Runtime Diagnostics display continuation passed the focused payload, template-attribute, payload-host, and live-body refresh tests in the header. The first rerun passed after a `12m 23s` rebuild; warmed follow-up tests completed in seconds. These prove the runtime-owned layout-engine report reaches the active snapshot payload, template root attributes, retained-host generated rows, and live body-surface refresh path.

The editor pane payload route-detail continuation tightens `runtime_diagnostics_payload_uses_active_ui_debug_snapshot_when_available` so the payload must include the exact per-node fallback line for a Zircon-owned route (`node=2`, `family=Overlay`, `selected=LegacyZircon`, `reason=ZirconOwnedSemantics`). This proves the workbench-facing `RuntimeDiagnosticsPanePayload` preserves fallback reasons, not only aggregate Taffy/Zircon counts.

The focused editor rerun passed `1 passed; 0 failed; 1415 filtered out` after a `3m 41s` warmed rebuild in `D:\cargo-targets\zircon-layout-editor-route-payload-20260521`. The earlier cold compile exceeded the tool timeout while still building, so only the completed warmed rerun is accepted as evidence.

The retained-host and template-runtime continuation extends the same route-detail requirement beyond the pane payload object. `runtime_diagnostics_host_conversion_keeps_payload_reflector_text_and_overlay` now requires the generated host rows to include the `Overlay`/`LegacyZircon`/`ZirconOwnedSemantics` line, `runtime_diagnostics_body_refresh_preserves_active_payload_reflector` proves an active payload reflector is not replaced by a live body-surface refresh, and `editor_ui_host_runtime_projects_pane_body_payload_metadata_into_root_attributes` proves the pane-body template attributes retain the same fallback reason line. The focused reruns passed `1 passed; 0 failed; 1415 filtered out` each; the first rebuilt in `3m 55s`, and the warmed follow-ups completed in `9.07s` and `4.71s`.

The editor host template route-report continuation adds `builtin_editor_host_templates_export_layout_engine_route_reports`. It loads the real built-in workbench shell, drawer source, floating-window source, and scene viewport toolbar documents through `EditorUiHostRuntime`, builds shared surfaces, computes layout, and checks their engine selections. Workbench flex scaffolding routes to native Taffy while overlay shell nodes remain explicit `LegacyZircon` fallbacks with `ZirconOwnedSemantics`; drawer, floating-window, and viewport-toolbar flex roots also route to native Taffy with no unsupported selections. The same test asserts the report is identical on `UiSurface`, `UiSurfaceFrame`, the debug snapshot, and the JSON debug snapshot roundtrip. It also recomputes the report counts from `selections`, requires every route to identify its node, requires native routes to be Taffy without fallback reasons, and requires every LegacyZircon fallback to carry an explicit diagnostic reason, so Editor host templates cannot regress to silent fallback. It also asserts key arranged frames from the same `UiSurfaceFrame`: workbench document host and pane surface, drawer top/body/status bands, floating-window top/center/document/status bands, and viewport-toolbar visible command controls. Those frame assertions require matching render command frame, clip, and z-index. Interactive controls (`OpenProject`, `FrameSelection`, and `SetProjectionMode`) additionally require matching hit-grid entries, identical `surface.hit_test(...)` and `hit_test_surface_frame(...)` results, and successful `UiPointerDispatcher` routing through the same bubble path. One intermediate assertion incorrectly reused the older host-model left-group width for the shared toolbar surface; the accepted test keeps the visible command frames and right group as the no-drift contract. The final focused rerun passed `1 passed; 0 failed; 1417 filtered out` after a `2m 41s` warmed rebuild in `D:\cargo-targets\zircon-layout-editor-host-routes-20260521`; it reported existing `zircon_runtime` and `zircon_editor` warning noise.

The flex/wrap/grid route tightening passed with:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_arranges_linear_wrap_and_grid_containers --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never
```

That test now proves the same pass that validates child geometry also records native Taffy selections for `Flex`, `Wrap`, and `Grid` root families.

The Zircon-owned route diagnostic continuation passed with:

```powershell
cargo test -p zircon_runtime --lib layout_pass_reports_taffy_native_and_zircon_fallback_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-route-20260521 --message-format short --color never
cargo test -p zircon_runtime --lib size_box_contain_aspect_ratio_stays_zircon_owned --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-route-20260521 --message-format short --color never
```

The first command passed `1 passed; 0 failed; 1757 filtered out` after a `3m 40s` warmed rebuild and covers `Free`, generic `Container`, and child-bearing `Space` Zircon-owned fallback alongside overlay, scrollable, virtual list, slot frame policy, and slot canvas placement fallback reasons. An intermediate extension failed because childless `Space` containers are intentionally omitted from the route report; the test now gives `Space` a child before asserting the fallback selection. The second command passed `1 passed; 0 failed; 1759 filtered out` after a `4m 06s` warmed rebuild and proves `SizeBox` keeps its contain-fit frame while reporting the `Container` family with `ZirconOwnedSemantics` fallback. Both passing runs used `--locked`; existing `zircon_runtime` warning noise remains.

The 2026-05-22 layout-pass reason-count continuation passed:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_aggregates_fallback_reason_counts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-taffy-reason-counts-20260522 --message-format short --color never
```

That focused run passed `1 passed; 0 failed; 1854 filtered out`. It computes one real surface with a Taffy-native horizontal parent and two child-bearing Zircon-owned child containers (`Overlay` and `SizeBox`), then verifies the route report keeps the native root selection plus a grouped `ZirconOwnedSemantics=2` fallback reason count. This closes the layout-pass-level aggregation path separately from the interface DTO and diagnostics JSON checks. The run used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`.

The distinct fallback-reason aggregation continuation passed with:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_aggregates_distinct_fallback_reason_counts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
```

That focused run passed `1 passed; 0 failed; 1859 filtered out`. It builds one child-bearing `Free` root plus child containers that force `ZirconOwnedSemantics`, `UnsupportedChildVisibility`, `ChildPlacementPolicy`, `SlotFramePolicy`, and `SlotCanvasPlacement` into the same `UiLayoutEngineSelectionReport`, then verifies the grouped counts are recomputed as `ZirconOwnedSemantics=2` and each other fallback reason equals `1`. This proves runtime aggregation handles mixed fallback causes, not only repeated same-reason fallback. The run used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`, and it reported existing runtime warning noise.

The 2026-05-22 linear slot sizing continuation passed:

```powershell
cargo test -p zircon_runtime --lib slot_sizing --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
cargo test -p zircon_runtime --lib linear_slot_sizing_bounds --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
```

The slot-sizing focused run passed `3 passed; 0 failed; 1876 filtered out`. It covers the existing `Stretch` route plus new measured-content cases for `Auto` and `StretchContent`. The `Auto` case keeps `Hello` and `Go` labels at their measured 25px and 10px widths inside a wider 100px flex parent, proving the Taffy path does not grow auto-sized children into leftover space. The `StretchContent` case uses the same measured bases inside a 95px parent and requires final frames of 55px and 40px, proving content basis plus equal grow share is preserved while the root remains native Taffy. The bounds focused run passed `1 passed; 0 failed; 1879 filtered out`; it gives one stretch child `min=80`/`max=90` inside a 200px parent and requires final frames of 90px and 110px, proving the Taffy-native path clamps the bounded child and redistributes the remaining space. The runs used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`, and they reported existing runtime warning noise.

The 2026-05-23 vertical linear slot sizing continuation passed:

```powershell
cargo test -p zircon_runtime --lib vertical_linear_slot_sizing --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
cargo test -p zircon_runtime --lib taffy_layout_pass_maps_vertical_linear_slot_sizing_bounds_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
cargo test -p zircon_runtime --lib taffy_layout_pass --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
```

The vertical sizing focused run passed `1 passed; 0 failed; 1889 filtered out`. It builds a `VerticalBox` with two `Stretch` linear slots weighted 2:1 inside a 300px-tall parent and requires child frames of `UiFrame::new(0.0, 0.0, 60.0, 200.0)` and `UiFrame::new(0.0, 200.0, 60.0, 100.0)`, proving Zircon linear slot sizing maps onto the vertical main-axis height while the route remains native Taffy. The vertical bounds focused run passed `1 passed; 0 failed; 1897 filtered out`; it gives one vertical stretch child `min=80`/`max=90` inside a 200px-tall parent and requires final frames of 90px and 110px, proving vertical height bounds clamp and redistribute like the horizontal slot-sizing path. The warmed module rerun passed `29 passed; 0 failed; 1869 filtered out`. The runs used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`, and they reported existing runtime warning noise.

The 2026-05-22 linear slot style continuation passed:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_maps_linear_slot_padding_and_cross_axis_alignment_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
cargo test -p zircon_runtime --lib taffy_layout_pass_maps_vertical_linear_slot_padding_and_cross_axis_alignment_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
```

The horizontal focused run passed `1 passed; 0 failed; 1860 filtered out`, and the vertical focused run passed `1 passed; 0 failed; 1861 filtered out`. The horizontal case builds a `HorizontalBox` with one fixed-size child whose linear slot carries finite padding plus vertical `End` alignment, producing `UiFrame::new(5.0, 17.0, 20.0, 10.0)`. The vertical case mirrors that contract through a `VerticalBox` with horizontal `End` alignment, producing `UiFrame::new(53.0, 2.0, 20.0, 10.0)`. Both roots remain native Taffy, proving linear `align_self` coverage matches the wrap/grid slot-style route instead of silently falling back to Zircon. The runs used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`, and they reported existing runtime warning noise.

The grid span continuation passed with:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_maps_grid_slot_span_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
```

That focused run passed `1 passed; 0 failed; 1863 filtered out`. It builds a `GridBox` with three columns, two rows, explicit gaps, and a child placed at column 1 / row 0 spanning two columns and two rows. The spanned child receives `UiFrame::new(54.0, 0.0, 102.0, 64.0)` and the root remains native Taffy, proving the zero-based Zircon span metadata maps into one-based Taffy CSS grid line starts/ends without falling back. The run used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`, and it reported existing runtime warning noise.

The out-of-bounds grid span continuation passed with:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_expands_grid_tracks_for_out_of_bounds_slot_span_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
```

That focused run passed `1 passed; 0 failed; 1864 filtered out`. It starts from a 1x1 `GridBox`, places a child at column 1 / row 1 with a 2x2 span, and proves the Taffy path expands template columns and rows before solving. The child receives `UiFrame::new(54.0, 35.0, 102.0, 65.0)` while the root stays native Taffy, covering the dynamic grid-dimension expansion path that protects authored spans extending beyond the initial container config. The run used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`, and it reported existing runtime warning noise.

The non-finite slot padding fallback continuation passed with:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_reports_non_finite_slot_padding_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
```

That focused run passed `1 passed; 0 failed; 1874 filtered out`. It builds a Taffy-eligible `HorizontalBox` whose linear slot carries `f32::INFINITY` padding and verifies the route report is `Flex` + `LegacyZircon` + `SlotFramePolicy`. This complements the negative-padding fallback case and proves the finite-spacing guard is visible through diagnostics. The run used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`, and it reported existing runtime warning noise.

The slot-alignment fallback continuation passed with:

```powershell
cargo test -p zircon_runtime --lib slot_alignment --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
```

That focused run passed `2 passed; 0 failed; 1869 filtered out`. It covers two Taffy-eligible flex boxes that must stay on the Zircon fallback path: a `VerticalBox` with per-child main-axis `Center` alignment and a `HorizontalBox` whose child asks for cross-axis `End` alignment without a fixed height. Both report `Flex` + `LegacyZircon` + `SlotFramePolicy`, proving the Taffy eligibility boundary is explicit for unsupported slot alignment and does not silently treat those cases as native. The run used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`, and it reported existing runtime warning noise.

The GridBox non-fixed alignment fallback continuation passed with:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_reports_grid_slot_alignment_without_fixed_extent_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
```

That focused run passed `1 passed; 0 failed; 1875 filtered out`. It builds a one-cell `GridBox` whose child has default non-fixed extents plus horizontal `Center` alignment, then verifies the route report is `Grid` + `LegacyZircon` + `SlotFramePolicy`. This keeps the grid `justify_self`/`align_self` mapping limited to fixed-size child axes where Zircon and Taffy agree on the aligned extent. The run used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`, and it reported existing runtime warning noise.

The 2026-05-22 WrapBox slot style continuation passed:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_maps_wrap_slot_padding_and_cross_axis_alignment_without_fallback --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
cargo test -p zircon_runtime --lib taffy_layout_pass --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
```

The focused run passed `1 passed; 0 failed; 1856 filtered out`, and the warmed module rerun passed `13 passed; 0 failed; 1844 filtered out`. The new case builds one `WrapBox` row whose first fixed child establishes line height, then gives the second fixed child finite padding plus vertical `End` alignment. The resulting `UiFrame::new(35.0, 16.0, 20.0, 10.0)` stays on the Taffy-native path with `UiLayoutEngineFamily::Wrap`, proving wrap-specific slot margin and fixed cross-axis alignment are covered separately from linear and grid mapping. These runs used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`, and they reported existing runtime warning noise.

The 2026-05-22 fallback reason completion passed:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_reports --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-wrap-authority-20260522 --message-format short --color never
```

That focused run passed `2 passed; 0 failed; 1857 filtered out`. It covers two Taffy-eligible `HorizontalBox` roots that deliberately cannot stay native: one has a collapsed child and must report `UnsupportedChildVisibility`, and one has a non-default child anchor plus position and must report `ChildPlacementPolicy`. This closes the explicit fallback reasons named by `taffy_arrange.rs` alongside the existing slot frame, canvas placement, and Zircon-owned semantic cases. The later warmed `taffy_layout_pass` rerun passed `29 passed; 0 failed; 1869 filtered out`, covering the full current Taffy pass gate including Auto/Stretch/StretchContent slot sizing, vertical main-axis slot sizing, horizontal/vertical linear slot min/max bounds, grid slot span/track expansion, non-finite padding fallback, horizontal/vertical linear slot alignment, unsupported slot-alignment fallback, GridBox non-fixed alignment fallback, and distinct fallback-reason aggregation. The run used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`, and it reported existing runtime warning noise.

The measured leaf sizing continuation passed with:

```powershell
cargo test -p zircon_runtime --lib taffy_layout_pass_uses_measured_text_and_image_desired_sizes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-measurement --message-format short --color never
```

That focused test passed `1 passed; 0 failed; 1740 filtered out` and proves Label text measurement plus IconButton image/padding measurement produce desired sizes that the Taffy-native flex arrange path consumes for final child frames. An earlier retry against `D:\cargo-targets\zircon-layout-impl` compiled but failed to write Cargo dep-info after the shared target fingerprint path disappeared; the first cold run in `D:\cargo-targets\zircon-layout-measurement` exceeded the 30-minute tool timeout while continuing to compile, and the warmed rerun above is the accepted evidence.

The Block/Container bridge boundary passed with:

```powershell
cargo test -p zircon_runtime --lib taffy_bridge_keeps_block_display_explicit_and_container_zircon_owned --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never
cargo test -p zircon_runtime_interface ui_layout_engine_block_is_explicit_not_implied_by_current_container_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-interface --message-format short --color never
```

The runtime bridge test passed `1 passed; 0 failed; 1723 filtered out` and proves `Block` maps to Taffy block display while generic `Container` still returns no Taffy style and reports Zircon-owned fallback. The interface test passed `1 passed; 0 failed; 101 filtered out` and proves no current `UiContainerKind` emits `UiLayoutEngineFamily::Block`, while an explicit `Block` request still selects Taffy natively.

The 2026-05-23 runtime arrange boundary guard extends `layout_pass_routes_supported_containers_through_taffy_arrange` so the runtime pass cannot accidentally present `Block` or generic `Container` as native arranged Taffy routes. This is a source-level guard only until the next `taffy_layout_pass` Cargo rerun is possible; current shared Hub/MUI/sound/render Cargo activity leaves `Cargo.lock` dirty and blocks clean locked/offline validation for this slice.

The 2026-05-23 invalid layout value guard adds `InvalidLayoutValue` to the layout-engine fallback reason contract. `taffy_arrange.rs` now checks parent frame values, container style inputs, child `AxisConstraint` values, measured desired sizes, and consumed `UiLinearSlotSizing` values before building Taffy nodes; non-finite values route through the Zircon fallback instead of being clamped or passed to Taffy. `taffy_bridge.rs` also rejects non-finite style inputs directly. Focused runtime tests cover non-finite container config, child constraints, and consumed linear slot sizing, the bridge test covers non-finite style conversion inputs, and `layout_engine_contracts` covers serialization of the new fallback reason. These source/test changes are pending Cargo rerun while unrelated active Cargo/rustc processes own the shared workspace.

The 2026-05-24 Masonry continuation adds `UiMasonryBoxConfig` and `UiContainerKind::MasonryBox` as a Zircon-owned runtime layout. It deliberately does not enter `taffy_arrange.rs`: Masonry children reuse Flow slots for order, padding, and alignment, then `arrange.rs` assigns each visible child into either the shortest current column or the authored sequential column order, matching the MUI Lab `sequential` switch. `measure.rs` computes content size from the same ordered children and slot padding so scroll containers can consume the final staggered height. The template builders and v2 surface-tree parser both accept `kind = "Masonry"`/`"MasonryBox"` with `columns`, `gap` or `spacing`, and `sequential`, which lets Material Lab mount a real `Masonry` container without treating it as Grid or Wrap. Focused runtime validation now passes for type-checking, shortest-column render/hit authority, sequential placement, and template parsing; the editor-side Material Lab Cargo filter still timed out during dependency compilation, so the accepted editor coverage for this slice is static TOML/theme validation plus the runtime layout tests.

The SizeBox TOML parser contract passed with:

```powershell
cargo test -p zircon_runtime template_tree_builder_parses_size_box_container_contract -- --nocapture
```

The runtime-interface layout engine contract passed with:

```powershell
cargo test -p zircon_runtime_interface ui_layout_engine_request_maps_current_container_contracts_to_engine_families -- --nocapture
```
