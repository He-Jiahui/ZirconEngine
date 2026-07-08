use super::*;

#[test]
fn runtime_15_status_output_expected_slice_guard_maps_status_is_synced() {
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/base_maps.rs",
    );
    let status_maps = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps/expected_slice_guard_maps.rs",
    );
    let date_maps = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps/expected_slice_guard_maps.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
    );
    let frameworks_index = read_repo("docs/plans/zircon_runtime/frameworks/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "status-support base maps row data records guard maps folder-backed split",
        &status_rows,
        &[
            MAPS_GUARD_BODY_SLICE,
            MAPS_GUARD_BODY_STATUS,
            "structure_convention/test_file_budget/status_slices/maps.rs",
            "structure_convention/test_file_budget/status_slices/maps/guard_body.rs",
            "structure_convention/test_file_budget/status_slices/maps/body/status_mirrors.rs",
            MAPS_GUARD_BODY_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status/date maps record guard maps folder-backed split",
        &format!("{status_maps}\n{date_maps}"),
        &[
            MAPS_GUARD_BODY_SLICE,
            MAPS_GUARD_BODY_STATUS,
            "Some(\"2026-07-07\")",
        ],
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02 plan", frameworks_plan.as_str()),
        ("Frameworks index", frameworks_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                MAPS_GUARD_BODY_SLICE,
                MAPS_GUARD_BODY_STATUS,
                MAPS_GUARD_BODY_FRAMEWORKS_STATUS,
                "structure_convention/test_file_budget/status_slices/maps/guard_body.rs",
                "structure_convention/test_file_budget/status_slices/maps/body/route_mounts.rs",
                "structure_convention/test_file_budget/status_slices/maps/body/status_mirrors.rs",
                MAPS_GUARD_BODY_GUARD,
                "Cargo gate deferred",
            ],
        );
    }

    for (label, source) in [
        ("status-support base maps row data", status_rows.as_str()),
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
                "Runtime 15 M3 status output expected-slice guard maps child-owner split",
                "runtime_15_status_output_expected_slice_guard_maps_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/status_slices/maps.rs",
                "structure_convention/test_file_budget/status_slices/maps/runtime_15_topics.rs",
                "runtime_15_status_output_expected_slice_guard_maps_are_child_owners",
            ],
        );
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 runtime-15 expected-slice topic guard child-module split",
                "runtime_15_runtime_15_expected_slice_topic_guard_child_module_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/status_slices/maps/rt15/runtime_15_expected_slice_maps.rs",
                "runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners",
            ],
        );
    }
}
