use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_f8_api_convergence_parent_mounts_child_owners(
    sources: &CodeReviewFindingsSources,
) {
    assert_contains_all(
        "F8 API convergence parent only mounts focused child review guard owners",
        &sources.f8_api_convergence,
        &[
            "#[path = \"f8_api_convergence/texture_import_settings.rs\"]",
            "mod texture_import_settings;",
            "#[path = \"f8_api_convergence/descriptor_builder.rs\"]",
            "mod descriptor_builder;",
            "#[path = \"f8_api_convergence/descriptor_privacy.rs\"]",
            "mod descriptor_privacy;",
        ],
    );
    assert_eq!(
        sources.f8_api_convergence.matches("#[test]").count(),
        0,
        "f8_api_convergence.rs should only mount child review guard owners"
    );
    for child_owned_test in [
        "fn review_f8_texture_import_settings_use_fallible_apply_not_with",
        "fn review_f8_runtime_plugin_descriptor_exposes_builder_scaffold",
        "fn review_f8_first_party_runtime_plugin_descriptors_use_builder",
        "fn review_f8_runtime_plugin_descriptor_test_fixtures_use_builder",
        "fn review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors",
        "fn review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
        "fn review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending",
    ] {
        assert!(
            !sources.f8_api_convergence.contains(child_owned_test),
            "child-owned F8 review guard `{child_owned_test}` should not return to f8_api_convergence.rs"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_f8_direct_assertions_guard_is_folder_backed() {
    let f8_parent = read_runtime_src(F8_DIRECT_ASSERTIONS_CHILD);
    let child_blob = f8_direct_assertion_child_source_blob();
    let sources = super::super::super::source_inventory::code_review_findings_sources();

    assert_f8_api_convergence_parent_mounts_child_owners(&sources);
    review_children::assert_f8_review_children_are_folder_backed(&sources);
    budgets::assert_f8_direct_assertions_children_line_budgets_are_current();
    for (_, child_path, child_guard) in F8_DIRECT_ASSERTIONS_GUARD_CHILDREN {
        assert!(
            f8_parent.contains(child_path),
            "F8 direct assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "F8 direct assertions child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !f8_parent
            .contains("F8 API convergence parent only mounts focused child review guard owners"),
        "f8.rs should delegate F8 parent mount assertions to parent_mounts.rs"
    );
    assert_contains_all(
        "F8 direct assertions parent records folder-backed status",
        &f8_parent,
        &[
            F8_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE,
            F8_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS,
            F8_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
            F8_DIRECT_ASSERTIONS_BUDGET_GUARD,
        ],
    );
}
