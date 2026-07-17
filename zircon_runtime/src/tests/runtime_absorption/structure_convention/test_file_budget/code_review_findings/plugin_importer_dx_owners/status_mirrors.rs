use super::*;

#[test]
fn runtime_15_plugin_importer_dx_structure_guard_folder_backed_status_is_current() {
    let runtime_15_plan =
        read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(STRUCTURE_GUARD_ROWS);
    let status_map = format!(
        "{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(REVIEW_GUARD_STATUS_MAP),
        read_runtime_src(REVIEW_GUARD_PLUGIN_IMPORTER_STATUS_MAP)
    );
    let date_map = format!(
        "{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(REVIEW_GUARD_DATE_MAP),
        read_runtime_src(REVIEW_GUARD_PLUGIN_IMPORTER_DATE_MAP)
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                FOLDER_BACKED_SLICE,
                FOLDER_BACKED_STATUS,
                PLUGIN_IMPORTER_DX_STRUCTURE_CHILD,
                PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD,
                PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
                PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD,
                PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD,
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD,
                PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD,
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD,
                PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD,
                PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_CHILD,
                PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_CHILD,
                PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_CHILD,
                PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD,
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
                "review_d10_animation_physics_tests_use_sdk_bridge_call",
                "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
                "review_d5_editor_authoring_plugins_use_sdk_macro",
                "review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
                GUARD,
                FOLDER_BACKED_GUARD,
                FOLDER_BACKED_STATUS_GUARD,
                BUDGET_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "status-output slice status map",
        &status_map,
        &[SLICE, STATUS, FOLDER_BACKED_SLICE, FOLDER_BACKED_STATUS],
    );
    assert_contains_all(
        "status-output slice date map",
        &date_map,
        &[SLICE, DATE, FOLDER_BACKED_SLICE, FOLDER_BACKED_DATE],
    );

    let source_status_anchors = [
        PLUGIN_IMPORTER_DX_SOURCE_STATUS_MAP_SLICE,
        PLUGIN_IMPORTER_DX_SOURCE_STATUS_MAP_STATUS,
        PLUGIN_IMPORTER_DX_ROOT_PATHS_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_CHILD_ROWS_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_INVENTORY_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_SOURCES_CHILD,
        PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD,
        PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD,
        PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD,
        PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD,
        REVIEW_GUARD_PLUGIN_IMPORTER_STATUS_MAP,
        FOLDER_BACKED_GUARD,
        PLUGIN_IMPORTER_DX_ROOT_INVENTORY_GUARD,
        FOLDER_BACKED_STATUS_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &source_status_anchors);
    }
    assert_contains_all(
        "status-output slice status map",
        &status_map,
        &[
            PLUGIN_IMPORTER_DX_SOURCE_STATUS_MAP_SLICE,
            PLUGIN_IMPORTER_DX_SOURCE_STATUS_MAP_STATUS,
        ],
    );
    assert_contains_all(
        "status-output slice date map",
        &date_map,
        &[
            PLUGIN_IMPORTER_DX_SOURCE_STATUS_MAP_SLICE,
            PLUGIN_IMPORTER_DX_SOURCE_STATUS_MAP_DATE,
        ],
    );
}
