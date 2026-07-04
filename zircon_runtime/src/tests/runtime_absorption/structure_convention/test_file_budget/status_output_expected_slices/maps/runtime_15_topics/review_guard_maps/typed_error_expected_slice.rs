use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_typed_error_maps_are_child_owned() {
    let status_review_child = read_runtime_src(STATUS_REVIEW_CHILD);
    let date_review_child = read_runtime_src(DATE_REVIEW_CHILD);
    let status_typed_error_child = read_runtime_src(STATUS_REVIEW_TYPED_ERROR_CHILD);
    let date_typed_error_child = read_runtime_src(DATE_REVIEW_TYPED_ERROR_CHILD);
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "review expected-slice parents mount typed-error children",
        &format!("{status_review_child}\n{date_review_child}"),
        &[
            "#[path = \"review_guard_maps/typed_error_maps.rs\"]",
            "mod typed_error_maps;",
            "typed_error_maps::expected_status_for_slice(slice)",
            "typed_error_maps::expected_date_for_slice(slice)",
        ],
    );
    for moved_literal in [
        "Runtime 15 M3 typed-error convergence guard child-owner split",
        "Runtime 15 M3 native plugin loader typed-error review guard child-owner split",
        "Runtime 15 M3 typed-error source inventory guard child-owner split",
        "Runtime 15 M3 native live-host replay-runtime typed-error review guard child-owner split",
    ] {
        assert!(
            !status_review_child.contains(moved_literal),
            "status review_guard_maps.rs should delegate typed-error literal {moved_literal}"
        );
        assert!(
            !date_review_child.contains(moved_literal),
            "date review_guard_maps.rs should delegate typed-error literal {moved_literal}"
        );
    }
    assert_contains_all(
        "typed-error expected-slice map children own typed-error literals",
        &format!("{status_typed_error_child}\n{date_typed_error_child}"),
        &[
            "Runtime 15 M3 typed-error convergence guard child-owner split",
            "runtime_15_typed_error_convergence_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 native plugin loader typed-error review guard child-owner split",
            "runtime_15_native_plugin_loader_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 typed-error source inventory guard child-owner split",
            "runtime_15_typed_error_source_inventory_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 native live-host replay-runtime typed-error review guard child-owner split",
            "runtime_15_native_live_host_replay_runtime_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
            "Some(\"2026-06-30\")",
        ],
    );

    for (label, source) in [
        ("status-output expected-slice rows", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 review guard typed-error expected-slice map child split",
                "runtime_15_review_guard_typed_error_expected_slice_map_child_split_static_passed_cargo_deferred",
                STATUS_REVIEW_CHILD,
                STATUS_REVIEW_TYPED_ERROR_CHILD,
                DATE_REVIEW_CHILD,
                DATE_REVIEW_TYPED_ERROR_CHILD,
                "runtime_15_review_guard_expected_slice_typed_error_maps_are_child_owned",
                "Cargo gate deferred",
            ],
        );
    }
}
