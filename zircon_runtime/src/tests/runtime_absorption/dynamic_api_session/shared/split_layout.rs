const SLICE: &str = "Runtime 15 M3 dynamic API session shared data folder-backed split";
const STATUS: &str =
    "runtime_15_dynamic_api_session_shared_data_folder_backed_static_passed_cargo_deferred";
const FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_dynamic_api_session_shared_data_folder_backed_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_dynamic_api_session_shared_data_is_folder_backed";

const PARENT_PATH: &str = "dynamic_api_session/shared.rs";
const CHILD_PATHS: &[&str] = &[
    "dynamic_api_session/shared/abi.rs",
    "dynamic_api_session/shared/behavior.rs",
    "dynamic_api_session/shared/diagnostics.rs",
    "dynamic_api_session/shared/docs.rs",
    "dynamic_api_session/shared/host_requests.rs",
    "dynamic_api_session/shared/slices.rs",
    "dynamic_api_session/shared/source_inventory.rs",
    "dynamic_api_session/shared/split_layout.rs",
];

#[test]
fn runtime_15_dynamic_api_session_shared_data_is_folder_backed() {
    let parent = include_str!("../shared.rs");
    let children = [
        include_str!("abi.rs"),
        include_str!("behavior.rs"),
        include_str!("diagnostics.rs"),
        include_str!("docs.rs"),
        include_str!("host_requests.rs"),
        include_str!("slices.rs"),
        include_str!("source_inventory.rs"),
        include_str!("split_layout.rs"),
    ];

    assert_contains_all(
        "dynamic API shared parent routes child owners",
        parent,
        &[
            r#"#[path = "shared/abi.rs"]"#,
            r#"#[path = "shared/behavior.rs"]"#,
            r#"#[path = "shared/diagnostics.rs"]"#,
            r#"#[path = "shared/docs.rs"]"#,
            r#"#[path = "shared/host_requests.rs"]"#,
            r#"#[path = "shared/slices.rs"]"#,
            r#"#[path = "shared/source_inventory.rs"]"#,
            r#"#[path = "shared/split_layout.rs"]"#,
        ],
    );

    for moved_anchor in [
        "EXPECTED_RUNTIME_10_SOURCE_FILES",
        "EXPECTED_RUNTIME_10_FUNCTION_TABLES",
        "EXPECTED_RUNTIME_10_BEHAVIOR_TEST_ANCHORS",
        "EXPECTED_RUNTIME_10_RUNTIME_DIAGNOSTICS_ANCHORS",
        "EXPECTED_RUNTIME_10_HOST_REQUEST_PAYLOAD_ANCHORS",
        "EXPECTED_RUNTIME_10_MIRROR_DOCS",
        "slice_between",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "dynamic API shared parent should not retain moved owner `{moved_anchor}`"
        );
        assert!(
            children.iter().any(|source| source.contains(moved_anchor)),
            "dynamic API shared children should own moved owner `{moved_anchor}`"
        );
    }

    let headless_profiles = include_str!("../headless_profiles.rs");
    let mirror_docs = include_str!("../mirror_docs.rs");
    let runtime_diagnostics = include_str!("../runtime_diagnostics.rs");
    assert_contains_all(
        "dynamic API shared callers import concrete child owners",
        headless_profiles,
        &["super::shared::slices::slice_between"],
    );
    assert_contains_all(
        "dynamic API mirror-docs imports concrete child owners",
        mirror_docs,
        &[
            "super::shared::abi::{",
            "super::shared::behavior::EXPECTED_RUNTIME_10_BEHAVIOR_TEST_ANCHORS",
            "super::shared::diagnostics::{",
            "super::shared::docs::EXPECTED_RUNTIME_10_MIRROR_DOCS",
            "super::shared::host_requests::EXPECTED_RUNTIME_10_HOST_REQUEST_PAYLOAD_ANCHORS",
            "super::shared::source_inventory::EXPECTED_RUNTIME_10_SOURCE_FILES",
        ],
    );
    assert_contains_all(
        "dynamic API runtime-diagnostics imports concrete child owners",
        runtime_diagnostics,
        &["super::shared::diagnostics::{"],
    );
    for (label, source, forbidden) in [
        (
            "headless profile shared import",
            headless_profiles,
            "super::shared::slice_between",
        ),
        (
            "mirror docs shared glob import",
            mirror_docs,
            "use super::shared::{",
        ),
        (
            "runtime diagnostics shared glob import",
            runtime_diagnostics,
            "use super::shared::{",
        ),
    ] {
        assert!(
            !source.contains(forbidden),
            "{label} should not retain legacy shared import `{forbidden}`"
        );
    }

    for (path, source, max_lines) in [
        (PARENT_PATH, parent, 30usize),
        (CHILD_PATHS[0], children[0], 90),
        (CHILD_PATHS[1], children[1], 80),
        (CHILD_PATHS[2], children[2], 180),
        (CHILD_PATHS[3], children[3], 40),
        (CHILD_PATHS[4], children[4], 180),
        (CHILD_PATHS[5], children[5], 25),
        (CHILD_PATHS[6], children[6], 80),
        (CHILD_PATHS[7], children[7], 210),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    let row_data = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs"
    );
    assert_contains_all(
        "module-convention row data records dynamic API shared data split",
        row_data,
        &[
            SLICE,
            STATUS,
            PARENT_PATH,
            CHILD_PATHS[0],
            CHILD_PATHS[7],
            GUARD,
        ],
    );

    let status_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps.rs"
    );
    assert_contains_all("structure route status map", status_map, &[SLICE, STATUS]);

    let date_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps.rs"
    );
    assert_contains_all("structure route date map", date_map, &[SLICE, "2026-07-05"]);

    for source in [
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"),
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"),
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md"),
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md"),
        include_str!(
            "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
        ),
    ] {
        assert_contains_all("dynamic API shared data status mirror", source, &[
            SLICE,
            STATUS,
            GUARD,
            CHILD_PATHS[7],
        ]);
    }

    let frameworks = include_str!(
        "../../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md"
    );
    assert_contains_all(
        "frameworks plan records dynamic API shared data split",
        frameworks,
        &[SLICE, STATUS, FRAMEWORKS_STATUS, GUARD],
    );
}

fn assert_contains_all(label: &str, source: &str, needles: &[&str]) {
    let missing = needles
        .iter()
        .copied()
        .filter(|needle| !source.contains(needle))
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} missing expected anchors:\n{}",
        missing.join("\n")
    );
}
