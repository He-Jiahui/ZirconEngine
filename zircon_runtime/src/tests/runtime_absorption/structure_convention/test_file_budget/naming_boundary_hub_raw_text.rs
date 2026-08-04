use super::*;

const STATUS: &str =
    "runtime_15_hub_raw_text_policy_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 Hub raw-text policy guard child-owner split";
const GUARD: &str = "runtime_15_hub_raw_text_policy_guard_is_child_owner";

#[test]
fn runtime_15_hub_raw_text_policy_guard_is_child_owner() {
    let parent = read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/hub.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/hub/raw_text_policy.rs",
    );

    assert_contains_all(
        "Runtime 15 Hub naming parent mounts raw-text policy child",
        &parent,
        &[
            "#[path = \"hub/raw_text_policy.rs\"]",
            "mod raw_text_policy;",
        ],
    );
    for retired in [
        "fn runtime_15_hub_message_raw_text_policy_uses_current_names",
        "fn hub_source_files",
        "fn collect_hub_source_files",
        "fn has_legacy_term",
    ] {
        assert!(
            !parent.contains(retired),
            "runtime_15_m2/hub.rs should mount the raw-text child instead of defining `{retired}`"
        );
    }

    assert_contains_all(
        "Runtime 15 Hub raw-text child owns policy guard and scan helpers",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_hub_message_raw_text_policy_uses_current_names",
            "fn hub_source_files",
            "fn collect_hub_source_files",
            "fn has_legacy_term",
            "HubMessage::raw_text",
            "runtime_15_hub_message_raw_text_policy_hard_cutover_static_passed_cargo_deferred",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/hub.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/hub/raw_text_policy.rs",
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
}
