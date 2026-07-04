use super::super::*;
use super::*;

pub(super) fn assert_p0_parent_routes_are_child_owned(sources: &P0RobustnessSources) {
    assert_contains_all(
        "P0 robustness parent mounts focused child owners",
        &sources.parent,
        &[
            "#[path = \"p0_robustness/native_host_callbacks.rs\"]",
            "mod native_host_callbacks;",
            "#[path = \"p0_robustness/lock_poison.rs\"]",
            "mod lock_poison;",
            "#[path = \"p0_robustness/render_submit.rs\"]",
            "mod render_submit;",
            "#[path = \"p0_robustness/native_fixture.rs\"]",
            "mod native_fixture;",
            "#[path = \"p0_robustness/priority_recommendation.rs\"]",
            "mod priority_recommendation;",
        ],
    );
    assert_eq!(
        sources.parent.matches("#[test]").count(),
        0,
        "p0_robustness.rs should only mount child review guard owners"
    );
    for child_owned_test in REVIEW_GUARDS {
        assert!(
            !sources.parent.contains(&format!("fn {child_owned_test}")),
            "child-owned P0 review guard `{child_owned_test}` should not return to p0_robustness.rs"
        );
    }
}
