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

    for audit_anchor in [
        "EXPECTED_SOURCE_FILE_COUNT = 52",
        "EXPECTED_UI_ENTRY_COUNT = 20",
        "EXPECTED_SURFACE_ENTRY_COUNT = 26",
        "EXPECTED_LEGACY_FULL_HITS = 70",
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

    assert!(
        architecture_doc.contains("expected_surface_entry_count = 26"),
        "current UI architecture doc should mirror the 26-entry surface map"
    );
    let numbered_status = concat!(
        include_str!("../../../../../docs/plans/zircon_runtime/runtime/09/2026-07-09-ui-subsystem-architecture-output-records.md"),
        include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md")
    );
    let mirror_docs = [
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
        ("runtime architecture review", architecture_review),
        ("runtime interface convergence doc", interface_doc),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for expected_anchor in [
            "ui_architecture_boundary",
            "expected_source_file_count = 52",
            "expected_ui_entry_count = 20",
            "legacy_full_hits = 70",
            "expected_legacy_full_hits = 70",
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
                doc_source.contains(expected_anchor) || numbered_status.contains(expected_anchor),
                "{doc_name} should mirror Runtime 09 UI architecture audit anchor `{expected_anchor}`"
            );
        }
    }
}
