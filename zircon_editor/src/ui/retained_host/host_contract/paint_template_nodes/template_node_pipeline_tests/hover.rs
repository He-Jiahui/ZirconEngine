use super::super::super::super::data::HostPaneInteractionStateData;
use super::super::hover::apply_template_hover_to_node;
use super::support::dropdown_node;

#[test]
fn transient_hover_updates_only_the_owned_node_used_for_paint() {
    let source = dropdown_node();
    let mut painted = source.clone();
    let interaction = HostPaneInteractionStateData {
        hovered_template_control_id: "Dropdown".into(),
        hovered_template_dispatch_kind: "workbench_option".into(),
        hovered_template_value_text: "dropdown".into(),
        ..HostPaneInteractionStateData::default()
    };

    apply_template_hover_to_node(&mut painted, &interaction);

    assert!(painted.hovered);
    assert!(painted
        .structured_options
        .get(0)
        .is_some_and(|option| option.hovered));
    assert!(!source.hovered);
    assert!(source
        .structured_options
        .get(0)
        .is_some_and(|option| !option.hovered));
}

#[test]
fn transient_hover_ignores_non_matching_nodes() {
    let mut node = dropdown_node();
    let before_hovered = node.hovered;
    let before_options = node.structured_options.clone();
    let interaction = HostPaneInteractionStateData {
        hovered_template_control_id: "AnotherControl".into(),
        ..HostPaneInteractionStateData::default()
    };

    apply_template_hover_to_node(&mut node, &interaction);

    assert_eq!(node.hovered, before_hovered);
    assert!(node.structured_options.shares_values_with(&before_options));
}

#[test]
fn presentation_snapshot_does_not_materialize_hover_into_full_node_models() {
    let snapshot = include_str!("../../window/presentation/snapshot.rs");
    let paint_pipeline = include_str!("../template_node_pipeline/draw.rs");

    assert!(!snapshot.contains("apply_template_hover_to_presentation"));
    assert!(paint_pipeline.contains("apply_template_hover_to_node"));
}
