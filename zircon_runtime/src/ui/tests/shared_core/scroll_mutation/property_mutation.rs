use super::*;

#[test]
fn surface_property_mutation_marks_dirty_only_when_values_change() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.reflector"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_state_flags(pointer_state())
            .with_input_policy(UiInputPolicy::Receive),
    );

    let unchanged = surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(1),
            "enabled",
            UiValue::Bool(true),
        ))
        .unwrap();
    assert_eq!(unchanged.status, UiPropertyMutationStatus::Unchanged);
    assert_eq!(unchanged.binding.unchanged_count, 1);
    assert_eq!(
        unchanged.binding.updates[0].source.kind,
        UiBindingSourceKind::RuntimeState
    );
    assert_eq!(
        unchanged.binding.updates[0].target.kind,
        UiBindingTargetKind::RetainedAttribute
    );
    assert!(!surface.tree.node(UiNodeId::new(1)).unwrap().dirty.any());

    let changed = surface
        .mutate_property(UiPropertyMutationRequest::widget_behavior(
            UiNodeId::new(1),
            "enabled",
            UiValue::Bool(false),
        ))
        .unwrap();
    assert_eq!(changed.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(changed.binding.applied_count, 2);
    assert_eq!(
        changed.binding.updates[0].previous,
        Some(UiValue::Bool(true))
    );
    assert_eq!(
        changed.binding.updates[0].source.kind,
        UiBindingSourceKind::WidgetBehavior
    );
    assert_eq!(
        changed.binding.updates[1].target.kind,
        UiBindingTargetKind::ComponentStateValue
    );
    assert_eq!(changed.binding.updates[1].previous, None);
    assert_eq!(changed.binding.updates[1].value, UiValue::Bool(false));
    assert!(changed
        .binding
        .dirty
        .contains(&UiBindingDirtyDomain::HitTest));
    assert!(changed
        .binding
        .dirty
        .contains(&UiBindingDirtyDomain::Render));
    assert!(changed.binding.dirty.contains(&UiBindingDirtyDomain::Input));
    assert!(changed.invalidation.dirty.input);
    assert!(changed.invalidation.dirty.hit_test);
    let node = surface.tree.node(UiNodeId::new(1)).unwrap();
    assert!(!node.state_flags.enabled);
    assert!(node.state_flags.dirty);
    assert!(node.dirty.input);

    let rejected = surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(1),
            "enabled",
            UiValue::String("false".to_string()),
        ))
        .unwrap();
    assert_eq!(rejected.status, UiPropertyMutationStatus::Rejected);
    assert_eq!(rejected.binding.rejected_count, 1);
    assert!(rejected.message.unwrap().contains("boolean"));
}

#[test]
fn surface_property_mutation_restores_collapsed_visibility_with_layout_dirty() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.reflector"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::VerticalBox(Default::default()))
            .with_layout_boundary(LayoutBoundary::ContentDriven),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/details"))
                .with_visibility(UiVisibility::Collapsed)
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(120.0),
                    height: fixed_constraint(32.0),
                }),
        )
        .unwrap();
    surface.compute_layout(UiSize::new(240.0, 120.0)).unwrap();
    surface.clear_dirty_flags();

    let report = surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "visibility",
            UiValue::Enum("visible".to_string()),
        ))
        .unwrap();

    assert_eq!(report.status, UiPropertyMutationStatus::Accepted);
    assert!(report.invalidation.dirty.layout);
    assert!(report.invalidation.dirty.hit_test);
    assert!(report.invalidation.dirty.render);
    assert!(report.invalidation.dirty.input);
    assert!(surface.tree.node(UiNodeId::new(2)).unwrap().dirty.layout);

    let rebuild = surface.rebuild_dirty(UiSize::new(240.0, 120.0)).unwrap();
    assert!(rebuild.layout_recomputed);
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .desired_size,
        DesiredSize::new(120.0, 32.0)
    );
}

