use crate::ui::{
    ecs::{
        UiEcsDirtyDomainImpact, UiEcsDirtyDomainKind, UiEcsDirtyDomains, UiEcsInteractionState,
        UiEcsNodeProjection, UiEcsProjectionChangeKind, UiEcsProjectionChangeReason,
        UiEcsProjectionDelta, UiEcsProjectionDeltaTotals, UiEcsProjectionScheduleImpact,
        UiEcsProjectionScheduleMask, UiEcsProjectionSnapshot, UiEcsProjectionTotals,
    },
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiFrame,
    pipeline::{UiPipelineDirtyReason, UiPipelineStage},
    tree::UiDirtyFlags,
};

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn ui_ecs_dirty_domains_derive_schedule_visible_domains_from_dirty_flags() {
    let dirty = UiDirtyFlags {
        layout: true,
        hit_test: false,
        render: false,
        style: false,
        text: true,
        input: false,
        visible_range: false,
    };

    let domains = UiEcsDirtyDomains::from_dirty_flags(dirty);

    assert!(domains.layout);
    assert!(domains.text);
    assert!(domains.picking);
    assert!(domains.accessibility);
    assert!(domains.render);
    assert!(!domains.input);
    assert!(domains.any());
    assert_eq!(round_trip(&domains), domains);
}

#[test]
fn ui_ecs_projection_snapshot_roundtrips_and_recomputes_totals() {
    let nodes = vec![
        UiEcsNodeProjection {
            node_id: UiNodeId::new(1),
            node_path: UiNodePath::new("root"),
            children: vec![UiNodeId::new(2)],
            frame: UiFrame::new(0.0, 0.0, 120.0, 40.0),
            render_command_count: 1,
            hit_entry_count: 0,
            ..UiEcsNodeProjection::default()
        },
        UiEcsNodeProjection {
            node_id: UiNodeId::new(2),
            node_path: UiNodePath::new("root/button"),
            parent: Some(UiNodeId::new(1)),
            component: "Button".to_string(),
            control_id: Some("primary".to_string()),
            frame: UiFrame::new(8.0, 8.0, 80.0, 24.0),
            dirty: UiEcsDirtyDomains {
                input: true,
                picking: true,
                accessibility: true,
                render: true,
                ..UiEcsDirtyDomains::default()
            },
            interaction: UiEcsInteractionState {
                visible: true,
                enabled: true,
                focused: true,
                hovered: true,
                pressed: true,
                focusable: true,
                clickable: true,
                ..UiEcsInteractionState::default()
            },
            render_command_count: 2,
            hit_entry_count: 1,
            ..UiEcsNodeProjection::default()
        },
    ];

    let snapshot =
        UiEcsProjectionSnapshot::from_nodes(UiTreeId::new("ui.ecs"), vec![UiNodeId::new(1)], nodes);

    assert_eq!(snapshot.totals.node_count, 2);
    assert_eq!(snapshot.totals.dirty_node_count, 1);
    assert_eq!(snapshot.totals.input_dirty_count, 1);
    assert_eq!(snapshot.totals.picking_dirty_count, 1);
    assert_eq!(snapshot.totals.accessibility_dirty_count, 1);
    assert_eq!(snapshot.totals.render_dirty_count, 1);
    assert_eq!(snapshot.totals.focused_count, 1);
    assert_eq!(snapshot.totals.hovered_count, 1);
    assert_eq!(snapshot.totals.pressed_count, 1);
    assert_eq!(snapshot.totals.render_command_count, 3);
    assert_eq!(snapshot.totals.hit_entry_count, 1);
    assert_eq!(
        UiEcsProjectionTotals::from_nodes(&snapshot.nodes),
        snapshot.totals
    );
    assert_eq!(snapshot.schedule_mask, snapshot.schedule_mask());
    assert_eq!(snapshot.schedule_impacts, snapshot.schedule_impacts());
    assert_eq!(
        snapshot.dirty_domain_impacts,
        snapshot.dirty_domain_impacts()
    );
    assert!(snapshot.derived_fields_are_fresh());
    assert!(snapshot
        .schedule_mask
        .requires_stage(UiPipelineStage::InputCollect));
    assert!(snapshot
        .schedule_mask
        .requires_stage(UiPipelineStage::WidgetBehavior));
    assert!(snapshot
        .schedule_mask
        .requires_stage(UiPipelineStage::A11yExtract));
    assert!(snapshot
        .schedule_mask
        .requires_stage(UiPipelineStage::RenderExtract));
    assert_eq!(round_trip(&snapshot), snapshot);
}

