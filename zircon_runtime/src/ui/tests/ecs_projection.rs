use std::collections::BTreeMap;

use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    ecs::{
        UiEcsDirtyDomainKind, UiEcsDirtyDomains, UiEcsProjectionChangeKind,
        UiEcsProjectionChangeReason,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    pipeline::{UiPipelineDirtyReason, UiPipelineStage},
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn ui_ecs_projection_exposes_runtime_node_dirty_interaction_and_render_facts() {
    let mut surface = projection_surface(false);
    surface.rebuild();
    surface.clear_dirty_flags();
    surface.tree.node_mut(UiNodeId::new(2)).unwrap().dirty.text = true;
    surface.focus.focused = Some(UiNodeId::new(2));
    surface.focus.hovered = vec![UiNodeId::new(1), UiNodeId::new(2)];
    surface.focus.pressed = Some(UiNodeId::new(2));
    surface.focus.captured = Some(UiNodeId::new(2));
    surface
        .component_states
        .set_selected(UiNodeId::new(2), true);

    let projection = surface.ui_ecs_projection();
    let button = projection
        .nodes
        .iter()
        .find(|node| node.node_id == UiNodeId::new(2))
        .expect("button projection");

    assert_eq!(
        projection.tree_id,
        UiTreeId::new("runtime.ui.ecs_projection")
    );
    assert_eq!(projection.roots, vec![UiNodeId::new(1)]);
    assert_eq!(projection.totals.node_count, 2);
    assert_eq!(projection.totals.dirty_node_count, 1);
    assert_eq!(projection.totals.text_dirty_count, 1);
    assert_eq!(projection.totals.accessibility_dirty_count, 1);
    assert_eq!(projection.totals.render_dirty_count, 1);
    assert_eq!(projection.totals.focused_count, 1);
    assert_eq!(projection.totals.hovered_count, 2);
    assert_eq!(projection.totals.pressed_count, 1);
    assert!(projection.totals.render_command_count > 0);
    assert!(projection.totals.hit_entry_count > 0);

    assert_eq!(button.component, "Button");
    assert_eq!(button.control_id.as_deref(), Some("primary"));
    assert_eq!(button.parent, Some(UiNodeId::new(1)));
    assert_eq!(button.frame, UiFrame::new(8.0, 8.0, 80.0, 24.0));
    assert_eq!(
        button.dirty,
        UiEcsDirtyDomains {
            text: true,
            accessibility: true,
            render: true,
            ..UiEcsDirtyDomains::default()
        }
    );
    assert!(button.interaction.visible);
    assert!(button.interaction.enabled);
    assert!(!button.interaction.disabled);
    assert!(button.interaction.focused);
    assert!(button.interaction.hovered);
    assert!(button.interaction.pressed);
    assert!(button.interaction.captured);
    assert!(button.interaction.selected);
    assert!(button.render_command_count > 0);
    assert_eq!(button.hit_entry_count, 1);
}

#[test]
fn ui_ecs_projection_uses_runtime_effective_disabled_gate() {
    let mut surface = projection_surface(true);
    surface.rebuild();

    let projection = surface.ui_ecs_projection();
    let child = projection
        .nodes
        .iter()
        .find(|node| node.node_id == UiNodeId::new(2))
        .expect("child projection");

    assert!(child.interaction.disabled);
    assert!(!child.interaction.enabled);
    assert_eq!(projection.totals.disabled_count, 2);
}

#[test]
fn surface_frame_and_debug_snapshot_carry_ui_ecs_projection() {
    let mut surface = projection_surface(false);
    surface.rebuild();

    let frame = surface.surface_frame();
    let snapshot = surface.debug_snapshot();

    assert_eq!(frame.ecs_projection, surface.ui_ecs_projection());
    assert_eq!(snapshot.ecs_projection, frame.ecs_projection);
    assert_eq!(snapshot.ecs_projection.totals.node_count, 2);
    assert_eq!(
        snapshot.ecs_projection.schedule_mask,
        snapshot.ecs_projection.schedule_mask()
    );
    assert!(frame.ecs_projection.derived_fields_are_fresh());
    assert!(snapshot.ecs_projection.derived_fields_are_fresh());
}

#[test]
fn ui_ecs_projection_delta_reports_runtime_interaction_and_structure_changes() {
    let mut surface = projection_surface(false);
    surface.rebuild();
    let previous = surface.ui_ecs_projection();

    surface.focus.focused = Some(UiNodeId::new(2));
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(8.0, 32.0, 80.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextInput".to_string(),
                    control_id: Some("search".to_string()),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let delta = surface.ui_ecs_projection_delta_from(&previous);
    let focused_button = delta
        .changes
        .iter()
        .find(|change| change.node_id == UiNodeId::new(2))
        .expect("focused button change");
    let added_input = delta
        .changes
        .iter()
        .find(|change| change.node_id == UiNodeId::new(3))
        .expect("added input change");

    assert_eq!(delta.totals.added_count, 1);
    assert!(delta.totals.updated_count >= 1);
    assert_eq!(delta.schedule_mask, delta.schedule_mask());
    assert!(delta.derived_fields_are_fresh());
    assert!(surface
        .ui_ecs_component_structure_change_node_ids_from(&previous)
        .contains(&UiNodeId::new(3)));
    assert!(surface
        .ui_ecs_interaction_change_node_ids_from(&previous)
        .contains(&UiNodeId::new(2)));
    assert_eq!(focused_button.kind, UiEcsProjectionChangeKind::Updated);
    assert!(focused_button
        .reasons
        .contains(&UiEcsProjectionChangeReason::Interaction));
    assert!(focused_button.domains.input);
    assert!(focused_button.domains.accessibility);
    assert!(focused_button.domains.render);

    assert_eq!(added_input.kind, UiEcsProjectionChangeKind::Added);
    assert_eq!(added_input.node_path, UiNodePath::new("root/input"));
    assert!(added_input.domains.layout);
    assert!(added_input.domains.picking);
    assert!(added_input.domains.accessibility);
    assert!(added_input.domains.render);
}

#[test]
fn ui_ecs_projection_delta_keeps_interaction_only_fast_path_out_of_component_structure() {
    let mut surface = projection_surface(false);
    surface.rebuild();
    let previous = surface.ui_ecs_projection();

    surface.focus.focused = Some(UiNodeId::new(2));
    surface.focus.hovered = vec![UiNodeId::new(2)];
    surface.focus.pressed = Some(UiNodeId::new(2));

    let delta = surface.ui_ecs_projection_delta_from(&previous);
    let button_change = delta
        .changes
        .iter()
        .find(|change| change.node_id == UiNodeId::new(2))
        .expect("button interaction change");

    assert_eq!(delta.totals.changed_node_count, 1);
    assert_eq!(delta.totals.component_structure_change_count, 0);
    assert_eq!(delta.totals.interaction_change_count, 1);
    assert!(!delta.component_structure_changed());
    assert!(delta.interaction_only());
    assert_eq!(
        surface.ui_ecs_component_structure_change_node_ids_from(&previous),
        Vec::<UiNodeId>::new()
    );
    assert_eq!(
        surface.ui_ecs_interaction_change_node_ids_from(&previous),
        vec![UiNodeId::new(2)]
    );
    assert_eq!(
        surface.ui_ecs_interaction_only_change_node_ids_from(&previous),
        vec![UiNodeId::new(2)]
    );
    assert_eq!(
        surface.ui_ecs_render_only_change_node_ids_from(&previous),
        Vec::<UiNodeId>::new()
    );
    assert!(button_change.is_interaction_only());
    assert!(!button_change.changes_component_structure());
    assert!(delta.schedule_mask.requires_stage(UiPipelineStage::Focus));
    assert!(delta
        .schedule_mask
        .requires_stage(UiPipelineStage::WidgetBehavior));
    assert!(!delta.schedule_mask.requires_stage(UiPipelineStage::Layout));
    assert!(!delta.schedule_mask.requires_stage(UiPipelineStage::Picking));
}

#[test]
fn ui_ecs_schedule_mask_from_previous_projection_reports_required_pipeline_stages() {
    let mut surface = projection_surface(false);
    surface.rebuild();
    let previous = surface.ui_ecs_projection();

    surface.clear_dirty_flags();
    surface.tree.node_mut(UiNodeId::new(2)).unwrap().dirty.text = true;

    let delta = surface.ui_ecs_projection_delta_from(&previous);
    let mask = surface.ui_ecs_schedule_mask_from(&previous);
    let helper_impacts = surface.ui_ecs_schedule_impacts_from(&previous);
    let helper_domain_impacts = surface.ui_ecs_dirty_domain_impacts_from(&previous);

    assert_eq!(delta.schedule_mask, mask);
    assert!(mask.requires_stage(UiPipelineStage::TextMeasure));
    assert!(mask.requires_stage(UiPipelineStage::Layout));
    assert!(mask.requires_stage(UiPipelineStage::PostLayout));
    assert!(mask.requires_stage(UiPipelineStage::Picking));
    assert!(mask.requires_stage(UiPipelineStage::A11yExtract));
    assert!(mask.requires_stage(UiPipelineStage::RenderExtract));
    assert!(mask.requires_stage(UiPipelineStage::BatchPrepare));
    assert!(!mask.requires_stage(UiPipelineStage::Diagnostics));

    let impacts = delta.schedule_impacts();
    assert_eq!(delta.schedule_impacts, impacts);
    assert_eq!(helper_impacts, impacts);
    assert_eq!(helper_domain_impacts, delta.dirty_domain_impacts());
    assert_eq!(
        delta.node_ids_requiring_stage(UiPipelineStage::TextMeasure),
        vec![UiNodeId::new(2)]
    );
    assert_eq!(
        delta.node_ids_in_dirty_domain(UiEcsDirtyDomainKind::Text),
        vec![UiNodeId::new(2)]
    );
    let text_measure = impacts
        .iter()
        .find(|impact| impact.stage == UiPipelineStage::TextMeasure)
        .expect("text measure impact");
    let render_extract = impacts
        .iter()
        .find(|impact| impact.stage == UiPipelineStage::RenderExtract)
        .expect("render extract impact");

    assert_eq!(text_measure.node_ids, vec![UiNodeId::new(2)]);
    assert_eq!(text_measure.node_count, 1);
    assert_eq!(
        text_measure.dirty_reasons,
        vec![UiPipelineDirtyReason::Text]
    );
    assert_eq!(render_extract.node_ids, vec![UiNodeId::new(2)]);
    assert!(render_extract
        .dirty_reasons
        .contains(&UiPipelineDirtyReason::Render));

    let text_domain = helper_domain_impacts
        .iter()
        .find(|impact| impact.domain == UiEcsDirtyDomainKind::Text)
        .expect("text domain impact");
    let render_domain = helper_domain_impacts
        .iter()
        .find(|impact| impact.domain == UiEcsDirtyDomainKind::Render)
        .expect("render domain impact");

    assert_eq!(text_domain.node_ids, vec![UiNodeId::new(2)]);
    assert_eq!(text_domain.node_count, 1);
    assert_eq!(render_domain.node_ids, vec![UiNodeId::new(2)]);
}

fn projection_surface(disabled_root: bool) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.ecs_projection"));
    let mut root_attributes = BTreeMap::new();
    if disabled_root {
        root_attributes.insert("disabled".to_string(), toml::Value::Boolean(true));
    }
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 40.0))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Panel".to_string(),
                control_id: Some("root".to_string()),
                attributes: root_attributes,
                ..UiTemplateNodeMetadata::default()
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("primary".to_string()),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
}
