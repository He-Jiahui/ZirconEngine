use super::*;

pub(super) fn assert_runtime_15_maps(sources: &TopLevelMapSources) {
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
