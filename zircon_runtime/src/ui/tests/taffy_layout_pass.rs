use crate::ui::{layout::compute_layout_tree, surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        Anchor, AxisConstraint, BoxConstraints, Position, StretchMode, UiAlignment, UiAlignment2D,
        UiCanvasSlotPlacement, UiContainerKind, UiFrame, UiGridBoxConfig, UiGridSlotPlacement,
        UiLayoutEngineBackend, UiLayoutEngineFallbackReason, UiLayoutEngineFamily,
        UiLayoutEngineSupport, UiLinearBoxConfig, UiLinearSlotSizeRule, UiLinearSlotSizing,
        UiMargin, UiScrollableBoxConfig, UiSize, UiSizeBoxConfig, UiSlot, UiSlotKind,
        UiVirtualListConfig, UiWrapBoxConfig,
    },
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode, UiVisibility},
};

#[test]
fn layout_pass_routes_supported_containers_through_taffy_arrange() {
    let arrange = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/layout/pass/arrange.rs"),
    )
    .expect("read arrange pass");
    let taffy_arrange = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/ui/layout/pass/taffy_arrange.rs"),
    )
    .expect("read taffy arrange pass");

    assert!(arrange.contains("try_arrange_taffy_owned_children("));
    assert!(taffy_arrange.contains("UiContainerKind::HorizontalBox(_)"));
    assert!(taffy_arrange.contains("UiContainerKind::VerticalBox(_)"));
    assert!(taffy_arrange.contains("UiContainerKind::WrapBox(_)"));
    assert!(taffy_arrange.contains("UiContainerKind::GridBox(_)"));
    assert!(!taffy_arrange.contains("template_metadata.is_some()"));
    assert!(!taffy_arrange.contains("UiLayoutEngineFamily::Block"));
    assert!(!taffy_arrange.contains("Display::Block"));
    assert!(!taffy_arrange.contains("UiContainerKind::Overlay"));
    assert!(!taffy_arrange.contains("UiContainerKind::ScrollableBox"));
    assert!(!taffy_arrange.contains("UiContainerKind::SizeBox"));
    assert!(!taffy_arrange.contains("UiContainerKind::Container =>"));
}

