use super::{sources::TopLevelMapSources, *};

pub(super) fn assert_expected_slice_maps_are_child_owners(sources: &TopLevelMapSources) {
    assert_contains_all(
        "status expected-slice parent delegates Runtime 15",
        &sources.status_parent,
        &[
            "#[path = \"status/pre_runtime_15.rs\"]",
            "mod pre_runtime_15;",
            "#[path = \"status/runtime_15.rs\"]",
            "mod runtime_15;",
            "runtime_15::expected_status_for_slice(slice)",
            "pre_runtime_15::expected_status_for_slice(slice)",
        ],
    );
    assert_contains_all(
        "date expected-slice parent delegates Runtime 15",
        &sources.date_parent,
        &[
            "#[path = \"date/pre_runtime_15.rs\"]",
            "mod pre_runtime_15;",
            "#[path = \"date/runtime_15.rs\"]",
            "mod runtime_15;",
            "runtime_15::expected_date_for_slice(slice)",
            "pre_runtime_15::expected_date_for_slice(slice)",
        ],
    );
    assert_contains_all(
        "test file budget root mounts expected-slice guard",
        &sources.test_budget_parent,
        &["mod status_output_expected_slices;"],
    );

    for moved_runtime_15_slice in [
        "Runtime 15 F9 runtime prelude required type coverage",
        "Runtime 15 M4 scene world project I/O mesh owner split",
        "Runtime 15 M3 status output Runtime 15 M4 row data split",
        "runtime_15_status_output_runtime_15_m4_row_data_split_static_passed_cargo_deferred",
    ] {
        assert!(
            !sources.status_parent.contains(moved_runtime_15_slice),
            "expected_slices/status.rs should delegate Runtime 15 status literals instead of keeping {moved_runtime_15_slice}"
        );
        assert!(
            !sources.date_parent.contains(moved_runtime_15_slice),
            "expected_slices/date.rs should delegate Runtime 15 date literals instead of keeping {moved_runtime_15_slice}"
        );
    }
    for moved_pre_runtime_15_slice in [
        "Runtime 14 Cargo 验证窗口探测",
        "Runtime 05 plan-status Cargo attempt 状态审计",
        "Runtime 11 full-lib default after graphics exposure retry",
        "Runtime 12 input boundary grouped manager import guard repair",
    ] {
        assert!(
            !sources.status_parent.contains(moved_pre_runtime_15_slice),
            "expected_slices/status.rs should delegate pre-Runtime-15 status literals instead of keeping {moved_pre_runtime_15_slice}"
        );
        assert!(
            !sources.date_parent.contains(moved_pre_runtime_15_slice),
            "expected_slices/date.rs should delegate pre-Runtime-15 date literals instead of keeping {moved_pre_runtime_15_slice}"
        );
    }

    assert_runtime_15_maps(sources);
    assert_pre_runtime_15_maps(sources);
    assert_line_budgets(sources);
    assert_status_and_docs(sources);
}