#[test]
fn surface_property_mutation_keeps_template_visibility_metadata_in_sync() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.template_visibility"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::VerticalBox(Default::default()))
            .with_layout_boundary(LayoutBoundary::ContentDriven),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/details"))
                .with_visibility(UiVisibility::Collapsed)
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(120.0),
                    height: fixed_constraint(32.0),
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "DetailsPanel".to_string(),
                    attributes: [(
                        "visibility".to_string(),
                        toml::Value::String("collapsed".to_string()),
                    )]
                    .into_iter()
                    .collect(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.compute_layout(UiSize::new(240.0, 120.0)).unwrap();
    surface.clear_dirty_flags();

    let report = surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "visibility",
            UiValue::Enum("visible".to_string()),
        ))
        .unwrap();

    assert_eq!(report.status, UiPropertyMutationStatus::Accepted);
    let rebuild = surface.rebuild_dirty(UiSize::new(240.0, 120.0)).unwrap();
    assert!(rebuild.layout_recomputed);

    let node = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert_eq!(node.visibility, UiVisibility::Visible);
    assert_eq!(
        node.template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("visibility"),
        Some(&toml::Value::String("visible".to_string()))
    );
    assert_eq!(
        node.layout_cache.desired_size,
        DesiredSize::new(120.0, 32.0)
    );
}

#[test]
fn surface_property_mutation_marks_material_layout_metadata_as_layout_dirty() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.reflector"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/apply"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 24.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("ApplyButton".to_string()),
                    attributes: toml::from_str("layout_min_width = 80.0").unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();

    let report = surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "layout_min_width",
            UiValue::Float(120.0),
        ))
        .unwrap();

    assert_eq!(report.status, UiPropertyMutationStatus::Accepted);
    assert!(report.invalidation.dirty.layout);
    assert!(report.invalidation.dirty.hit_test);
    assert!(report.invalidation.dirty.render);

    let snapshot = surface.reflector_snapshot(None);
    let reflected = snapshot.node(UiNodeId::new(2)).expect("reflected button");
    let property = reflected
        .properties
        .get("layout_min_width")
        .expect("layout metadata property");
    assert!(property.invalidation.dirty.layout);
    assert!(property.invalidation.dirty.hit_test);
    assert!(property.invalidation.dirty.render);
}

#[test]
fn surface_property_mutation_updates_authored_metadata_and_reflector_snapshot() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.reflector"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/title"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 20.0))
                .with_state_flags(pointer_state())
                .with_input_policy(UiInputPolicy::Receive)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Label".to_string(),
                    control_id: Some("TitleLabel".to_string()),
                    attributes: toml::from_str("text = 'Inspect'").unwrap(),
                    bindings: vec![UiBindingRef {
                        id: "Title/Activate".to_string(),
                        event: UiEventKind::Click,
                        route: Some("MenuAction.OpenProject".to_string()),
                        action: None,
                        targets: Vec::new(),
                    }],
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let report = surface
        .mutate_property(
            UiPropertyMutationRequest::new(
                UiNodeId::new(2),
                "text",
                UiValue::String("Reflect".to_string()),
            )
            .with_source(UiReflectedPropertySource::Authored),
        )
        .unwrap();
    assert_eq!(report.status, UiPropertyMutationStatus::Accepted);
    assert!(report.invalidation.dirty.layout);
    assert!(report.invalidation.dirty.text);

    let snapshot = surface.reflector_snapshot(Some(
        zircon_runtime_interface::ui::surface::UiHitTestQuery::new(UiPoint::new(10.0, 10.0)),
    ));
    let reflected = snapshot.node(UiNodeId::new(2)).expect("reflected title");
    let text = reflected.properties.get("text").expect("text property");
    assert_eq!(reflected.display_name, "TitleLabel");
    assert_eq!(text.source, UiReflectedPropertySource::Authored);
    assert_eq!(text.value_kind, UiValueKind::String);
    assert_eq!(text.resolved_value, UiValue::String("Reflect".to_string()));
    assert!(text.invalidation.dirty.layout);
    assert_eq!(
        reflected
            .actions
            .get("Title/Activate")
            .expect("route-backed action")
            .binding_symbol,
        "MenuAction.OpenProject"
    );
    assert_eq!(
        snapshot.hit_context.unwrap().hit_target,
        Some(UiNodeId::new(2))
    );
}