#[test]
fn layout_pass_reports_taffy_native_and_zircon_fallback_routes() {
    let mut surface = UiSurface::new(UiTreeId::new("taffy.layout.report.native"));
    surface
        .tree
        .insert_root(
            node(1).with_container(UiContainerKind::HorizontalBox(UiLinearBoxConfig {
                gap: 0.0,
            })),
        );
    insert_child(&mut surface.tree, 1, node(2));
    insert_child(&mut surface.tree, 1, node(3));
    surface.compute_layout(UiSize::new(160.0, 20.0)).unwrap();
    let frame = surface.surface_frame();
    let report = &frame.layout_engine_report;
    assert_eq!(report.request_count, 1);
    assert_eq!(report.taffy_selected_count, 1);
    let root = selection_for_node(report, 1);
    assert_eq!(root.request.family, UiLayoutEngineFamily::Flex);
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(root.support, UiLayoutEngineSupport::Native);

    let mut free = UiSurface::new(UiTreeId::new("taffy.layout.report.free"));
    free.tree
        .insert_root(node(5).with_container(UiContainerKind::Free));
    insert_child(&mut free.tree, 5, fixed_node(6, Some(10.0), Some(10.0)));
    free.compute_layout(UiSize::new(80.0, 20.0)).unwrap();
    let free_frame = free.surface_frame();
    let free_report = &free_frame.layout_engine_report;
    let free_root = selection_for_node(free_report, 5);
    assert_eq!(free_root.request.family, UiLayoutEngineFamily::Free);
    assert_eq!(
        free_root.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(
        free_root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );

    let mut container = UiSurface::new(UiTreeId::new("taffy.layout.report.container"));
    container
        .tree
        .insert_root(node(7).with_container(UiContainerKind::Container));
    insert_child(
        &mut container.tree,
        7,
        fixed_node(8, Some(12.0), Some(12.0)),
    );
    container.compute_layout(UiSize::new(80.0, 20.0)).unwrap();
    let container_frame = container.surface_frame();
    let container_report = &container_frame.layout_engine_report;
    let container_root = selection_for_node(container_report, 7);
    assert_eq!(
        container_root.request.family,
        UiLayoutEngineFamily::Container
    );
    assert_eq!(
        container_root.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(
        container_root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );

    let mut space = UiSurface::new(UiTreeId::new("taffy.layout.report.space"));
    space
        .tree
        .insert_root(node(9).with_container(UiContainerKind::Space));
    insert_child(&mut space.tree, 9, fixed_node(19, Some(12.0), Some(12.0)));
    space.compute_layout(UiSize::new(80.0, 20.0)).unwrap();
    let space_frame = space.surface_frame();
    let space_report = &space_frame.layout_engine_report;
    let space_root = selection_for_node(space_report, 9);
    assert_eq!(space_root.request.family, UiLayoutEngineFamily::Container);
    assert_eq!(
        space_root.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(
        space_root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );

    let mut overlay = UiSurface::new(UiTreeId::new("taffy.layout.report.overlay"));
    overlay
        .tree
        .insert_root(node(10).with_container(UiContainerKind::Overlay));
    insert_child(&mut overlay.tree, 10, node(11));
    overlay.compute_layout(UiSize::new(80.0, 40.0)).unwrap();
    let overlay_frame = overlay.surface_frame();
    let overlay_report = &overlay_frame.layout_engine_report;
    let overlay_root = selection_for_node(overlay_report, 10);
    assert_eq!(
        overlay_root.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(
        overlay_root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );

    let mut slotted = UiSurface::new(UiTreeId::new("taffy.layout.report.slot"));
    slotted
        .tree
        .insert_root(
            node(20).with_container(UiContainerKind::HorizontalBox(UiLinearBoxConfig {
                gap: 0.0,
            })),
        );
    insert_child(&mut slotted.tree, 20, node(21));
    slotted.tree.slots.push(
        UiSlot::new(UiNodeId::new(20), UiNodeId::new(21), UiSlotKind::Linear)
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::Start)),
    );
    slotted.compute_layout(UiSize::new(80.0, 20.0)).unwrap();
    let slot_frame = slotted.surface_frame();
    let slot_report = &slot_frame.layout_engine_report;
    let slotted_root = selection_for_node(slot_report, 20);
    assert_eq!(
        slotted_root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::SlotFramePolicy)
    );

    let mut canvas_slot = UiSurface::new(UiTreeId::new("taffy.layout.report.canvas_slot"));
    canvas_slot
        .tree
        .insert_root(
            node(25).with_container(UiContainerKind::HorizontalBox(UiLinearBoxConfig {
                gap: 0.0,
            })),
        );
    insert_child(&mut canvas_slot.tree, 25, node(26));
    canvas_slot.tree.slots.push(
        UiSlot::new(UiNodeId::new(25), UiNodeId::new(26), UiSlotKind::Linear)
            .with_canvas_placement(UiCanvasSlotPlacement::default()),
    );
    canvas_slot.compute_layout(UiSize::new(80.0, 20.0)).unwrap();
    let canvas_frame = canvas_slot.surface_frame();
    let canvas_report = &canvas_frame.layout_engine_report;
    let canvas_root = selection_for_node(canvas_report, 25);
    assert_eq!(
        canvas_root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::SlotCanvasPlacement)
    );

    let mut scrollable = UiSurface::new(UiTreeId::new("taffy.layout.report.scrollable"));
    scrollable
        .tree
        .insert_root(node(28).with_container(UiContainerKind::ScrollableBox(
            UiScrollableBoxConfig::default(),
        )));
    insert_child(&mut scrollable.tree, 28, fixed_node(29, None, Some(20.0)));
    scrollable.compute_layout(UiSize::new(80.0, 20.0)).unwrap();
    let scroll_frame = scrollable.surface_frame();
    let scroll_report = &scroll_frame.layout_engine_report;
    let scroll_root = selection_for_node(scroll_report, 28);
    assert_eq!(scroll_root.request.family, UiLayoutEngineFamily::Scrollable);
    assert_eq!(
        scroll_root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );

    let mut virtual_list = UiSurface::new(UiTreeId::new("taffy.layout.report.virtual"));
    virtual_list
        .tree
        .insert_root(node(30).with_container(UiContainerKind::ScrollableBox(
            UiScrollableBoxConfig {
                virtualization: Some(UiVirtualListConfig {
                    item_extent: 20.0,
                    overscan: 0,
                }),
                ..UiScrollableBoxConfig::default()
            },
        )));
    insert_child(&mut virtual_list.tree, 30, fixed_node(31, None, Some(20.0)));
    virtual_list
        .compute_layout(UiSize::new(80.0, 20.0))
        .unwrap();
    let virtual_frame = virtual_list.surface_frame();
    let virtual_report = &virtual_frame.layout_engine_report;
    let virtual_root = selection_for_node(virtual_report, 30);
    assert_eq!(
        virtual_root.request.family,
        UiLayoutEngineFamily::VirtualizedList
    );
    assert_eq!(
        virtual_root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
}

#[test]
fn taffy_layout_pass_aggregates_fallback_reason_counts() {
    let mut tree = tree_with_root(
        500,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(
        &mut tree,
        500,
        fixed_node(501, Some(60.0), Some(40.0)).with_container(UiContainerKind::Overlay),
    );
    insert_child(&mut tree, 501, fixed_node(511, Some(20.0), Some(10.0)));
    insert_child(
        &mut tree,
        500,
        fixed_node(502, Some(60.0), Some(40.0)).with_container(UiContainerKind::SizeBox(
            UiSizeBoxConfig { aspect_ratio: 1.0 },
        )),
    );
    insert_child(&mut tree, 502, fixed_node(512, Some(20.0), Some(10.0)));

    let report = compute_layout_tree(&mut tree, UiSize::new(120.0, 40.0)).unwrap();

    assert_taffy_native_family(&report, 500, UiLayoutEngineFamily::Flex);
    assert_zircon_owned_route(&report, 501, UiLayoutEngineFamily::Overlay);
    assert_zircon_owned_route(&report, 502, UiLayoutEngineFamily::Container);
    assert_eq!(report.fallback_reason_counts.len(), 1);
    assert_eq!(
        report.fallback_reason_counts[0].reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
    assert_eq!(report.fallback_reason_counts[0].count, 2);
}

#[test]
fn taffy_layout_pass_aggregates_distinct_fallback_reason_counts() {
    let mut tree = tree_with_root(510, UiContainerKind::Free);
    insert_child(
        &mut tree,
        510,
        fixed_node(520, Some(80.0), Some(24.0)).with_container(UiContainerKind::HorizontalBox(
            UiLinearBoxConfig { gap: 0.0 },
        )),
    );
    insert_child(
        &mut tree,
        520,
        fixed_node(521, Some(20.0), Some(10.0)).with_visibility(UiVisibility::Collapsed),
    );
    insert_child(
        &mut tree,
        510,
        fixed_node(530, Some(80.0), Some(24.0)).with_container(UiContainerKind::HorizontalBox(
            UiLinearBoxConfig { gap: 0.0 },
        )),
    );
    insert_child(
        &mut tree,
        530,
        fixed_node(531, Some(20.0), Some(10.0))
            .with_anchor(Anchor::new(0.5, 0.0))
            .with_position(Position::new(4.0, 0.0)),
    );
    insert_child(
        &mut tree,
        510,
        fixed_node(540, Some(80.0), Some(24.0)).with_container(UiContainerKind::HorizontalBox(
            UiLinearBoxConfig { gap: 0.0 },
        )),
    );
    insert_child(&mut tree, 540, fixed_node(541, Some(20.0), Some(10.0)));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(540), UiNodeId::new(541), UiSlotKind::Linear)
            .with_padding(UiMargin::new(-1.0, 0.0, 0.0, 0.0)),
    );
    insert_child(
        &mut tree,
        510,
        fixed_node(550, Some(80.0), Some(24.0)).with_container(UiContainerKind::HorizontalBox(
            UiLinearBoxConfig { gap: 0.0 },
        )),
    );
    insert_child(&mut tree, 550, fixed_node(551, Some(20.0), Some(10.0)));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(550), UiNodeId::new(551), UiSlotKind::Linear)
            .with_canvas_placement(UiCanvasSlotPlacement::default()),
    );
    insert_child(
        &mut tree,
        510,
        fixed_node(560, Some(80.0), Some(24.0)).with_container(UiContainerKind::Overlay),
    );
    insert_child(&mut tree, 560, fixed_node(561, Some(20.0), Some(10.0)));

    let report = compute_layout_tree(&mut tree, UiSize::new(120.0, 80.0)).unwrap();

    assert_zircon_owned_route(&report, 510, UiLayoutEngineFamily::Free);
    assert_fallback_route_reason(
        &report,
        520,
        UiLayoutEngineFamily::Flex,
        UiLayoutEngineFallbackReason::UnsupportedChildVisibility,
    );
    assert_fallback_route_reason(
        &report,
        530,
        UiLayoutEngineFamily::Flex,
        UiLayoutEngineFallbackReason::ChildPlacementPolicy,
    );
    assert_fallback_route_reason(
        &report,
        540,
        UiLayoutEngineFamily::Flex,
        UiLayoutEngineFallbackReason::SlotFramePolicy,
    );
    assert_fallback_route_reason(
        &report,
        550,
        UiLayoutEngineFamily::Flex,
        UiLayoutEngineFallbackReason::SlotCanvasPlacement,
    );
    assert_zircon_owned_route(&report, 560, UiLayoutEngineFamily::Overlay);
    assert_eq!(report.fallback_reason_counts.len(), 5);
    assert_fallback_reason_count(
        &report,
        UiLayoutEngineFallbackReason::ZirconOwnedSemantics,
        2,
    );
    assert_fallback_reason_count(
        &report,
        UiLayoutEngineFallbackReason::UnsupportedChildVisibility,
        1,
    );
    assert_fallback_reason_count(
        &report,
        UiLayoutEngineFallbackReason::ChildPlacementPolicy,
        1,
    );
    assert_fallback_reason_count(&report, UiLayoutEngineFallbackReason::SlotFramePolicy, 1);
    assert_fallback_reason_count(
        &report,
        UiLayoutEngineFallbackReason::SlotCanvasPlacement,
        1,
    );
}

