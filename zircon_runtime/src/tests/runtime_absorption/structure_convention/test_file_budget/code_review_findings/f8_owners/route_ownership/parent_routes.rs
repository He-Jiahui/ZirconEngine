use super::super::*;
use super::*;

pub(super) fn assert_f8_parent_routes_are_child_owned(sources: &F8ReviewSources) {
    assert_contains_all(
        "F8 API convergence parent mounts focused child owners",
        &sources.parent,
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
        sources.parent.matches("#[test]").count(),
        0,
        "f8_api_convergence.rs should only mount child review guard owners"
    );
    for child_owned_test in REVIEW_GUARDS {
        assert!(
            !sources.parent.contains(&format!("fn {child_owned_test}")),
            "child-owned F8 review guard `{child_owned_test}` should not return to f8_api_convergence.rs"
        );
    }
}
