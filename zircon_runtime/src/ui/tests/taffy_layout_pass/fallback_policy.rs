use super::*;

#[test]
fn taffy_layout_pass_rejects_unsupported_slot_padding_values() {
    let mut tree = tree_with_root(
        190,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 190, fixed_node(191, Some(20.0), Some(10.0)));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(190), UiNodeId::new(191), UiSlotKind::Linear)
            .with_padding(UiMargin::new(-1.0, 0.0, 0.0, 0.0)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 30.0)).unwrap();

    let root = selection_for_node(&report, 190);
    assert_eq!(
        root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::SlotFramePolicy)
    );
}

#[test]
fn taffy_layout_pass_reports_non_finite_slot_padding_fallback() {
    let mut tree = tree_with_root(
        189,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 189, fixed_node(188, Some(20.0), Some(10.0)));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(189), UiNodeId::new(188), UiSlotKind::Linear)
            .with_padding(UiMargin::new(f32::INFINITY, 0.0, 0.0, 0.0)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 30.0)).unwrap();

    assert_fallback_route_reason(
        &report,
        189,
        UiLayoutEngineFamily::Flex,
        UiLayoutEngineFallbackReason::SlotFramePolicy,
    );
}

#[test]
fn taffy_layout_pass_reports_linear_main_axis_slot_alignment_fallback() {
    let mut tree = tree_with_root(
        196,
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 196, fixed_node(197, Some(20.0), Some(10.0)));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(196), UiNodeId::new(197), UiSlotKind::Linear)
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::Center)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 30.0)).unwrap();

    let root = selection_for_node(&report, 196);
    assert_eq!(root.request.family, UiLayoutEngineFamily::Flex);
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::Zircon);
    assert_eq!(root.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::SlotFramePolicy)
    );
}

#[test]
fn taffy_layout_pass_reports_cross_axis_slot_alignment_without_fixed_extent_fallback() {
    let mut tree = tree_with_root(
        198,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 198, fixed_node(199, Some(20.0), None));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(198), UiNodeId::new(199), UiSlotKind::Linear)
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::End)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 30.0)).unwrap();

    let root = selection_for_node(&report, 198);
    assert_eq!(root.request.family, UiLayoutEngineFamily::Flex);
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::Zircon);
    assert_eq!(root.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::SlotFramePolicy)
    );
}

#[test]
fn taffy_layout_pass_reports_axis_constraint_priority_fallback() {
    let mut tree = tree_with_root(
        470,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 470, priority_stretch_node(471, 1));
    insert_child(&mut tree, 470, priority_stretch_node(472, 0));

    let report = compute_layout_tree(&mut tree, UiSize::new(100.0, 20.0)).unwrap();

    assert_eq!(frame(&tree, 471), UiFrame::new(0.0, 0.0, 100.0, 10.0));
    assert_eq!(frame(&tree, 472), UiFrame::new(100.0, 0.0, 0.0, 10.0));
    assert_fallback_route_reason(
        &report,
        470,
        UiLayoutEngineFamily::Flex,
        UiLayoutEngineFallbackReason::AxisConstraintPriority,
    );
}

#[test]
fn taffy_layout_pass_reports_non_finite_axis_constraint_fallback() {
    let mut tree = tree_with_root(
        480,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 480, fixed_node_with_axis_max(481, f32::INFINITY));

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 20.0)).unwrap();

    assert_eq!(frame(&tree, 481), UiFrame::new(0.0, 0.0, 20.0, 10.0));
    assert_fallback_route_reason(
        &report,
        480,
        UiLayoutEngineFamily::Flex,
        UiLayoutEngineFallbackReason::InvalidLayoutValue,
    );
}

#[test]
fn taffy_layout_pass_reports_non_finite_linear_slot_sizing_fallback() {
    let mut tree = tree_with_root(
        490,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 490, node(491));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(490), UiNodeId::new(491), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch)
                .with_value(1.0)
                .with_max(f32::INFINITY),
        ),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 20.0)).unwrap();

    assert_eq!(frame(&tree, 491), UiFrame::new(0.0, 0.0, 80.0, 20.0));
    assert_fallback_route_reason(
        &report,
        490,
        UiLayoutEngineFamily::Flex,
        UiLayoutEngineFallbackReason::InvalidLayoutValue,
    );
}

#[test]
fn taffy_layout_pass_reports_non_finite_container_config_fallback() {
    let mut tree = tree_with_root(
        495,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: f32::NAN }),
    );
    insert_child(&mut tree, 495, fixed_node(496, Some(20.0), Some(10.0)));
    insert_child(&mut tree, 495, fixed_node(497, Some(20.0), Some(10.0)));

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 20.0)).unwrap();

    assert_eq!(frame(&tree, 496), UiFrame::new(0.0, 0.0, 20.0, 10.0));
    assert_eq!(frame(&tree, 497), UiFrame::new(20.0, 0.0, 20.0, 10.0));
    assert_fallback_route_reason(
        &report,
        495,
        UiLayoutEngineFamily::Flex,
        UiLayoutEngineFallbackReason::InvalidLayoutValue,
    );
}

#[test]
fn taffy_layout_pass_reports_collapsed_child_visibility_fallback() {
    let mut tree = tree_with_root(
        192,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(
        &mut tree,
        192,
        fixed_node(193, Some(20.0), Some(10.0)).with_visibility(UiVisibility::Collapsed),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 30.0)).unwrap();

    assert_taffy_native_family(&report, 192, UiLayoutEngineFamily::Flex);
    assert!(report.fallback_reason_counts.is_empty());
}

#[test]
fn taffy_layout_pass_reports_child_placement_policy_fallback() {
    let mut tree = tree_with_root(
        194,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(
        &mut tree,
        194,
        fixed_node(195, Some(20.0), Some(10.0))
            .with_anchor(Anchor::new(0.5, 0.0))
            .with_position(Position::new(4.0, 0.0)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 30.0)).unwrap();

    let root = selection_for_node(&report, 194);
    assert_eq!(root.request.family, UiLayoutEngineFamily::Flex);
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::Zircon);
    assert_eq!(root.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ChildPlacementPolicy)
    );
}

#[test]
fn size_box_contain_aspect_ratio_stays_zircon_owned() {
    let mut tree = tree_with_root(
        200,
        UiContainerKind::SizeBox(UiSizeBoxConfig { aspect_ratio: 2.0 }),
    );
    insert_child(&mut tree, 200, node(201));

    let report = compute_layout_tree(&mut tree, UiSize::new(100.0, 100.0)).unwrap();

    assert_eq!(frame(&tree, 200), UiFrame::new(0.0, 0.0, 100.0, 100.0));
    assert_eq!(frame(&tree, 201), UiFrame::new(0.0, 25.0, 100.0, 50.0));
    let root = selection_for_node(&report, 200);
    assert_eq!(root.request.family, UiLayoutEngineFamily::Container);
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::Zircon);
    assert_eq!(
        root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
}
