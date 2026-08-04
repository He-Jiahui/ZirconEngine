use super::*;

#[test]
fn runtime_15_ui_component_catalog_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/component_catalog.rs");
    let catalog_inventory = read_runtime_src("ui/tests/component_catalog/catalog_inventory.rs");
    let descriptor_contracts =
        read_runtime_src("ui/tests/component_catalog/descriptor_contracts.rs");
    let registry_queries = read_runtime_src("ui/tests/component_catalog/registry_queries.rs");

    assert_contains_all(
        "UI component catalog parent mounts folder-backed children",
        &parent,
        &[
            "mod catalog_inventory;",
            "mod component_state;",
            "mod descriptor_contracts;",
            "mod registry_queries;",
            "fn assert_category_component_ids(",
            "fn assert_value_matches_schema_kind(",
            "fn test_asset_source(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/component_catalog.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "runtime_component_catalog_contains_showcase_v1_controls",
        "runtime_component_catalog_marks_v2_model_tiers_and_layout_roles",
        "runtime_component_descriptors_validate_palette_and_schema_contracts",
        "runtime_component_registry_filters_by_host_capabilities_and_reports_missing",
        "runtime_component_registry_builds_descriptor_palette_views",
        "runtime_component_registry_revision_changes_only_for_descriptor_set_changes",
        "runtime_component_catalog_schemas_are_normalized_and_type_consistent",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI component catalog test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI component catalog inventory child owns V1 catalog coverage",
        &catalog_inventory,
        &["fn runtime_component_catalog_contains_showcase_v1_controls"],
    );
    assert_contains_all(
        "UI component catalog descriptor child owns descriptor contracts",
        &descriptor_contracts,
        &[
            "fn runtime_component_catalog_marks_v2_model_tiers_and_layout_roles",
            "fn runtime_component_descriptors_validate_palette_and_schema_contracts",
            "fn runtime_component_catalog_schemas_are_normalized_and_type_consistent",
        ],
    );
    assert_contains_all(
        "UI component catalog registry child owns query and revision contracts",
        &registry_queries,
        &[
            "fn runtime_component_registry_filters_by_host_capabilities_and_reports_missing",
            "fn runtime_component_registry_builds_descriptor_palette_views",
            "fn runtime_component_registry_revision_changes_only_for_descriptor_set_changes",
        ],
    );

    let child_test_total = [
        catalog_inventory.as_str(),
        descriptor_contracts.as_str(),
        registry_queries.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 7,
        "UI component catalog children should preserve all 7 parent tests"
    );

    for (path, source) in [
        ("ui/tests/component_catalog.rs", parent.as_str()),
        (
            "ui/tests/component_catalog/catalog_inventory.rs",
            catalog_inventory.as_str(),
        ),
        (
            "ui/tests/component_catalog/descriptor_contracts.rs",
            descriptor_contracts.as_str(),
        ),
        (
            "ui/tests/component_catalog/registry_queries.rs",
            registry_queries.as_str(),
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
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI component catalog test folder split",
                "runtime_15_ui_component_catalog_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/component_catalog.rs",
                "ui/tests/component_catalog/catalog_inventory.rs",
                "ui/tests/component_catalog/descriptor_contracts.rs",
                "runtime_15_ui_component_catalog_tests_are_folder_backed",
            ],
        );
    }
}
