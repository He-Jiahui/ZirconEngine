use super::*;

const PARENT_SOURCE: &str = include_str!("../../ui_hotspot.rs");
const AGGREGATION_TESTS_SOURCE: &str = include_str!("aggregation.rs");
const EVIDENCE_TESTS_SOURCE: &str = include_str!("evidence.rs");
const GPU_ALERTS_TESTS_SOURCE: &str = include_str!("gpu_alerts.rs");
const INTERACTION_ALERTS_TESTS_SOURCE: &str = include_str!("interaction_alerts.rs");
const SUPPORT_TESTS_SOURCE: &str = include_str!("support.rs");

mod support;
use support::counter;

mod aggregation;
mod evidence;
mod gpu_alerts;
mod interaction_alerts;

#[test]
fn ui_hotspot_behavior_tests_are_folder_backed() {
    assert!(
        PARENT_SOURCE.contains("#[path = \"ui_hotspot/tests/mod.rs\"]"),
        "ui_hotspot.rs should route behavior tests through its folder-backed owner"
    );
    for forbidden in ["mod tests {", "include!(\"ui_hotspot/tests/"] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "ui_hotspot.rs should not retain inline behavior test `{forbidden}`"
        );
    }
    for (label, source) in [
        ("ui hotspot aggregation tests", AGGREGATION_TESTS_SOURCE),
        ("ui hotspot evidence tests", EVIDENCE_TESTS_SOURCE),
        ("ui hotspot GPU alert tests", GPU_ALERTS_TESTS_SOURCE),
        (
            "ui hotspot interaction alert tests",
            INTERACTION_ALERTS_TESTS_SOURCE,
        ),
        ("ui hotspot test support", SUPPORT_TESTS_SOURCE),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= 800,
            "{label} has {line_count} lines; expected at most 800"
        );
    }
}