#[test]
fn taffy_layout_pass_arranges_linear_wrap_and_grid_containers() {
    let mut linear = tree_with_root(
        1,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 5.0 }),
    );
    insert_child(&mut linear, 1, fixed_node(2, Some(50.0), None));
    insert_child(&mut linear, 1, node(3));
    insert_child(&mut linear, 1, fixed_node(4, Some(25.0), None));
    let linear_report = compute_layout_tree(&mut linear, UiSize::new(200.0, 40.0)).unwrap();
    assert_eq!(frame(&linear, 2), UiFrame::new(0.0, 0.0, 50.0, 40.0));
    assert_eq!(frame(&linear, 3), UiFrame::new(55.0, 0.0, 115.0, 40.0));
    assert_eq!(frame(&linear, 4), UiFrame::new(175.0, 0.0, 25.0, 40.0));
    assert_taffy_native_family(&linear_report, 1, UiLayoutEngineFamily::Flex);

    let mut wrap = tree_with_root(
        10,
        UiContainerKind::WrapBox(UiWrapBoxConfig {
            horizontal_gap: 4.0,
            vertical_gap: 5.0,
            item_min_width: 30.0,
        }),
    );
    insert_child(&mut wrap, 10, fixed_node(11, Some(40.0), Some(10.0)));
    insert_child(&mut wrap, 10, fixed_node(12, Some(40.0), Some(10.0)));
    insert_child(&mut wrap, 10, fixed_node(13, Some(40.0), Some(10.0)));
    let wrap_report = compute_layout_tree(&mut wrap, UiSize::new(90.0, 40.0)).unwrap();
    assert_eq!(frame(&wrap, 11), UiFrame::new(0.0, 0.0, 40.0, 10.0));
    assert_eq!(frame(&wrap, 12), UiFrame::new(44.0, 0.0, 40.0, 10.0));
    assert_eq!(frame(&wrap, 13), UiFrame::new(0.0, 15.0, 40.0, 10.0));
    assert_taffy_native_family(&wrap_report, 10, UiLayoutEngineFamily::Wrap);

    let mut grid = tree_with_root(
        20,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 2,
            rows: 1,
            column_gap: 4.0,
            row_gap: 0.0,
        }),
    );
    insert_child(&mut grid, 20, node(21));
    insert_child(&mut grid, 20, node(22));
    let grid_report = compute_layout_tree(&mut grid, UiSize::new(104.0, 20.0)).unwrap();
    assert_eq!(frame(&grid, 21), UiFrame::new(0.0, 0.0, 50.0, 20.0));
    assert_eq!(frame(&grid, 22), UiFrame::new(54.0, 0.0, 50.0, 20.0));
    assert_taffy_native_family(&grid_report, 20, UiLayoutEngineFamily::Grid);
}