#[test]
fn ui_ecs_projection_legacy_payload_defaults_totals_and_nodes() {
    let snapshot: UiEcsProjectionSnapshot = serde_json::from_str(
        r#"{
            "tree_id": "legacy.ui"
        }"#,
    )
    .unwrap();

    assert_eq!(snapshot.tree_id, UiTreeId::new("legacy.ui"));
    assert!(snapshot.roots.is_empty());
    assert!(snapshot.nodes.is_empty());
    assert_eq!(snapshot.totals, UiEcsProjectionTotals::default());
    assert_eq!(
        snapshot.schedule_mask,
        UiEcsProjectionScheduleMask::default()
    );
    assert!(snapshot.schedule_impacts.is_empty());
    assert!(snapshot.dirty_domain_impacts.is_empty());
}

#[test]
fn ui_ecs_projection_legacy_payload_without_derived_fields_can_recompute_from_nodes() {
    let snapshot = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("legacy.ui"),
        vec![UiNodeId::new(1)],
        vec![UiEcsNodeProjection {
            node_id: UiNodeId::new(1),
            node_path: UiNodePath::new("root/input"),
            dirty: UiEcsDirtyDomains {
                text: true,
                accessibility: true,
                render: true,
                ..UiEcsDirtyDomains::default()
            },
            ..UiEcsNodeProjection::default()
        }],
    );
    let expected_totals = snapshot.totals;
    let expected_mask = snapshot.schedule_mask;
    let expected_impacts = snapshot.schedule_impacts.clone();
    let expected_domain_impacts = snapshot.dirty_domain_impacts.clone();
    let mut payload = serde_json::to_value(&snapshot).unwrap();
    payload.as_object_mut().unwrap().remove("totals");
    payload.as_object_mut().unwrap().remove("schedule_mask");
    payload.as_object_mut().unwrap().remove("schedule_impacts");
    payload
        .as_object_mut()
        .unwrap()
        .remove("dirty_domain_impacts");

    let mut recovered: UiEcsProjectionSnapshot = serde_json::from_value(payload).unwrap();

    assert_eq!(recovered.totals, UiEcsProjectionTotals::default());
    assert_eq!(
        recovered.schedule_mask,
        UiEcsProjectionScheduleMask::default()
    );
    assert!(recovered.schedule_impacts.is_empty());
    assert!(recovered.dirty_domain_impacts.is_empty());
    assert!(!recovered.derived_fields_are_fresh());
    assert_eq!(recovered.schedule_mask(), expected_mask);
    assert_eq!(recovered.schedule_impacts(), expected_impacts);
    assert_eq!(recovered.dirty_domain_impacts(), expected_domain_impacts);
    assert!(recovered
        .clone()
        .with_recomputed_derived_fields()
        .derived_fields_are_fresh());
    recovered.recompute_derived_fields();
    assert_eq!(recovered.totals, expected_totals);
    assert_eq!(recovered.schedule_mask, expected_mask);
    assert_eq!(recovered.schedule_impacts, expected_impacts);
    assert_eq!(recovered.dirty_domain_impacts, expected_domain_impacts);
    assert!(recovered.derived_fields_are_fresh());
    assert!(recovered
        .schedule_mask
        .requires_stage(UiPipelineStage::TextMeasure));
}

