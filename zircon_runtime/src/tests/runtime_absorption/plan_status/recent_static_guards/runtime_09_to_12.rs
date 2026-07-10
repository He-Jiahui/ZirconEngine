use super::super::support::assert_contains_all;
use super::document_sources::RecentStaticGuardSources;

pub(super) fn assert_runtime_09_to_12_anchors(sources: &RecentStaticGuardSources) {
    let runtime_09_anchors = [
        "runtime_09_ui_architecture_doc_records_current_boundaries",
        "runtime_09_ui_architecture_baselines_match_current_source_scan",
        "runtime_09_v2_verdict_matches_runtime_and_interface_modules",
        "runtime_09_m0_ui_architecture_static_passed",
        "v2-replacement-mainline",
        "runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending",
        "runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt",
        "routed_result",
        "runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending",
        "runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt",
        "has_pointer_capture_for_owner",
        "runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending",
        "runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt",
        "split_row_label_table_text",
        "runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending",
        "runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt",
        "component_name_interaction_fallback",
        "runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending",
        "runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt",
        "runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending",
        "runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt",
        "state_visible_flag",
        "runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending",
        "runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt",
        "fallback_properties",
        "runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending",
        "runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt",
        "UiLayoutEngineBackend::Zircon",
        "UiLayoutEngineCapability::zircon",
        "zircon_selected_count",
        "runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending",
        "runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt",
        "default_open_boolean_value",
        "runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending",
        "plan_scrollable_virtual_window",
        "runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending",
        "UI_TEMPLATE_RUNTIME_PIPELINE_STAGES",
        "UiTemplateRuntimePipeline",
    ];
    let runtime_10_plan_anchors = [
        "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
        "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
        "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
        "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
        "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
        "M2 UI 镜像契约 pending gate",
        "ui_contract_duplicate_public_types = 0",
        "ui_v2_contract_sync_anchors = 9/9",
        "UiComponentApiVersion",
        "v2-replacement-mainline",
        "minimal`/`headless` profile 通过 `uses_render_bridge()` 跳过 render bridge",
        "capture 返回空帧",
        "bind/unbind/present 为 no-op",
    ];
    let runtime_10_doc_anchors = [
        "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
        "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
        "minimal` and `headless` profiles now skip `RuntimeRenderBridge` creation",
        "frame capture returns an empty encoded frame",
        "surface bind/unbind/present operations are no-ops",
    ];
    let runtime_10_interface_anchors = [
        "Runtime 10 UI Contract M2 Gate",
        "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
        "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
        "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
        "ui_contract_duplicate_public_types = 0",
        "ui_v2_contract_sync_anchors = 9/9",
    ];
    let runtime_11_anchors = [
        "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
        "tasks/ecs_schedule/worker_pool/rayon",
        "parallel_frustum.rs",
        "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending",
        "direct_rayon_paths = 2",
    ];
    let runtime_12_anchors = [
        "runtime_12_input_stack_contracts_stay_documented_and_exported",
        "runtime_12_action_mapping_keeps_ui_filtered_evaluation_path",
        "runtime_12_gamepad_bridge_keeps_runtime_abi_path",
    ];

    assert_contains_all("Runtime 09 subplan", &sources.archives, &runtime_09_anchors);
    assert_contains_all(
        "Runtime 09 mirror doc",
        sources.runtime_09_doc,
        &runtime_09_anchors,
    );
    assert_contains_all(
        "Runtime 10 subplan",
        &sources.archives,
        &runtime_10_plan_anchors,
    );
    assert_contains_all(
        "Runtime 10 mirror doc",
        sources.runtime_10_doc,
        &runtime_10_doc_anchors,
    );
    assert_contains_all(
        "Runtime 10 interface convergence doc",
        sources.runtime_10_interface_doc,
        &runtime_10_interface_anchors,
    );
    assert_contains_all("Runtime 11 subplan", &sources.archives, &runtime_11_anchors);
    assert_contains_all(
        "Runtime 11 mirror doc",
        sources.runtime_11_doc,
        &runtime_11_anchors,
    );
    assert_contains_all("Runtime 12 subplan", &sources.archives, &runtime_12_anchors);
    assert_contains_all(
        "Runtime 12 mirror doc",
        sources.runtime_12_doc,
        &runtime_12_anchors,
    );
}
