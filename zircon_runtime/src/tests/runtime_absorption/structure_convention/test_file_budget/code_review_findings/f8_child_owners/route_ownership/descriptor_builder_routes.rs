use super::super::*;
use super::*;

pub(super) fn assert_f8_descriptor_builder_routes_are_child_owned(sources: &F8ReviewSources) {
    assert_contains_all(
        "F8 descriptor builder route mounts focused child owners",
        &sources.descriptor_builder,
        &[
            "#[path = \"descriptor_builder/first_party_descriptors.rs\"]",
            "mod first_party_descriptors;",
            "#[path = \"descriptor_builder/scaffold.rs\"]",
            "mod scaffold;",
            "#[path = \"descriptor_builder/test_fixtures.rs\"]",
            "mod test_fixtures;",
        ],
    );
    assert_eq!(
        sources.descriptor_builder.matches("#[test]").count(),
        0,
        "descriptor_builder.rs should only mount descriptor builder review guard owners"
    );
    for child_owned_test in &REVIEW_GUARDS[1..4] {
        assert!(
            !sources
                .descriptor_builder
                .contains(&format!("fn {child_owned_test}")),
            "child-owned F8 descriptor builder guard `{child_owned_test}` should not return to descriptor_builder.rs"
        );
    }
}