#[test]
fn ui_ecs_projection_delta_reports_added_removed_and_updated_domains() {
    let previous = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        vec![UiNodeId::new(1)],
        vec![
            UiEcsNodeProjection {
                node_id: UiNodeId::new(1),
                node_path: UiNodePath::new("root"),
                children: vec![UiNodeId::new(2)],
                frame: UiFrame::new(0.0, 0.0, 120.0, 40.0),
                ..UiEcsNodeProjection::default()
            },
            UiEcsNodeProjection {
                node_id: UiNodeId::new(2),
                node_path: UiNodePath::new("root/button"),
                parent: Some(UiNodeId::new(1)),
                component: "Button".to_string(),
                frame: UiFrame::new(8.0, 8.0, 80.0, 24.0),
                ..UiEcsNodeProjection::default()
            },
        ],
    );
    let current = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        vec![UiNodeId::new(1)],
        vec![
            UiEcsNodeProjection {
                node_id: UiNodeId::new(1),
                node_path: UiNodePath::new("root"),
                children: vec![UiNodeId::new(3)],
                frame: UiFrame::new(0.0, 0.0, 120.0, 40.0),
                ..UiEcsNodeProjection::default()
            },
            UiEcsNodeProjection {
                node_id: UiNodeId::new(3),
                node_path: UiNodePath::new("root/input"),
                parent: Some(UiNodeId::new(1)),
                component: "TextInput".to_string(),
                frame: UiFrame::new(8.0, 8.0, 96.0, 24.0),
                dirty: UiEcsDirtyDomains {
                    text: true,
                    accessibility: true,
                    render: true,
                    ..UiEcsDirtyDomains::default()
                },
                interaction: UiEcsInteractionState {
                    visible: true,
                    enabled: true,
                    focused: true,
                    ..UiEcsInteractionState::default()
                },
                ..UiEcsNodeProjection::default()
            },
        ],
    );

    let delta = current.diff_from(&previous);
    let root_change = delta
        .changes
        .iter()
        .find(|change| change.node_id == UiNodeId::new(1))
        .expect("root child-list change");
    let removed = delta
        .changes
        .iter()
        .find(|change| change.node_id == UiNodeId::new(2))
        .expect("removed button");
    let added = delta
        .changes
        .iter()
        .find(|change| change.node_id == UiNodeId::new(3))
        .expect("added text input");

    assert_eq!(delta.previous_tree_id, UiTreeId::new("ui.ecs"));
    assert_eq!(delta.current_tree_id, UiTreeId::new("ui.ecs"));
    assert_eq!(delta.totals.changed_node_count, 3);
    assert_eq!(delta.totals.added_count, 1);
    assert_eq!(delta.totals.removed_count, 1);
    assert_eq!(delta.totals.updated_count, 1);
    assert_eq!(delta.totals.component_structure_change_count, 3);
    assert_eq!(delta.totals.interaction_change_count, 0);
    assert_eq!(delta.totals.render_only_change_count, 0);
    assert_eq!(
        delta.component_structure_change_node_ids(),
        vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert_eq!(delta.interaction_change_node_ids(), Vec::<UiNodeId>::new());
    assert_eq!(
        delta.interaction_only_change_node_ids(),
        Vec::<UiNodeId>::new()
    );
    assert_eq!(delta.render_only_change_node_ids(), Vec::<UiNodeId>::new());
    assert_eq!(
        UiEcsProjectionDeltaTotals::from_changes(&delta.changes),
        delta.totals
    );
    assert_eq!(
        delta
            .change(UiNodeId::new(3))
            .expect("added node change")
            .kind,
        UiEcsProjectionChangeKind::Added
    );
    assert!(delta.change(UiNodeId::new(99)).is_none());
    assert_eq!(
        delta.node_ids_by_change_kind(UiEcsProjectionChangeKind::Added),
        vec![UiNodeId::new(3)]
    );
    assert_eq!(
        delta.node_ids_by_change_kind(UiEcsProjectionChangeKind::Removed),
        vec![UiNodeId::new(2)]
    );
    assert_eq!(
        delta
            .changes_by_kind(UiEcsProjectionChangeKind::Updated)
            .into_iter()
            .map(|change| change.node_id)
            .collect::<Vec<_>>(),
        vec![UiNodeId::new(1)]
    );
    assert_eq!(delta.schedule_mask, delta.schedule_mask());
    assert_eq!(delta.schedule_impacts, delta.schedule_impacts());
    assert_eq!(delta.dirty_domain_impacts, delta.dirty_domain_impacts());
    assert!(delta.derived_fields_are_fresh());
    assert!(delta.component_structure_changed());
    assert!(!delta.interaction_only());
    assert!(delta
        .schedule_mask
        .requires_stage(UiPipelineStage::TextMeasure));
    assert!(delta.schedule_mask.requires_stage(UiPipelineStage::Layout));

    assert_eq!(root_change.kind, UiEcsProjectionChangeKind::Updated);
    assert!(root_change.changes_component_structure());
    assert!(!root_change.is_interaction_only());
    assert_eq!(
        root_change.reasons,
        vec![UiEcsProjectionChangeReason::Children]
    );
    assert!(root_change.domains.layout);
    assert!(root_change.domains.picking);
    assert!(root_change.domains.accessibility);
    assert!(root_change.domains.render);

    assert_eq!(removed.kind, UiEcsProjectionChangeKind::Removed);
    assert!(removed.changes_component_structure());
    assert_eq!(removed.node_path, UiNodePath::new("root/button"));
    assert_eq!(removed.reasons, vec![UiEcsProjectionChangeReason::Removed]);
    assert!(removed.domains.layout);
    assert!(removed.domains.picking);

    assert_eq!(added.kind, UiEcsProjectionChangeKind::Added);
    assert!(added.changes_component_structure());
    assert_eq!(added.node_path, UiNodePath::new("root/input"));
    assert_eq!(added.reasons, vec![UiEcsProjectionChangeReason::Added]);
    assert!(added.domains.layout);
    assert!(added.domains.text);
    assert!(added.domains.accessibility);
    assert!(round_trip(&delta).totals.render_dirty_count >= 3);

    let impacts = delta.schedule_impacts();
    assert_eq!(delta.schedule_impacts, impacts);
    let text_measure = impacts
        .iter()
        .find(|impact| impact.stage == UiPipelineStage::TextMeasure)
        .expect("text measure impact");
    let layout = impacts
        .iter()
        .find(|impact| impact.stage == UiPipelineStage::Layout)
        .expect("layout impact");

    assert_eq!(text_measure.node_ids, vec![UiNodeId::new(3)]);
    assert_eq!(text_measure.node_count, 1);
    assert_eq!(
        text_measure.dirty_reasons,
        vec![UiPipelineDirtyReason::Text]
    );
    assert_eq!(
        layout.node_ids,
        vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert_eq!(layout.node_count, 3);
    assert_eq!(
        layout.dirty_reasons,
        vec![UiPipelineDirtyReason::Text, UiPipelineDirtyReason::Layout]
    );

    let domain_impacts = delta.dirty_domain_impacts();
    assert_eq!(delta.dirty_domain_impacts, domain_impacts);
    assert_eq!(
        domain_impacts
            .iter()
            .find(|impact| impact.domain == UiEcsDirtyDomainKind::Text)
            .expect("text domain impact")
            .node_ids,
        vec![UiNodeId::new(3)]
    );
    assert_eq!(
        domain_impacts
            .iter()
            .find(|impact| impact.domain == UiEcsDirtyDomainKind::Layout)
            .expect("layout domain impact")
            .node_ids,
        vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert_eq!(
        delta.node_ids_requiring_stage(UiPipelineStage::Layout),
        vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert_eq!(
        delta
            .changes_requiring_stage(UiPipelineStage::Layout)
            .into_iter()
            .map(|change| change.node_id)
            .collect::<Vec<_>>(),
        vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert_eq!(
        delta
            .schedule_impact(UiPipelineStage::TextMeasure)
            .expect("text query impact")
            .node_ids,
        vec![UiNodeId::new(3)]
    );
    assert_eq!(
        delta.node_ids_in_dirty_domain(UiEcsDirtyDomainKind::Text),
        vec![UiNodeId::new(3)]
    );
    assert_eq!(
        delta
            .changes_in_dirty_domain(UiEcsDirtyDomainKind::Text)
            .into_iter()
            .map(|change| change.node_id)
            .collect::<Vec<_>>(),
        vec![UiNodeId::new(3)]
    );
    assert_eq!(
        delta.node_ids_in_dirty_domain(UiEcsDirtyDomainKind::Style),
        Vec::<UiNodeId>::new()
    );
}

#[test]
fn ui_ecs_projection_delta_classifies_interaction_fast_path_without_component_structure() {
    let previous = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        vec![UiNodeId::new(1)],
        vec![UiEcsNodeProjection {
            node_id: UiNodeId::new(1),
            node_path: UiNodePath::new("root/button"),
            component: "Button".to_string(),
            interaction: UiEcsInteractionState {
                visible: true,
                enabled: true,
                ..UiEcsInteractionState::default()
            },
            ..UiEcsNodeProjection::default()
        }],
    );
    let current = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        vec![UiNodeId::new(1)],
        vec![UiEcsNodeProjection {
            node_id: UiNodeId::new(1),
            node_path: UiNodePath::new("root/button"),
            component: "Button".to_string(),
            interaction: UiEcsInteractionState {
                visible: true,
                enabled: true,
                focused: true,
                hovered: true,
                pressed: true,
                ..UiEcsInteractionState::default()
            },
            ..UiEcsNodeProjection::default()
        }],
    );

    let delta = current.diff_from(&previous);
    let change = delta.changes.first().expect("interaction change");

    assert_eq!(delta.totals.changed_node_count, 1);
    assert_eq!(delta.totals.component_structure_change_count, 0);
    assert_eq!(delta.totals.interaction_change_count, 1);
    assert_eq!(delta.totals.render_only_change_count, 0);
    assert!(!delta.component_structure_changed());
    assert!(delta.interaction_only());
    assert_eq!(
        delta.component_structure_change_node_ids(),
        Vec::<UiNodeId>::new()
    );
    assert_eq!(delta.interaction_change_node_ids(), vec![UiNodeId::new(1)]);
    assert_eq!(
        delta.interaction_only_change_node_ids(),
        vec![UiNodeId::new(1)]
    );
    assert_eq!(delta.render_only_change_node_ids(), Vec::<UiNodeId>::new());
    assert!(change.is_interaction_change());
    assert!(change.is_interaction_only());
    assert!(!change.changes_component_structure());
    assert!(delta
        .schedule_mask
        .requires_stage(UiPipelineStage::InputCollect));
    assert!(delta.schedule_mask.requires_stage(UiPipelineStage::Focus));
    assert!(delta
        .schedule_mask
        .requires_stage(UiPipelineStage::WidgetBehavior));
    assert!(delta
        .schedule_mask
        .requires_stage(UiPipelineStage::A11yExtract));
    assert!(delta
        .schedule_mask
        .requires_stage(UiPipelineStage::RenderExtract));
    assert!(!delta
        .schedule_mask
        .requires_stage(UiPipelineStage::TextMeasure));
    assert!(!delta.schedule_mask.requires_stage(UiPipelineStage::Layout));
    assert!(!delta.schedule_mask.requires_stage(UiPipelineStage::Picking));
}

#[test]
fn ui_ecs_projection_delta_classifies_render_only_fast_path_without_layout_or_interaction() {
    let previous = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        vec![UiNodeId::new(1)],
        vec![UiEcsNodeProjection {
            node_id: UiNodeId::new(1),
            node_path: UiNodePath::new("root/label"),
            component: "Label".to_string(),
            render_command_count: 1,
            ..UiEcsNodeProjection::default()
        }],
    );
    let current = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        vec![UiNodeId::new(1)],
        vec![UiEcsNodeProjection {
            node_id: UiNodeId::new(1),
            node_path: UiNodePath::new("root/label"),
            component: "Label".to_string(),
            render_command_count: 2,
            ..UiEcsNodeProjection::default()
        }],
    );

    let delta = current.diff_from(&previous);
    let change = delta.changes.first().expect("render-only change");

    assert_eq!(delta.totals.changed_node_count, 1);
    assert_eq!(delta.totals.component_structure_change_count, 0);
    assert_eq!(delta.totals.interaction_change_count, 0);
    assert_eq!(delta.totals.render_only_change_count, 1);
    assert!(!delta.component_structure_changed());
    assert!(!delta.interaction_only());
    assert_eq!(
        delta.component_structure_change_node_ids(),
        Vec::<UiNodeId>::new()
    );
    assert_eq!(delta.interaction_change_node_ids(), Vec::<UiNodeId>::new());
    assert_eq!(
        delta.interaction_only_change_node_ids(),
        Vec::<UiNodeId>::new()
    );
    assert_eq!(delta.render_only_change_node_ids(), vec![UiNodeId::new(1)]);
    assert_eq!(
        change.reasons,
        vec![UiEcsProjectionChangeReason::RenderCommandCount]
    );
    assert!(change.is_render_only_change());
    assert!(!change.is_interaction_change());
    assert!(!change.changes_component_structure());
    assert!(delta
        .schedule_mask
        .requires_stage(UiPipelineStage::RenderExtract));
    assert!(delta
        .schedule_mask
        .requires_stage(UiPipelineStage::BatchPrepare));
    assert!(!delta
        .schedule_mask
        .requires_stage(UiPipelineStage::InputCollect));
    assert!(!delta.schedule_mask.requires_stage(UiPipelineStage::Focus));
    assert!(!delta
        .schedule_mask
        .requires_stage(UiPipelineStage::WidgetBehavior));
    assert!(!delta
        .schedule_mask
        .requires_stage(UiPipelineStage::TextMeasure));
    assert!(!delta.schedule_mask.requires_stage(UiPipelineStage::Layout));
    assert!(!delta.schedule_mask.requires_stage(UiPipelineStage::Picking));
    assert!(!delta
        .schedule_mask
        .requires_stage(UiPipelineStage::A11yExtract));
}

