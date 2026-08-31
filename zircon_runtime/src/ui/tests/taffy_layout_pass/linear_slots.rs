use super::*;

#[test]
fn taffy_layout_pass_maps_linear_slot_padding_without_fallback() {
    let mut tree = tree_with_root(
        180,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 180, fixed_node(181, Some(20.0), Some(10.0)));
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(180), UiNodeId::new(181), UiSlotKind::Linear)
            .with_padding(UiMargin::new(5.0, 2.0, 7.0, 3.0)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 30.0)).unwrap();

    assert_eq!(frame(&tree, 181), UiFrame::new(5.0, 2.0, 20.0, 10.0));
    assert_taffy_native_family(&report, 180, UiLayoutEngineFamily::Flex);
}

#[test]
fn taffy_layout_pass_maps_linear_slot_padding_and_cross_axis_alignment_without_fallback() {
    let mut tree = tree_with_root(
        182,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 182, fixed_node(183, Some(20.0), Some(10.0)));
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(182), UiNodeId::new(183), UiSlotKind::Linear)
            .with_padding(UiMargin::new(5.0, 2.0, 7.0, 3.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::End)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 30.0)).unwrap();

    assert_eq!(frame(&tree, 183), UiFrame::new(5.0, 17.0, 20.0, 10.0));
    assert_taffy_native_family(&report, 182, UiLayoutEngineFamily::Flex);
}

#[test]
fn taffy_layout_pass_maps_vertical_linear_slot_padding_and_cross_axis_alignment_without_fallback() {
    let mut tree = tree_with_root(
        184,
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 184, fixed_node(185, Some(20.0), Some(10.0)));
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(184), UiNodeId::new(185), UiSlotKind::Linear)
            .with_padding(UiMargin::new(5.0, 2.0, 7.0, 3.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::End, UiAlignment::Start)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 40.0)).unwrap();

    assert_eq!(frame(&tree, 185), UiFrame::new(53.0, 2.0, 20.0, 10.0));
    assert_taffy_native_family(&report, 184, UiLayoutEngineFamily::Flex);
}

#[test]
fn taffy_layout_pass_maps_wrap_slot_padding_and_cross_axis_alignment_without_fallback() {
    let mut tree = tree_with_root(
        186,
        UiContainerKind::WrapBox(UiWrapBoxConfig {
            horizontal_gap: 0.0,
            vertical_gap: 6.0,
            item_min_width: 1.0,
        }),
    );
    insert_child(&mut tree, 186, fixed_node(187, Some(30.0), Some(30.0)));
    insert_child(&mut tree, 186, fixed_node(188, Some(20.0), Some(10.0)));
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(186), UiNodeId::new(188), UiSlotKind::Flow)
            .with_padding(UiMargin::new(5.0, 2.0, 3.0, 4.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::End)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(100.0, 40.0)).unwrap();

    assert_eq!(frame(&tree, 187), UiFrame::new(0.0, 0.0, 30.0, 30.0));
    assert_eq!(frame(&tree, 188), UiFrame::new(35.0, 16.0, 20.0, 10.0));
    assert_taffy_native_family(&report, 186, UiLayoutEngineFamily::Wrap);
}

#[test]
fn taffy_layout_pass_ignores_flow_slot_linear_sizing_without_fallback() {
    let mut tree = tree_with_root(
        460,
        UiContainerKind::WrapBox(UiWrapBoxConfig {
            horizontal_gap: 0.0,
            vertical_gap: 0.0,
            item_min_width: 1.0,
        }),
    );
    insert_child(&mut tree, 460, fixed_node(461, Some(20.0), Some(10.0)));
    insert_child(&mut tree, 460, fixed_node(462, Some(20.0), Some(10.0)));
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(460), UiNodeId::new(461), UiSlotKind::Flow).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(3.0),
        ),
    );
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(460), UiNodeId::new(462), UiSlotKind::Flow).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(1.0),
        ),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(100.0, 20.0)).unwrap();

    assert_eq!(frame(&tree, 461), UiFrame::new(0.0, 0.0, 20.0, 10.0));
    assert_eq!(frame(&tree, 462), UiFrame::new(20.0, 0.0, 20.0, 10.0));
    assert_taffy_native_family(&report, 460, UiLayoutEngineFamily::Wrap);
}

#[test]
fn taffy_layout_pass_maps_linear_slot_sizing_without_fallback() {
    let mut tree = tree_with_root(
        400,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 400, node(401));
    insert_child(&mut tree, 400, node(402));
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(400), UiNodeId::new(401), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(2.0),
        ),
    );
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(400), UiNodeId::new(402), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(1.0),
        ),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(300.0, 30.0)).unwrap();

    assert_eq!(frame(&tree, 401), UiFrame::new(0.0, 0.0, 200.0, 30.0));
    assert_eq!(frame(&tree, 402), UiFrame::new(200.0, 0.0, 100.0, 30.0));
    let root = selection_for_node(&report, 400);
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(root.support, UiLayoutEngineSupport::Native);
}

