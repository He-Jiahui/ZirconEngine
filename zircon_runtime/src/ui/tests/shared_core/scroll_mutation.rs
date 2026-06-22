use super::*;

#[test]
fn virtual_list_window_tracks_visible_range_with_overscan() {
    let window = compute_virtual_list_window(120.0, 150.0, 50.0, 20, 1);
    assert_eq!(
        window,
        UiVirtualListWindow {
            first_visible: 1,
            last_visible_exclusive: 7,
        }
    );

    let clamped = compute_virtual_list_window(0.0, 40.0, 50.0, 2, 3);
    assert_eq!(
        clamped,
        UiVirtualListWindow {
            first_visible: 0,
            last_visible_exclusive: 2,
        }
    );
}

#[test]
fn scrollable_box_tracks_content_metrics_virtual_window_and_local_scroll_invalidation() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_layout_boundary(LayoutBoundary::ContentDriven)
            .with_constraints(BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/scroll"))
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: stretch_constraint(90.0, 90.0, 100, 1.0),
                })
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: Some(UiVirtualListConfig {
                        item_extent: 40.0,
                        overscan: 1,
                    }),
                }))
                .with_scroll_state(UiScrollState::default()),
        )
        .unwrap();

    for item in 0..5 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(10 + item),
                    UiNodePath::new(format!("root/scroll/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: fixed_constraint(40.0),
                })
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: false,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();

    let scroll = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert_eq!(scroll.layout_cache.content_size, UiSize::new(200.0, 200.0));
    assert_eq!(
        scroll.layout_cache.virtual_window,
        Some(UiVirtualListWindow {
            first_visible: 0,
            last_visible_exclusive: 4,
        })
    );
    assert_eq!(
        scroll.scroll_state,
        Some(UiScrollState {
            offset: 0.0,
            viewport_extent: 90.0,
            content_extent: 200.0,
        })
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(10))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 200.0, 40.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(14))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::default()
    );

    surface
        .tree
        .set_scroll_offset(UiNodeId::new(2), 80.0)
        .unwrap();

    let root = surface.tree.node(UiNodeId::new(1)).unwrap();
    assert!(!root.dirty.layout);

    let scroll = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert!(scroll.dirty.layout);
    assert!(scroll.dirty.hit_test);
    assert!(scroll.dirty.render);
    assert!(scroll.dirty.visible_range);
    assert_eq!(scroll.scroll_state.unwrap().offset, 80.0);

    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();

    let scroll = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert_eq!(
        scroll.layout_cache.virtual_window,
        Some(UiVirtualListWindow {
            first_visible: 1,
            last_visible_exclusive: 5,
        })
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(10))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::default()
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(11))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, -40.0, 200.0, 40.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(12))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 200.0, 40.0)
    );
}

#[test]
fn pointer_dispatcher_applies_block_passthrough_and_capture_semantics() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/bottom"))
                .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0))
                .with_z_index(0)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/top"))
                .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0))
                .with_z_index(10)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: false,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .unwrap();
    surface.rebuild();

    let mut block_dispatcher = UiPointerDispatcher::default();
    block_dispatcher.register(UiNodeId::new(3), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::blocked()
    });
    block_dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::handled()
    });

    let blocked = surface
        .dispatch_pointer_event(
            &block_dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(30.0, 30.0)),
        )
        .unwrap();
    assert_eq!(blocked.blocked_by, Some(UiNodeId::new(3)));
    assert_eq!(blocked.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(
        blocked
            .invocations
            .iter()
            .map(|invocation| (invocation.node_id, invocation.effect))
            .collect::<Vec<_>>(),
        vec![
            (UiNodeId::new(3), UiPointerDispatchEffect::Blocked),
            (UiNodeId::new(2), UiPointerDispatchEffect::Handled),
        ]
    );

    let mut passthrough_dispatcher = UiPointerDispatcher::default();
    passthrough_dispatcher.register(UiNodeId::new(3), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::passthrough()
    });
    passthrough_dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::handled()
    });
    let passthrough = surface
        .dispatch_pointer_event(
            &passthrough_dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(30.0, 30.0)),
        )
        .unwrap();
    assert_eq!(passthrough.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(passthrough.passthrough, vec![UiNodeId::new(3)]);

    let mut capture_dispatcher = UiPointerDispatcher::default();
    capture_dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::capture()
    });
    let captured = surface
        .dispatch_pointer_event(
            &capture_dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(30.0, 30.0)),
        )
        .unwrap();
    assert_eq!(captured.captured_by, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
}

#[test]
fn captured_pointer_dispatch_keeps_move_and_up_targeting_the_captured_node_outside_hit_bounds() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 120.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/viewport"))
                .with_frame(UiFrame::new(0.0, 0.0, 100.0, 100.0))
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .unwrap();
    surface.rebuild();

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::capture()
    });
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Move, |_context| {
        UiPointerDispatchEffect::handled()
    });
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Up, |_context| {
        UiPointerDispatchEffect::handled()
    });

    let down = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(down.captured_by, Some(UiNodeId::new(2)));

    let moved = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(160.0, 160.0)),
        )
        .unwrap();
    assert_eq!(moved.route.target, Some(UiNodeId::new(2)));
    assert_eq!(moved.handled_by, Some(UiNodeId::new(2)));

    let up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(160.0, 160.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(up.route.target, Some(UiNodeId::new(2)));
    assert_eq!(up.handled_by, Some(UiNodeId::new(2)));
}

#[test]
fn scroll_pointer_event_scrolls_the_nearest_scrollable_box_when_unhandled() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_layout_boundary(LayoutBoundary::ContentDriven)
            .with_constraints(BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/scroll"))
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: stretch_constraint(90.0, 90.0, 100, 1.0),
                })
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: Some(UiVirtualListConfig {
                        item_extent: 40.0,
                        overscan: 0,
                    }),
                }))
                .with_scroll_state(UiScrollState::default())
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: false,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .unwrap();
    for item in 0..4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(20 + item),
                    UiNodePath::new(format!("root/scroll/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: fixed_constraint(40.0),
                })
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: false,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
            )
            .unwrap();
    }
    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();

    let result = surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(50.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        50.0
    );
    assert!(surface.tree.node(UiNodeId::new(2)).unwrap().dirty.layout);
}

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
