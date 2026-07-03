use super::super::super::*;

const PLUGIN_IMPORTER_DX_STATUS_DOC_SLICE: &str =
    "Runtime 15 M3 plugin-importer DX status-doc guard child-owner split";
const PLUGIN_IMPORTER_DX_STATUS_DOC_STATUS: &str =
    "runtime_15_plugin_importer_dx_status_docs_child_owner_split_static_passed_cargo_deferred";
const PLUGIN_IMPORTER_DX_STATUS_DOC_GUARD: &str =
    "runtime_15_plugin_importer_dx_status_docs_are_child_owner";
const PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_docs.rs";
const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_SLICE: &str =
    "Runtime 15 M3 plugin-importer DX structure assertions guard child-owner split";
const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_STATUS: &str =
    "runtime_15_plugin_importer_dx_structure_assertions_guard_child_owner_split_static_passed_cargo_deferred";
const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD: &str =
    "runtime_15_plugin_importer_dx_structure_assertions_are_child_owner";
const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions.rs";
const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_SLICE: &str =
    "Runtime 15 M3 plugin-importer DX structure assertions guard folder-backed split";
const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_STATUS: &str =
    "runtime_15_plugin_importer_dx_structure_assertions_guard_folder_backed_static_passed_cargo_deferred";
const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_GUARD: &str =
    "runtime_15_plugin_importer_dx_structure_assertions_guard_folder_backed_status_is_current";
const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD_GUARD: &str =
    "runtime_15_plugin_importer_dx_structure_assertions_children_are_child_owned";
const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/review_mounts.rs";
const PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/delegation.rs";
const PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/child_ownership.rs";
const PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/status_mirrors.rs";
const PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_SLICE: &str =
    "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard child-owner split";
const PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_STATUS: &str =
    "runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_child_owner_split_static_passed_cargo_deferred";
const PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_GUARD: &str =
    "runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner";
const PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk.rs";

#[test]
fn runtime_15_plugin_importer_dx_status_docs_are_child_owner() {
    assert_plugin_importer_dx_status_docs_are_synced();
}

pub(super) fn assert_plugin_importer_dx_status_docs_are_synced() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = format!(
        "{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows.rs",
        )
    );
    let status_maps = format!(
        "{}\n{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs",
        )
    );
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

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
                "Runtime 15 M3 code review findings plugin-importer DX structure guard child-owner split",
                "runtime_15_code_review_findings_plugin_importer_dx_structure_guard_child_owner_split_static_passed_cargo_deferred",
                "Runtime 15 M3 plugin-importer DX status-doc guard child-owner split",
                "runtime_15_plugin_importer_dx_status_docs_child_owner_split_static_passed_cargo_deferred",
                "Runtime 15 M3 plugin-importer DX source inventory guard child-owner split",
                "runtime_15_plugin_importer_dx_source_inventory_guard_child_owner_split_static_passed_cargo_deferred",
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_SLICE,
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_STATUS,
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_SLICE,
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_STATUS,
                PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_SLICE,
                PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_STATUS,
                "Runtime 15 M3 plugin-importer DX review guard child-owner split",
                "runtime_15_plugin_importer_dx_review_guard_child_owner_split_static_passed_cargo_deferred",
                "Runtime 15 M3 plugin-importer D13 SDK review guard child-owner split",
                "runtime_15_plugin_importer_d13_sdk_review_guard_child_owner_split_static_passed_cargo_deferred",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners.rs",
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_OWNER,
                PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_OWNER,
                PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_OWNER,
                PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_OWNER,
                PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_OWNER,
                PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_OWNER,
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/source_inventory.rs",
                PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER,
                "runtime_15_code_review_findings_plugin_importer_dx_structure_guard_is_child_owner",
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD,
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD_GUARD,
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_GUARD,
                PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_GUARD,
                "runtime_15_plugin_importer_dx_source_inventory_is_child_owner",
                PLUGIN_IMPORTER_DX_STATUS_DOC_GUARD,
                "runtime_15_code_review_findings_tests_are_folder_backed",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/manifest_parity.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_crates.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_exports.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_manifests.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d5_editor_authoring.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d8_registration_builder.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d9_editor_runtime_mirror.rs",
                "review_d10_animation_physics_tests_use_sdk_bridge_call",
                "review_d5_editor_authoring_plugins_use_sdk_macro",
                "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
                "review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
                "review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "status/date expected-slice maps",
        &status_maps,
        &[
            PLUGIN_IMPORTER_DX_STATUS_DOC_SLICE,
            PLUGIN_IMPORTER_DX_STATUS_DOC_STATUS,
            "Runtime 15 M3 plugin-importer DX source inventory guard child-owner split",
            "runtime_15_plugin_importer_dx_source_inventory_guard_child_owner_split_static_passed_cargo_deferred",
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_SLICE,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_STATUS,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_SLICE,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_STATUS,
            PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_SLICE,
            PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_STATUS,
            "2026-07-02",
            "2026-06-30",
        ],
    );
    assert_contains_all(
        "runtime architecture session note",
        &session_note,
        &[
            PLUGIN_IMPORTER_DX_STATUS_DOC_SLICE,
            PLUGIN_IMPORTER_DX_STATUS_DOC_STATUS,
            PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER,
            PLUGIN_IMPORTER_DX_STATUS_DOC_GUARD,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_SLICE,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_STATUS,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_OWNER,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_SLICE,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_STATUS,
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_OWNER,
            PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_OWNER,
            PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_OWNER,
            PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_OWNER,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD_GUARD,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_GUARD,
            PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_SLICE,
            PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_STATUS,
            PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_OWNER,
            PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_GUARD,
        ],
    );
}
