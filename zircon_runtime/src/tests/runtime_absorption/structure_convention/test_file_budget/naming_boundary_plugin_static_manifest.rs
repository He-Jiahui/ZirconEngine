use super::*;

const STATUS: &str =
    "runtime_15_plugin_static_manifest_naming_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 plugin static manifest naming guard child-owner split";
const GUARD: &str = "runtime_15_plugin_static_manifest_naming_guard_is_child_owner";

#[test]
fn runtime_15_plugin_static_manifest_naming_guard_is_child_owner() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest.rs",
    );
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest/contract_owners.rs",
    );

    assert_contains_all(
        "Runtime 15 plugin static manifest naming parent mounts contract owner child",
        &parent,
        &[
            "#[path = \"plugin_static_manifest/contract_owners.rs\"]",
            "mod contract_owners;",
        ],
    );
    assert!(
        !parent.contains("fn runtime_15_plugin_static_manifest_contract_owners_use_domain_names"),
        "runtime_15_m2/plugin_static_manifest.rs should mount the contract_owners child instead of defining the naming guard"
    );

    assert_contains_all(
        "Runtime 15 plugin static manifest contract child owns domain naming guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_plugin_static_manifest_contract_owners_use_domain_names",
            "src/tests/plugin_extensions/static_manifest_contracts",
            "feature_bundle_rows.rs",
            "package_coordinate_resolution.rs",
            "package_id_tokens.rs",
            "package_kind_fields.rs",
            "runtime_15_plugin_static_manifest_contract_owner_naming_hard_cutover_static_passed_cargo_deferred",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest/contract_owners.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/naming_guard_maps.rs",
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
                GUARD,
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest.rs",
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest/contract_owners.rs",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status/date maps record plugin static manifest child owner",
        &format!("{status_map}\n{date_map}"),
        &[SLICE, STATUS, "2026-06-30"],
    );
}
