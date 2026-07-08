use super::*;

#[test]
fn runtime_15_f8_child_owner_structure_guard_folder_backed_status_is_current() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(STRUCTURE_GUARD_ROWS);
    let status_map = format!(
        "{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(REVIEW_GUARD_STATUS_MAP),
        read_runtime_src(REVIEW_GUARD_F8_STATUS_MAP)
    );
    let date_map = format!(
        "{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(REVIEW_GUARD_DATE_MAP),
        read_runtime_src(REVIEW_GUARD_F8_DATE_MAP)
    );

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
                STRUCTURE_GUARD_OWNER,
                PARENT,
                TEXTURE_IMPORT_SETTINGS,
                DESCRIPTOR_BUILDER,
                DESCRIPTOR_PRIVACY,
                GUARD,
                FOLDER_BACKED_GUARD,
                "Cargo gate deferred",
            ],
        );
        assert_contains_all(label, source, REVIEW_GUARDS);
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