#[test]
fn taffy_layout_pass_maps_vertical_linear_slot_sizing_without_fallback() {
    let mut tree = tree_with_root(
        440,
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 440, node(441));
    insert_child(&mut tree, 440, node(442));
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(440), UiNodeId::new(441), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(2.0),
        ),
    );
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(440), UiNodeId::new(442), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(1.0),
        ),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(60.0, 300.0)).unwrap();

    assert_eq!(frame(&tree, 441), UiFrame::new(0.0, 0.0, 60.0, 200.0));
    assert_eq!(frame(&tree, 442), UiFrame::new(0.0, 200.0, 60.0, 100.0));
    assert_taffy_native_family(&report, 440, UiLayoutEngineFamily::Flex);
}

#[test]
fn taffy_layout_pass_maps_linear_auto_slot_sizing_without_fallback() {
    let mut tree = tree_with_root(
        410,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(
        &mut tree,
        410,
        node(411).with_template_metadata(metadata_with_attributes(
            "Label",
            r#"
text = "Hello"
font_size = 10.0
line_height = 12.0
"#,
        )),
    );
    insert_child(
        &mut tree,
        410,
        node(412).with_template_metadata(metadata_with_attributes(
            "Label",
            r#"
text = "Go"
font_size = 10.0
line_height = 12.0
"#,
        )),
    );
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(410), UiNodeId::new(411), UiSlotKind::Linear)
            .with_linear_sizing(UiLinearSlotSizing::new(UiLinearSlotSizeRule::Auto)),
    );
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(410), UiNodeId::new(412), UiSlotKind::Linear)
            .with_linear_sizing(UiLinearSlotSizing::new(UiLinearSlotSizeRule::Auto)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(100.0, 30.0)).unwrap();

    let first = tree.node(UiNodeId::new(411)).expect("first auto node");
    assert_eq!(first.layout_cache.desired_size.width, 25.0);
    assert_eq!(first.layout_cache.frame, UiFrame::new(0.0, 0.0, 25.0, 30.0));
    let second = tree.node(UiNodeId::new(412)).expect("second auto node");
    assert_eq!(second.layout_cache.desired_size.width, 10.0);
    assert_eq!(
        second.layout_cache.frame,
        UiFrame::new(25.0, 0.0, 10.0, 30.0)
    );
    assert_taffy_native_family(&report, 410, UiLayoutEngineFamily::Flex);
}

#[test]
fn taffy_layout_pass_maps_linear_stretch_content_slot_sizing_without_fallback() {
    let mut tree = tree_with_root(
        420,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(
        &mut tree,
        420,
        node(421).with_template_metadata(metadata_with_attributes(
            "Label",
            r#"
text = "Hello"
font_size = 10.0
line_height = 12.0
"#,
        )),
    );
    insert_child(
        &mut tree,
        420,
        node(422).with_template_metadata(metadata_with_attributes(
            "Label",
            r#"
text = "Go"
font_size = 10.0
line_height = 12.0
"#,
        )),
    );
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(420), UiNodeId::new(421), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::StretchContent).with_value(1.0),
        ),
    );
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(420), UiNodeId::new(422), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::StretchContent).with_value(1.0),
        ),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(95.0, 30.0)).unwrap();

    let first = tree
        .node(UiNodeId::new(421))
        .expect("first stretch-content node");
    assert_eq!(first.layout_cache.desired_size.width, 25.0);
    assert_eq!(first.layout_cache.frame, UiFrame::new(0.0, 0.0, 55.0, 30.0));
    let second = tree
        .node(UiNodeId::new(422))
        .expect("second stretch-content node");
    assert_eq!(second.layout_cache.desired_size.width, 10.0);
    assert_eq!(
        second.layout_cache.frame,
        UiFrame::new(55.0, 0.0, 40.0, 30.0)
    );
    assert_taffy_native_family(&report, 420, UiLayoutEngineFamily::Flex);
}

#[test]
fn taffy_layout_pass_maps_linear_slot_sizing_bounds_without_fallback() {
    let mut tree = tree_with_root(
        430,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 430, node(431));
    insert_child(&mut tree, 430, node(432));
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(430), UiNodeId::new(431), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch)
                .with_value(1.0)
                .with_min(80.0)
                .with_max(90.0),
        ),
    );
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(430), UiNodeId::new(432), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(1.0),
        ),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(200.0, 30.0)).unwrap();

    assert_eq!(frame(&tree, 431), UiFrame::new(0.0, 0.0, 90.0, 30.0));
    assert_eq!(frame(&tree, 432), UiFrame::new(90.0, 0.0, 110.0, 30.0));
    assert_taffy_native_family(&report, 430, UiLayoutEngineFamily::Flex);
}

#[test]
fn taffy_layout_pass_maps_vertical_linear_slot_sizing_bounds_without_fallback() {
    let mut tree = tree_with_root(
        450,
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 450, node(451));
    insert_child(&mut tree, 450, node(452));
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(450), UiNodeId::new(451), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch)
                .with_value(1.0)
                .with_min(80.0)
                .with_max(90.0),
        ),
    );
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(450), UiNodeId::new(452), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(1.0),
        ),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(60.0, 200.0)).unwrap();

    assert_eq!(frame(&tree, 451), UiFrame::new(0.0, 0.0, 60.0, 90.0));
    assert_eq!(frame(&tree, 452), UiFrame::new(0.0, 90.0, 60.0, 110.0));
    assert_taffy_native_family(&report, 450, UiLayoutEngineFamily::Flex);
}