#[test]
fn taffy_layout_pass_accepts_template_metadata_from_v2_assets() {
    let mut tree = tree_with_root(
        100,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 4.0 }),
    );
    insert_child(
        &mut tree,
        100,
        fixed_node(101, Some(44.0), None).with_template_metadata(template_metadata("Button")),
    );
    insert_child(
        &mut tree,
        100,
        node(102).with_template_metadata(template_metadata("Label")),
    );

    compute_layout_tree(&mut tree, UiSize::new(160.0, 32.0)).unwrap();

    assert_eq!(frame(&tree, 101), UiFrame::new(0.0, 0.0, 44.0, 32.0));
    assert_eq!(frame(&tree, 102), UiFrame::new(48.0, 0.0, 112.0, 32.0));
}

#[test]
fn taffy_layout_pass_uses_measured_text_and_image_desired_sizes() {
    let mut tree = tree_with_root(
        150,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 3.0 }),
    );
    insert_child(
        &mut tree,
        150,
        node(151).with_template_metadata(metadata_with_attributes(
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
        150,
        node(152).with_template_metadata(metadata_with_attributes(
            "IconButton",
            r#"
image = "asset://icons/run.png"
layout_icon_size = 18.0
layout_padding_left = 2.0
layout_padding_right = 2.0
"#,
        )),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 30.0)).unwrap();

    let text = tree.node(UiNodeId::new(151)).expect("text node");
    assert_eq!(text.layout_cache.desired_size.width, 25.0);
    assert_eq!(text.layout_cache.desired_size.height, 12.0);
    assert_eq!(text.layout_cache.frame, UiFrame::new(0.0, 0.0, 25.0, 30.0));

    let image = tree.node(UiNodeId::new(152)).expect("image node");
    assert_eq!(image.layout_cache.desired_size.width, 22.0);
    assert_eq!(image.layout_cache.desired_size.height, 18.0);
    assert_eq!(
        image.layout_cache.frame,
        UiFrame::new(28.0, 0.0, 22.0, 30.0)
    );
    assert_taffy_native_family(&report, 150, UiLayoutEngineFamily::Flex);
}

