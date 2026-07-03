use super::*;

#[test]
fn runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts() {
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let architecture_review =
        include_str!("../../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let interface_doc =
        include_str!("../../../../../docs/engine-architecture/runtime-interface-convergence.md");
    let audit_script = include_str!(
        "../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py"
    );
    let ui_guard = [
        include_str!("../ui_architecture.rs"),
        include_str!("architecture_boundaries.rs"),
        include_str!("legacy_renames.rs"),
        include_str!("mirror_docs.rs"),
    ]
    .join("\n");
    let cargo_gate_guard = include_str!("../plan_status/cargo_gates/middle.rs");

    for guard_anchor in [
        "runtime_09_ui_architecture_doc_records_current_boundaries",
        "runtime_09_ui_architecture_baselines_match_current_source_scan",
        "runtime_09_v2_verdict_matches_runtime_and_interface_modules",
        "runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt",
        "runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt",
        "runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt",
        "runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt",
        "runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt",
        "runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt",
        "runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt",
        "runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt",
        "runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt",
        "runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt",
        "runtime_09_ui_input_events_route_through_single_dispatch_authority",
        "runtime_09_taffy_layout_pass_order_uses_bridge_authority",
        "runtime_09_virtualization_scroll_boundary_records_invalidation_authority",
        "runtime_09_template_pipeline_boundary_records_compile_instance_validate_authority",
        "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
        "runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation",
    ] {
        assert!(
            ui_guard.contains(guard_anchor) || cargo_gate_guard.contains(guard_anchor),
            "Runtime 09 guard anchor `{guard_anchor}` should stay visible to ui_architecture_boundary"
        );
    }

    for audit_anchor in [
        "EXPECTED_SOURCE_FILE_COUNT = 52",
        "EXPECTED_UI_ENTRY_COUNT = 19",
        "EXPECTED_SURFACE_ENTRY_COUNT = 21",
        "EXPECTED_LEGACY_FULL_HITS = 54",
        "EXPECTED_LEGACY_PRODUCTION_HITS = 0",
        "EXPECTED_LEGACY_PRODUCTION_FILE_COUNT = 0",
        "EXPECTED_TAFFY_PRODUCTION_HITS = 175",
        "EXPECTED_TAFFY_PRODUCTION_FILE_COUNT = 10",
        "MIRROR_DOCS_GUARD",
        "\"runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts\"",
        "\"mirror_docs_guard_present\"",
    ] {
        assert!(
            audit_script.contains(audit_anchor),
            "ui_architecture_boundary should expose audit anchor `{audit_anchor}`"
        );
    }

    let mirror_docs = [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
        ("runtime architecture review", architecture_review),
        ("runtime interface convergence doc", interface_doc),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for expected_anchor in [
            "ui_architecture_boundary",
            "expected_source_file_count = 52",
            "expected_ui_entry_count = 19",
            "expected_surface_entry_count = 21",
            "legacy_full_hits = 54",
            "expected_legacy_full_hits = 54",
            "legacy_production_hits = 0",
            "expected_legacy_production_hits = 0",
            "legacy_production_file_count = 0",
            "expected_legacy_production_file_count = 0",
            "taffy_production_hits = 175",
            "expected_taffy_production_hits = 175",
            "taffy_production_file_count = 10",
            "expected_taffy_production_file_count = 10",
            "runtime_v2_anchor_count = 10",
            "interface_v2_anchor_count = 9",
            "guard_anchor_count = 19",
            "cargo_gate_anchor_count = 7",
            "doc_anchor_count = 61",
            "missing_doc_anchors = []",
            "missing_cargo_gate_anchors = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
            "runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending",
            "runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending",
            "runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending",
            "runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending",
            "runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending",
            "runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending",
        ] {
            assert!(
                doc_source.contains(expected_anchor),
                "{doc_name} should mirror Runtime 09 UI architecture audit anchor `{expected_anchor}`"
            );
        }
    }
}