fn assert_runtime_15_maps(sources: &TopLevelMapSources) {
    assert_contains_all(
        "Runtime 15 status expected-slice child delegates topic owners",
        &sources.status_runtime_15,
        &[
            "pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str>",
            "#[path = \"runtime_15/foundation.rs\"]",
            "mod foundation;",
            "#[path = \"runtime_15/naming_boundary.rs\"]",
            "mod naming_boundary;",
            "#[path = \"runtime_15/m4_surface_cleanup.rs\"]",
            "mod m4_surface_cleanup;",
            "#[path = \"runtime_15/m3_structure_support.rs\"]",
            "mod m3_structure_support;",
            "foundation::expected_status_for_slice(slice)",
            "naming_boundary::expected_status_for_slice(slice)",
            "m4_surface_cleanup::expected_status_for_slice(slice)",
            "m3_structure_support::expected_status_for_slice(slice)",
        ],
    );
    assert_contains_all(
        "Runtime 15 date expected-slice child delegates topic owners",
        &sources.date_runtime_15,
        &[
            "pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str>",
            "#[path = \"runtime_15/foundation.rs\"]",
            "mod foundation;",
            "#[path = \"runtime_15/naming_boundary.rs\"]",
            "mod naming_boundary;",
            "#[path = \"runtime_15/m4_surface_cleanup.rs\"]",
            "mod m4_surface_cleanup;",
            "#[path = \"runtime_15/m3_structure_support.rs\"]",
            "mod m3_structure_support;",
            "foundation::expected_date_for_slice(slice)",
            "naming_boundary::expected_date_for_slice(slice)",
            "m4_surface_cleanup::expected_date_for_slice(slice)",
            "m3_structure_support::expected_date_for_slice(slice)",
        ],
    );

    assert_contains_all(
        "Runtime 15 status expected-slice topic owners preserve representative literals",
        &format!(
            "{}\n{}\n{}\n{}",
            sources.status_runtime_15_foundation,
            sources.status_runtime_15_naming_boundary,
            sources.status_runtime_15_m4_surface_cleanup,
            sources.status_runtime_15_m3_structure_support
        ),
        &[
            "Runtime 15 F9 runtime prelude required type coverage",
            "Runtime 15 M2 core runtime state module naming hard cutover",
            "Runtime 15 M4 scene world project I/O mesh owner split",
            "Runtime 15 M3 status output expected-slice maps split",
            "runtime_15_status_output_expected_slice_maps_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 date expected-slice topic owners preserve representative literals",
        &format!(
            "{}\n{}\n{}\n{}",
            sources.date_runtime_15_foundation,
            sources.date_runtime_15_naming_boundary,
            sources.date_runtime_15_m4_surface_cleanup,
            sources.date_runtime_15_m3_structure_support
        ),
        &[
            "Runtime 15 F9 runtime prelude required type coverage",
            "Runtime 15 M2 core runtime state module naming hard cutover",
            "Runtime 15 M4 scene world project I/O mesh owner split",
            "Runtime 15 M3 status output expected-slice maps split",
            "Some(\"2026-06-23\")",
        ],
    );
}

fn assert_pre_runtime_15_maps(sources: &TopLevelMapSources) {
    assert_contains_all(
        "pre-Runtime-15 status expected-slice parent delegates legacy status literals",
        &sources.status_pre_runtime_15,
        &[
            "pub(super) fn expected_status_for_slice(slice: &str) -> &'static str",
            "#[path = \"pre_runtime_15/runtime_01_05.rs\"]",
            "mod runtime_01_05;",
            "#[path = \"pre_runtime_15/runtime_06_10.rs\"]",
            "mod runtime_06_10;",
            "#[path = \"pre_runtime_15/runtime_11_14.rs\"]",
            "mod runtime_11_14;",
            "runtime_01_05::expected_status_for_slice(slice)",
            "runtime_06_10::expected_status_for_slice(slice)",
            "runtime_11_14::expected_status_for_slice(slice)",
            "mirror_docs_static_passed_cargo_pending",
        ],
    );
    assert_contains_all(
        "pre-Runtime-15 status expected-slice children own legacy status literals",
        &format!(
            "{}\n{}\n{}",
            sources.status_pre_runtime_15_runtime_01_05,
            sources.status_pre_runtime_15_runtime_06_10,
            sources.status_pre_runtime_15_runtime_11_14
        ),
        &[
            "pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str>",
            "Runtime 14 Cargo 验证窗口探测",
            "Runtime 05 plan-status Cargo attempt 状态审计",
            "Runtime 11 full-lib default after graphics exposure retry",
            "Runtime 12 input boundary grouped manager import guard repair",
        ],
    );
    assert_contains_all(
        "pre-Runtime-15 date expected-slice parent delegates legacy date literals",
        &sources.date_pre_runtime_15,
        &[
            "pub(super) fn expected_date_for_slice(slice: &str) -> &'static str",
            "#[path = \"pre_runtime_15/runtime_01_05.rs\"]",
            "mod runtime_01_05;",
            "#[path = \"pre_runtime_15/runtime_06_10.rs\"]",
            "mod runtime_06_10;",
            "#[path = \"pre_runtime_15/runtime_11_14.rs\"]",
            "mod runtime_11_14;",
            "runtime_01_05::expected_date_for_slice(slice)",
            "runtime_06_10::expected_date_for_slice(slice)",
            "runtime_11_14::expected_date_for_slice(slice)",
            "2026-06-14",
        ],
    );
    assert_contains_all(
        "pre-Runtime-15 date expected-slice children own legacy date literals",
        &format!(
            "{}\n{}\n{}",
            sources.date_pre_runtime_15_runtime_01_05,
            sources.date_pre_runtime_15_runtime_06_10,
            sources.date_pre_runtime_15_runtime_11_14
        ),
        &[
            "pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str>",
            "Runtime 10 F18 asset manager resolution return shape",
            "Runtime 14 animation runtime-status focused recheck timeout",
            "Runtime 11 full-lib default after graphics exposure retry",
            "Runtime 12 input boundary grouped manager import guard repair",
        ],
    );
}

fn assert_line_budgets(sources: &TopLevelMapSources) {
    for (path, source) in [
        (
            "plan_status/status_output_tables/expected_slices/status.rs",
            sources.status_parent.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
            sources.status_runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs",
            sources.status_runtime_15_foundation.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
            sources.status_runtime_15_naming_boundary.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
            sources.status_runtime_15_m4_surface_cleanup.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
            sources.status_runtime_15_m3_structure_support.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
            sources.status_pre_runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs",
            sources.status_pre_runtime_15_runtime_01_05.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs",
            sources.status_pre_runtime_15_runtime_06_10.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_11_14.rs",
            sources.status_pre_runtime_15_runtime_11_14.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date.rs",
            sources.date_parent.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
            sources.date_runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs",
            sources.date_runtime_15_foundation.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
            sources.date_runtime_15_naming_boundary.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
            sources.date_runtime_15_m4_surface_cleanup.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
            sources.date_runtime_15_m3_structure_support.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
            sources.date_pre_runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_01_05.rs",
            sources.date_pre_runtime_15_runtime_01_05.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs",
            sources.date_pre_runtime_15_runtime_06_10.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs",
            sources.date_pre_runtime_15_runtime_11_14.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices.rs",
            sources.status_output_expected_slices_guard.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

fn assert_status_and_docs(sources: &TopLevelMapSources) {
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
