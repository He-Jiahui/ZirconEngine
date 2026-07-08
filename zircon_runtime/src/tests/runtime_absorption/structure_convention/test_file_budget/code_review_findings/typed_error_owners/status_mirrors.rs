use super::*;

#[test]
fn runtime_15_typed_error_structure_guard_folder_backed_status_is_current() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = typed_error_structure_status_row_source();
    let status_map = typed_error_structure_status_map_source();
    let date_map = typed_error_structure_date_map_source();

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
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
                TYPED_ERROR_STRUCTURE_CHILD,
                TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD,
                TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
                TYPED_ERROR_TOP_LEVEL_STATUS_MIRRORS_CHILD,
                TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_CHILD,
                TYPED_ERROR_STATUS_DOCS_CHILD,
                TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD,
                TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD,
                TYPED_ERROR_STRUCTURE_DELEGATION_CHILD,
                TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD,
                TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_CHILD,
                TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD,
                "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
                "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
                "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
                "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
                "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs",
                "review_f5_texture_loader_uses_typed_error",
                "review_f5_mesh_loader_and_obj_decoder_use_typed_errors",
                "review_f5_asset_authoring_uses_typed_error",
                "review_f5_native_plugin_descriptor_abi_uses_typed_error",
                "review_f5_ui_surface_input_effects_use_typed_errors_before_rejected_reason_boundary",
                "review_f7_asset_artifact_errors_use_asset_import_error_sources",
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
}