#[test]
fn taffy_layout_pass_maps_linear_slot_padding_without_fallback() {
    let mut tree = tree_with_root(
        180,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 180, fixed_node(181, Some(20.0), Some(10.0)));
    tree.slots.push(
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
    tree.slots.push(
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
    tree.slots.push(
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
    tree.slots.push(
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
    tree.slots.push(
        UiSlot::new(UiNodeId::new(460), UiNodeId::new(461), UiSlotKind::Flow).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(3.0),
        ),
    );
    tree.slots.push(
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
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::LegacyZircon);
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
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::LegacyZircon);
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

    let root = selection_for_node(&report, 192);
    assert_eq!(root.request.family, UiLayoutEngineFamily::Flex);
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::LegacyZircon);
    assert_eq!(root.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::UnsupportedChildVisibility)
    );
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
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::LegacyZircon);
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
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::LegacyZircon);
    assert_eq!(
        root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
}

#[test]
fn taffy_layout_pass_maps_grid_slot_placement_without_fallback() {
    let mut tree = tree_with_root(
        300,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 2,
            rows: 2,
            column_gap: 4.0,
            row_gap: 6.0,
        }),
    );
    insert_child(&mut tree, 300, node(301));
    insert_child(&mut tree, 300, node(302));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(300), UiNodeId::new(301), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(0, 0)),
    );
    tree.slots.push(
        UiSlot::new(UiNodeId::new(300), UiNodeId::new(302), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(1, 1)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(124.0, 82.0)).unwrap();

    assert_eq!(frame(&tree, 301), UiFrame::new(0.0, 0.0, 60.0, 38.0));
    assert_eq!(frame(&tree, 302), UiFrame::new(64.0, 44.0, 60.0, 38.0));
    let root = selection_for_node(&report, 300);
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(root.support, UiLayoutEngineSupport::Native);
}

