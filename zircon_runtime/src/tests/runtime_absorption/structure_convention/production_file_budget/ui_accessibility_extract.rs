use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_accessibility_extract_state_is_child_owner() {
    let parent = read_runtime_src("ui/accessibility/extract.rs");
    let state = read_runtime_src("ui/accessibility/extract/state.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");

    assert_contains_all(
        "accessibility extract parent keeps snapshot traversal, relation, role, and action assembly",
        &parent,
        &[
            "mod state;",
            "use state::{",
            "checked_state_for",
            "disabled_state_for",
            "expanded_state_for",
            "pub(crate) fn accessibility_snapshot",
            "fn build_node(",
            "fn resolve_names(",
            "fn resolve_descriptions(",
            "fn filter_children(",
            "fn role_for(",
            "fn actions_for(",
            "fn widget_behavior(",
        ],
    );
    for moved_owner in [
        "fn open_state_for(",
        "fn open_component_state_flag(",
        "fn default_expanded_state(",
        "fn value_attribute_text(",
        "fn component_state_value_text(",
        "fn clamp_text_byte_offset(",
        "UiA11yCheckedState",
        "UiA11yTextSelection",
        "UiValue",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "ui/accessibility/extract.rs should delegate state projection owner `{moved_owner}` to extract/state.rs"
        );
    }
    assert_contains_all(
        "accessibility state child owns state projection and component-state conversion",
        &state,
        &[
            "pub(super) fn expanded_state_for",
            "pub(super) fn disabled_state_for",
            "pub(super) fn selected_state_for",
            "pub(super) fn pressed_state_for",
            "pub(super) fn checked_state_for",
            "pub(super) fn value_state_for",
            "pub(super) fn text_selection_state_for",
            "fn open_state_for(",
            "fn bool_component_state_value(",
            "fn attribute_display_text(",
            "use super::widget_behavior;",
            "ui_surface_effective_disabled",
            "UiValue",
        ],
    );

    for (path, source) in [
        ("ui/accessibility/extract.rs", parent.as_str()),
        ("ui/accessibility/extract/state.rs", state.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

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
                "Runtime 15 M4 UI accessibility extract state owner split",
                "runtime_15_ui_accessibility_extract_state_owner_split_static_passed_cargo_deferred",
                "ui/accessibility/extract.rs",
                "ui/accessibility/extract/state.rs",
                "runtime_15_ui_accessibility_extract_state_is_child_owner",
            ],
        );
    }
}
