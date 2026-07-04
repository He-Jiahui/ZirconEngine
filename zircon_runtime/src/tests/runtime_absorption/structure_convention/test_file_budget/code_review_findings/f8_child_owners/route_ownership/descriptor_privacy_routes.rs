use super::super::*;
use super::*;

pub(super) fn assert_f8_descriptor_privacy_routes_are_child_owned(sources: &F8ReviewSources) {
    assert_contains_all(
        "F8 descriptor privacy route mounts focused child owners",
        &sources.descriptor_privacy,
        &[
            "#[path = \"descriptor_privacy/constructor_retirement.rs\"]",
            "mod constructor_retirement;",
            "#[path = \"descriptor_privacy/private_fields.rs\"]",
            "mod private_fields;",
            "#[path = \"descriptor_privacy/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );
    assert_eq!(
        sources.descriptor_privacy.matches("#[test]").count(),
        0,
        "descriptor_privacy.rs should only mount descriptor privacy review guard owners"
    );
    for child_owned_test in &REVIEW_GUARDS[4..7] {
        assert!(
            !sources
                .descriptor_privacy
                .contains(&format!("fn {child_owned_test}")),
            "child-owned F8 descriptor privacy guard `{child_owned_test}` should not return to descriptor_privacy.rs"
        );
    }
}