#[test]
fn taffy_layout_pass_maps_grid_slot_span_without_fallback() {
    let mut tree = tree_with_root(
        320,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 3,
            rows: 2,
            column_gap: 6.0,
            row_gap: 4.0,
        }),
    );
    insert_child(&mut tree, 320, node(321));
    insert_child(&mut tree, 320, node(322));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(320), UiNodeId::new(321), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(0, 0)),
    );
    tree.slots.push(
        UiSlot::new(UiNodeId::new(320), UiNodeId::new(322), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(1, 0).with_span(2, 2)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(156.0, 64.0)).unwrap();

    assert_eq!(frame(&tree, 321), UiFrame::new(0.0, 0.0, 48.0, 30.0));
    assert_eq!(frame(&tree, 322), UiFrame::new(54.0, 0.0, 102.0, 64.0));
    assert_taffy_native_family(&report, 320, UiLayoutEngineFamily::Grid);
}

#[test]
fn taffy_layout_pass_expands_grid_tracks_for_out_of_bounds_slot_span_without_fallback() {
    let mut tree = tree_with_root(
        330,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 1,
            rows: 1,
            column_gap: 6.0,
            row_gap: 5.0,
        }),
    );
    insert_child(&mut tree, 330, node(331));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(330), UiNodeId::new(331), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(1, 1).with_span(2, 2)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(156.0, 100.0)).unwrap();

    assert_eq!(frame(&tree, 331), UiFrame::new(54.0, 35.0, 102.0, 65.0));
    assert_taffy_native_family(&report, 330, UiLayoutEngineFamily::Grid);
}

#[test]
fn taffy_layout_pass_maps_grid_slot_padding_and_alignment_without_fallback() {
    let mut tree = tree_with_root(
        350,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 2,
            rows: 2,
            column_gap: 4.0,
            row_gap: 6.0,
        }),
    );
    insert_child(&mut tree, 350, fixed_node(351, Some(20.0), Some(10.0)));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(350), UiNodeId::new(351), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(1, 1))
            .with_padding(UiMargin::new(2.0, 3.0, 4.0, 5.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::End)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(124.0, 82.0)).unwrap();

    assert_eq!(frame(&tree, 351), UiFrame::new(83.0, 67.0, 20.0, 10.0));
    assert_taffy_native_family(&report, 350, UiLayoutEngineFamily::Grid);
}

#[test]
fn taffy_layout_pass_reports_grid_slot_alignment_without_fixed_extent_fallback() {
    let mut tree = tree_with_root(
        360,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 1,
            rows: 1,
            column_gap: 0.0,
            row_gap: 0.0,
        }),
    );
    insert_child(&mut tree, 360, node(361));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(360), UiNodeId::new(361), UiSlotKind::Grid)
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::Start)),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 30.0)).unwrap();

    assert_fallback_route_reason(
        &report,
        360,
        UiLayoutEngineFamily::Grid,
        UiLayoutEngineFallbackReason::SlotFramePolicy,
    );
}

#[test]
fn taffy_layout_pass_maps_linear_slot_sizing_without_fallback() {
    let mut tree = tree_with_root(
        400,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 400, node(401));
    insert_child(&mut tree, 400, node(402));
    tree.slots.push(
        UiSlot::new(UiNodeId::new(400), UiNodeId::new(401), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(2.0),
        ),
    );
    tree.slots.push(
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
    tree.slots.push(
        UiSlot::new(UiNodeId::new(440), UiNodeId::new(441), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(2.0),
        ),
    );
    tree.slots.push(
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
    tree.slots.push(
        UiSlot::new(UiNodeId::new(410), UiNodeId::new(411), UiSlotKind::Linear)
            .with_linear_sizing(UiLinearSlotSizing::new(UiLinearSlotSizeRule::Auto)),
    );
    tree.slots.push(
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
    tree.slots.push(
        UiSlot::new(UiNodeId::new(420), UiNodeId::new(421), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::StretchContent).with_value(1.0),
        ),
    );
    tree.slots.push(
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
    tree.slots.push(
        UiSlot::new(UiNodeId::new(430), UiNodeId::new(431), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch)
                .with_value(1.0)
                .with_min(80.0)
                .with_max(90.0),
        ),
    );
    tree.slots.push(
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
    tree.slots.push(
        UiSlot::new(UiNodeId::new(450), UiNodeId::new(451), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch)
                .with_value(1.0)
                .with_min(80.0)
                .with_max(90.0),
        ),
    );
    tree.slots.push(
        UiSlot::new(UiNodeId::new(450), UiNodeId::new(452), UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(1.0),
        ),
    );

    let report = compute_layout_tree(&mut tree, UiSize::new(60.0, 200.0)).unwrap();

    assert_eq!(frame(&tree, 451), UiFrame::new(0.0, 0.0, 60.0, 90.0));
    assert_eq!(frame(&tree, 452), UiFrame::new(0.0, 90.0, 60.0, 110.0));
    assert_taffy_native_family(&report, 450, UiLayoutEngineFamily::Flex);
}

fn tree_with_root(root_id: u64, container: UiContainerKind) -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(format!("taffy.layout.{root_id}")));
    tree.insert_root(node(root_id).with_container(container));
    tree
}