#[test]
fn ui_ecs_projection_delta_legacy_payload_defaults() {
    let delta: UiEcsProjectionDelta = serde_json::from_str("{}").unwrap();

    assert!(delta.is_empty());
    assert!(delta.changes.is_empty());
    assert_eq!(delta.totals, UiEcsProjectionDeltaTotals::default());
    assert_eq!(delta.schedule_mask, UiEcsProjectionScheduleMask::default());
    assert!(delta.schedule_impacts.is_empty());
    assert!(delta.dirty_domain_impacts.is_empty());
}

#[test]
fn ui_ecs_projection_delta_legacy_payload_without_derived_fields_can_recompute_from_changes() {
    let previous = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("legacy.ui"),
        vec![UiNodeId::new(1)],
        vec![UiEcsNodeProjection {
            node_id: UiNodeId::new(1),
            node_path: UiNodePath::new("root"),
            frame: UiFrame::new(0.0, 0.0, 120.0, 40.0),
            ..UiEcsNodeProjection::default()
        }],
    );
    let current = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("legacy.ui"),
        vec![UiNodeId::new(1)],
        vec![UiEcsNodeProjection {
            node_id: UiNodeId::new(1),
            node_path: UiNodePath::new("root"),
            frame: UiFrame::new(0.0, 0.0, 128.0, 40.0),
            ..UiEcsNodeProjection::default()
        }],
    );
    let delta = current.diff_from(&previous);
    let expected_totals = delta.totals;
    let expected_mask = delta.schedule_mask;
    let expected_impacts = delta.schedule_impacts.clone();
    let expected_domain_impacts = delta.dirty_domain_impacts.clone();
    let mut payload = serde_json::to_value(&delta).unwrap();
    payload.as_object_mut().unwrap().remove("totals");
    payload.as_object_mut().unwrap().remove("schedule_mask");
    payload.as_object_mut().unwrap().remove("schedule_impacts");
    payload
        .as_object_mut()
        .unwrap()
        .remove("dirty_domain_impacts");

    let mut recovered: UiEcsProjectionDelta = serde_json::from_value(payload).unwrap();

    assert_eq!(recovered.totals, UiEcsProjectionDeltaTotals::default());
    assert_eq!(
        recovered.schedule_mask,
        UiEcsProjectionScheduleMask::default()
    );
    assert!(recovered.schedule_impacts.is_empty());
    assert!(recovered.dirty_domain_impacts.is_empty());
    assert!(!recovered.derived_fields_are_fresh());
    assert_eq!(recovered.schedule_mask(), expected_mask);
    assert_eq!(recovered.schedule_impacts(), expected_impacts);
    assert_eq!(recovered.dirty_domain_impacts(), expected_domain_impacts);
    assert!(recovered
        .clone()
        .with_recomputed_derived_fields()
        .derived_fields_are_fresh());
    recovered.recompute_derived_fields();
    assert_eq!(recovered.totals, expected_totals);
    assert_eq!(recovered.schedule_mask, expected_mask);
    assert_eq!(recovered.schedule_impacts, expected_impacts);
    assert_eq!(recovered.dirty_domain_impacts, expected_domain_impacts);
    assert!(recovered.derived_fields_are_fresh());
    assert!(recovered
        .schedule_mask
        .requires_stage(UiPipelineStage::Layout));
}

