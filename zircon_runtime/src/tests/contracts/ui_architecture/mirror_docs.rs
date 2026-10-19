use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
#[test]
fn runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts() {
    let architecture_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = history("versions/reconciliation-515/6b7c6e18f36ee98dafb0f0d44e06a4bf67db37edefbb5af501aeef63c1833fbf/docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md");
    let runtime_index = history("versions/reconciliation-515/eba15ac42775b6ad479b3ccd5d90869900041fdf944d9b46cee303e21f8b7107/docs/plans/zircon_runtime/runtime/index.md");
    let architecture_review =
        include_str!("../../../../../docs/architecture/runtime-architecture-review-m0.md");
    let interface_doc =
        include_str!("../../../../../docs/architecture/runtime-interface-convergence.md");
    let audit_script = include_str!(
        "../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py"
    );
    let boundary_guard = include_str!("architecture_boundaries.rs");

    for anchor in [
        "EXPECTED_SOURCE_FILE_COUNT = 49",
        "EXPECTED_UI_ENTRY_COUNT = 28",
        "EXPECTED_SURFACE_ENTRY_COUNT = 50",
        "EXPECTED_LEGACY_FULL_HITS = 15",
        "EXPECTED_LEGACY_PRODUCTION_HITS = 0",
        "EXPECTED_LEGACY_PRODUCTION_FILE_COUNT = 0",
        "EXPECTED_TAFFY_PRODUCTION_HITS = 249",
        "EXPECTED_TAFFY_PRODUCTION_FILE_COUNT = 16",
        "LEGACY_MIGRATION_TERMS",
        "MIRROR_DOCS_GUARD",
        "\"runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts\"",
        "\"mirror_docs_guard_present\"",
    ] {
        assert!(audit_script.contains(anchor), "UI audit anchor: {anchor}");
    }

    for anchor in [
        "fn runtime_09_ui_architecture_doc_records_current_boundaries()",
        "fn runtime_09_ui_architecture_baselines_match_current_source_scan()",
        "LEGACY_MIGRATION_TERMS",
        "matching_migration_line_count",
        "const EXPECTED_UI_ENTRY_COUNT: usize = 28;",
        "const EXPECTED_SURFACE_ENTRY_COUNT: usize = 50;",
        "const EXPECTED_SURFACE_SOURCE_ENTRY_COUNT: usize = 50;",
    ] {
        assert!(boundary_guard.contains(anchor), "UI guard anchor: {anchor}");
    }

    for anchor in [
        "expected_ui_entry_count = 28",
        "expected_surface_entry_count = 50",
        "surface_source_module_entry_count = 50",
        "surface_empty_directory_count = 0",
        "ui_taffy_production_hits=249",
    ] {
        assert!(architecture_doc.contains(anchor), "UI inventory: {anchor}");
    }
    // These four mirrors retain their independently recorded dated baseline.
    for (doc_name, doc_source) in [
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
        ("runtime architecture review", architecture_review),
        ("interface convergence", interface_doc),
    ] {
        for anchor in [
            "ui_architecture_boundary",
            "expected_source_file_count = 49",
            "expected_ui_entry_count = 23",
            "expected_surface_entry_count = 45",
            "legacy_full_hits = 15",
            "expected_legacy_full_hits = 15",
            "legacy_production_hits = 0",
            "expected_legacy_production_hits = 0",
            "legacy_production_file_count = 0",
            "expected_legacy_production_file_count = 0",
            "taffy_production_hits = 254",
            "expected_taffy_production_hits = 254",
            "taffy_production_file_count = 16",
            "expected_taffy_production_file_count = 16",
            "runtime_v2_anchor_count = 10",
            "interface_v2_anchor_count = 9",
            "guard_anchor_count = 19",
            "cargo_gate_anchor_count = 7",
            "doc_anchor_count = 61",
            "missing_doc_anchors = []",
            "missing_cargo_gate_anchors = []",
            "mirror_docs_guard_present = true",
            "risks = []",
        ] {
            assert!(doc_source.contains(anchor), "{doc_name}: {anchor}");
        }
    }
    let fixed_09 = history("docs/plans/zircon_runtime/runtime/09/2026-07-09-ui-subsystem-architecture-output-records.md");
    let fixed_index = history(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    for (label, record) in [
        ("fixed Runtime 09", fixed_09),
        ("fixed runtime index", fixed_index),
    ] {
        for anchor in [
        "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts", "runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending",
        "runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending", "runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending",
        "runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending", "runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending",
        "runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending", "runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending",
        "runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending", "runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending",
        "runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending", "runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending",
        "runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending", "runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending",
        "runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending",
        ] {
            assert!(record.contains(anchor), "{label}: {anchor}");
        }
    }
    for anchor in [
        "expected_source_file_count = 49",
        "expected_ui_entry_count = 22",
        "expected_surface_entry_count = 44",
        "taffy_production_hits = 217",
    ] {
        assert!(fixed_09.contains(anchor), "fixed09 inventory: {anchor}");
    }
    for anchor in [
        "expected_source_file_count = 52",
        "expected_ui_entry_count = 18",
        "expected_surface_entry_count = 20",
        "taffy_production_hits = 175",
    ] {
        assert!(fixed_index.contains(anchor), "fixed index: {anchor}");
    }
    for (label, record) in [
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        assert!(
            record.contains("surface_source_module_entry_count = 43"),
            "{label} dated inventory"
        );
    }
}