fn insert_child(tree: &mut UiTree, parent_id: u64, child: UiTreeNode) {
    tree.insert_child(UiNodeId::new(parent_id), child)
        .expect("insert child");
}

fn node(id: u64) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("node.{id}")))
}

fn fixed_node(id: u64, width: Option<f32>, height: Option<f32>) -> UiTreeNode {
    let mut constraints = BoxConstraints::default();
    if let Some(width) = width {
        constraints.width = fixed_axis(width);
    }
    if let Some(height) = height {
        constraints.height = fixed_axis(height);
    }
    node(id).with_constraints(constraints)
}

fn priority_stretch_node(id: u64, width_priority: i32) -> UiTreeNode {
    let mut constraints = BoxConstraints::default();
    constraints.width = AxisConstraint {
        min: 0.0,
        max: -1.0,
        preferred: 0.0,
        priority: width_priority,
        weight: 1.0,
        stretch_mode: StretchMode::Stretch,
    };
    constraints.height = fixed_axis(10.0);
    node(id).with_constraints(constraints)
}

fn fixed_node_with_axis_max(id: u64, max: f32) -> UiTreeNode {
    let mut constraints = BoxConstraints::default();
    constraints.width = AxisConstraint {
        min: 0.0,
        max,
        preferred: 20.0,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    };
    constraints.height = fixed_axis(10.0);
    node(id).with_constraints(constraints)
}

fn fixed_axis(value: f32) -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: value,
        preferred: value,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn frame(tree: &UiTree, id: u64) -> UiFrame {
    tree.node(UiNodeId::new(id))
        .expect("node")
        .layout_cache
        .frame
}

fn template_metadata(component: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: component.to_string(),
        ..UiTemplateNodeMetadata::default()
    }
}

fn metadata_with_attributes(component: &str, attributes: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: component.to_string(),
        attributes: toml::from_str(attributes).expect("metadata attributes"),
        ..UiTemplateNodeMetadata::default()
    }
}

fn selection_for_node<'a>(
    report: &'a zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
    node_id: u64,
) -> &'a zircon_runtime_interface::ui::layout::UiLayoutEngineSelection {
    report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(UiNodeId::new(node_id)))
        .expect("layout engine selection")
}

fn assert_taffy_native_family(
    report: &zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
    node_id: u64,
    family: UiLayoutEngineFamily,
) {
    let selection = selection_for_node(report, node_id);
    assert_eq!(selection.request.family, family);
    assert_eq!(selection.selected_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(selection.fallback_reason, None);
}

fn assert_zircon_owned_route(
    report: &zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
    node_id: u64,
    family: UiLayoutEngineFamily,
) {
    assert_fallback_route_reason(
        report,
        node_id,
        family,
        UiLayoutEngineFallbackReason::ZirconOwnedSemantics,
    );
}

fn assert_fallback_route_reason(
    report: &zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
    node_id: u64,
    family: UiLayoutEngineFamily,
    reason: UiLayoutEngineFallbackReason,
) {
    let selection = selection_for_node(report, node_id);
    assert_eq!(selection.request.family, family);
    assert_eq!(
        selection.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(selection.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(selection.fallback_reason, Some(reason));
}

fn assert_fallback_reason_count(
    report: &zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
    reason: UiLayoutEngineFallbackReason,
    count: u64,
) {
    let reason_count = report
        .fallback_reason_counts
        .iter()
        .find(|reason_count| reason_count.reason == Some(reason))
        .expect("fallback reason count");
    assert_eq!(reason_count.count, count);
}
