use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_surface_property_mutation_metadata_dirty_is_child_owner() {
    let parent = read_runtime_src("ui/surface/property_mutation.rs");
    let metadata_dirty = read_runtime_src("ui/surface/property_mutation/metadata_dirty.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");

    assert_contains_all(
        "property mutation parent keeps request/report and mutation entry point",
        &parent,
        &[
            "mod metadata_dirty;",
            "use self::metadata_dirty::{metadata_attribute_dirty, render_dirty};",
            "pub struct UiPropertyMutationRequest",
            "pub struct UiPropertyMutationReport",
            "pub fn mutate_tree_property(",
            "fn mutate_node_state_bool(",
            "fn sync_template_attribute_if_present(",
            "fn property_binding_report(",
            "fn visibility_value(",
            "fn input_policy_value(",
            "fn mark_state_dirty(",
            "fn visibility_dirty(",
            "fn input_dirty(",
        ],
    );
    for moved_owner in [
        "fn metadata_attribute_dirty(",
        "fn virtualized_range_dirty(",
        "fn is_mui_customization_attribute(",
        "fn is_overlay_position_attribute(",
        "fn is_overlay_interaction_attribute(",
        "fn is_mui_feedback_attribute(",
        "fn is_transition_attribute(",
        "fn is_mui_overlay_component(",
        "fn is_virtualized_range_attribute(",
        "fn is_render_only_numeric_value_component(",
        "fn is_layout_metadata_attribute(",
        "UiValueKind",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "ui/surface/property_mutation.rs should delegate metadata dirty owner `{moved_owner}` to metadata_dirty.rs"
        );
    }

    assert_contains_all(
        "metadata dirty child owns dirty-domain classification and predicates",
        &metadata_dirty,
        &[
            "pub(super) fn render_dirty(",
            "pub(super) fn metadata_attribute_dirty(",
            "fn virtualized_range_dirty(",
            "fn is_mui_customization_attribute(",
            "fn is_overlay_position_attribute(",
            "fn is_overlay_interaction_attribute(",
            "fn is_mui_feedback_attribute(",
            "fn is_transition_attribute(",
            "fn is_mui_overlay_component(",
            "fn is_virtualized_range_attribute(",
            "fn is_render_only_numeric_value_component(",
            "fn is_layout_metadata_attribute(",
            "UiValueKind",
            "UiDirtyFlags",
        ],
    );

    for (path, source) in [
        ("ui/surface/property_mutation.rs", parent.as_str()),
        (
            "ui/surface/property_mutation/metadata_dirty.rs",
            metadata_dirty.as_str(),
        ),
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
                "Runtime 15 M4 UI surface property mutation metadata dirty owner split",
                "runtime_15_ui_surface_property_mutation_metadata_dirty_owner_split_static_passed_cargo_deferred",
                "ui/surface/property_mutation.rs",
                "ui/surface/property_mutation/metadata_dirty.rs",
                "runtime_15_ui_surface_property_mutation_metadata_dirty_is_child_owner",
            ],
        );
    }
}