#[test]
fn ui_ecs_projection_schedule_mask_maps_domains_to_ordered_pipeline_stages() {
    let mask = UiEcsProjectionScheduleMask::from_dirty_domains(UiEcsDirtyDomains {
        text: true,
        accessibility: true,
        render: true,
        ..UiEcsDirtyDomains::default()
    });

    assert!(!mask.is_empty());
    assert_eq!(
        mask.pipeline_stages(),
        vec![
            UiPipelineStage::TextMeasure,
            UiPipelineStage::Layout,
            UiPipelineStage::PostLayout,
            UiPipelineStage::Picking,
            UiPipelineStage::A11yExtract,
            UiPipelineStage::RenderExtract,
            UiPipelineStage::BatchPrepare,
        ]
    );
    assert_eq!(
        mask.dirty_reasons(),
        vec![
            UiPipelineDirtyReason::Text,
            UiPipelineDirtyReason::Layout,
            UiPipelineDirtyReason::Picking,
            UiPipelineDirtyReason::A11y,
            UiPipelineDirtyReason::Render,
        ]
    );
    assert_eq!(round_trip(&mask), mask);
}

#[test]
fn ui_ecs_projection_snapshot_schedule_impacts_group_dirty_nodes_by_stage() {
    let snapshot = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        vec![UiNodeId::new(1)],
        vec![
            UiEcsNodeProjection {
                node_id: UiNodeId::new(1),
                node_path: UiNodePath::new("root"),
                children: vec![UiNodeId::new(2), UiNodeId::new(3)],
                ..UiEcsNodeProjection::default()
            },
            UiEcsNodeProjection {
                node_id: UiNodeId::new(2),
                node_path: UiNodePath::new("root/button"),
                parent: Some(UiNodeId::new(1)),
                dirty: UiEcsDirtyDomains {
                    input: true,
                    picking: true,
                    accessibility: true,
                    render: true,
                    ..UiEcsDirtyDomains::default()
                },
                ..UiEcsNodeProjection::default()
            },
            UiEcsNodeProjection {
                node_id: UiNodeId::new(3),
                node_path: UiNodePath::new("root/text"),
                parent: Some(UiNodeId::new(1)),
                dirty: UiEcsDirtyDomains {
                    text: true,
                    accessibility: true,
                    render: true,
                    ..UiEcsDirtyDomains::default()
                },
                ..UiEcsNodeProjection::default()
            },
        ],
    );

    let impacts = snapshot.schedule_impacts();
    assert_eq!(snapshot.schedule_impacts, impacts);
    let domain_impacts = snapshot.dirty_domain_impacts();
    assert_eq!(snapshot.dirty_domain_impacts, domain_impacts);
    let input_collect = impacts
        .iter()
        .find(|impact| impact.stage == UiPipelineStage::InputCollect)
        .expect("input collect impact");
    let text_measure = impacts
        .iter()
        .find(|impact| impact.stage == UiPipelineStage::TextMeasure)
        .expect("text measure impact");
    let render_extract = impacts
        .iter()
        .find(|impact| impact.stage == UiPipelineStage::RenderExtract)
        .expect("render extract impact");

    assert!(input_collect.required);
    assert_eq!(input_collect.node_ids, vec![UiNodeId::new(2)]);
    assert_eq!(
        input_collect.dirty_reasons,
        vec![UiPipelineDirtyReason::Input]
    );
    assert_eq!(text_measure.node_ids, vec![UiNodeId::new(3)]);
    assert_eq!(
        text_measure.dirty_reasons,
        vec![UiPipelineDirtyReason::Text]
    );
    assert_eq!(
        render_extract.node_ids,
        vec![UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert_eq!(
        render_extract.dirty_reasons,
        vec![
            UiPipelineDirtyReason::Text,
            UiPipelineDirtyReason::Layout,
            UiPipelineDirtyReason::Render,
        ]
    );
    assert_eq!(round_trip(render_extract), *render_extract);
    assert_eq!(
        domain_impacts
            .iter()
            .find(|impact| impact.domain == UiEcsDirtyDomainKind::Input)
            .expect("input domain impact")
            .node_ids,
        vec![UiNodeId::new(2)]
    );
    assert_eq!(
        domain_impacts
            .iter()
            .find(|impact| impact.domain == UiEcsDirtyDomainKind::Text)
            .expect("text domain impact")
            .node_ids,
        vec![UiNodeId::new(3)]
    );
    assert_eq!(
        domain_impacts
            .iter()
            .find(|impact| impact.domain == UiEcsDirtyDomainKind::Render)
            .expect("render domain impact")
            .node_ids,
        vec![UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert_eq!(
        snapshot.node_ids_requiring_stage(UiPipelineStage::TextMeasure),
        vec![UiNodeId::new(3)]
    );
    assert_eq!(
        snapshot
            .schedule_impact(UiPipelineStage::Picking)
            .expect("picking query impact")
            .node_ids,
        vec![UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert_eq!(
        snapshot.node_ids_requiring_stage(UiPipelineStage::Diagnostics),
        Vec::<UiNodeId>::new()
    );
    assert_eq!(
        snapshot.node_ids_in_dirty_domain(UiEcsDirtyDomainKind::Input),
        vec![UiNodeId::new(2)]
    );
    assert!(snapshot
        .dirty_domain_impact(UiEcsDirtyDomainKind::Layout)
        .is_none());
}

#[test]
fn ui_ecs_projection_schedule_impact_legacy_payload_defaults() {
    let impact: UiEcsProjectionScheduleImpact = serde_json::from_str("{}").unwrap();

    assert_eq!(impact.stage, UiPipelineStage::InputCollect);
    assert!(!impact.required);
    assert!(impact.dirty_reasons.is_empty());
    assert!(impact.node_ids.is_empty());
    assert_eq!(impact.node_count, 0);
}

#[test]
fn ui_ecs_projection_dirty_domain_impact_legacy_payload_defaults() {
    let impact: UiEcsDirtyDomainImpact = serde_json::from_str("{}").unwrap();

    assert_eq!(impact.domain, UiEcsDirtyDomainKind::Layout);
    assert!(!impact.active);
    assert!(impact.node_ids.is_empty());
    assert_eq!(impact.node_count, 0);
}

#[test]
fn ui_ecs_projection_delta_exposes_schedule_mask() {
    let previous = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        vec![UiNodeId::new(1)],
        vec![UiEcsNodeProjection {
            node_id: UiNodeId::new(1),
            node_path: UiNodePath::new("root"),
            frame: UiFrame::new(0.0, 0.0, 120.0, 40.0),
            ..UiEcsNodeProjection::default()
        }],
    );
    let current = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        vec![UiNodeId::new(1)],
        vec![UiEcsNodeProjection {
            node_id: UiNodeId::new(1),
            node_path: UiNodePath::new("root"),
            frame: UiFrame::new(0.0, 0.0, 128.0, 40.0),
            interaction: UiEcsInteractionState {
                visible: true,
                enabled: true,
                focused: true,
                ..UiEcsInteractionState::default()
            },
            ..UiEcsNodeProjection::default()
        }],
    );

    let mask = current.diff_from(&previous).schedule_mask();
    let delta = current.diff_from(&previous);

    assert_eq!(delta.schedule_mask, mask);
    assert!(mask.requires_stage(UiPipelineStage::Focus));
    assert!(mask.requires_stage(UiPipelineStage::Layout));
    assert!(mask.requires_stage(UiPipelineStage::PostLayout));
    assert!(mask.requires_stage(UiPipelineStage::Picking));
    assert!(mask.requires_stage(UiPipelineStage::A11yExtract));
    assert!(mask.requires_stage(UiPipelineStage::RenderExtract));
    assert!(mask.requires_stage(UiPipelineStage::BatchPrepare));
    assert!(!mask.requires_stage(UiPipelineStage::Diagnostics));
}
