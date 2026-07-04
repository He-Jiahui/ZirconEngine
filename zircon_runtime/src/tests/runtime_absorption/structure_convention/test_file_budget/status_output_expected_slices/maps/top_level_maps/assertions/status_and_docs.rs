use super::*;

pub(super) fn assert_status_and_docs(sources: &TopLevelMapSources) {
    assert_contains_all(
        "status-output Runtime 15 row data",
        &sources.status_rows,
        &[
            "Runtime 15 M3 status output expected-slice maps split",
            "runtime_15_status_output_expected_slice_maps_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_slices/status.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
            "runtime_15_status_output_expected_slice_maps_are_child_owners",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", sources.runtime_15_plan.as_str()),
        ("Runtime index", sources.runtime_index.as_str()),
        ("review findings", sources.review_findings.as_str()),
        (
            "structure convention",
            sources.structure_convention.as_str(),
        ),
        ("module convention doc", sources.module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 status output expected-slice maps split",
                "runtime_15_status_output_expected_slice_maps_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_slices/status.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
                "plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
                "plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
                "runtime_15_status_output_expected_slice_maps_are_child_owners",
            ],
        );
    }
}
