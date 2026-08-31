use super::*;

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
    assert!(taffy_arrange.contains("UiContainerKind::BlockBox"));
    assert!(!taffy_arrange.contains("template_metadata.is_some()"));
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
    assert_eq!(free_root.selected_backend, UiLayoutEngineBackend::Zircon);
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
        UiLayoutEngineBackend::Zircon
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
    assert_eq!(space_root.selected_backend, UiLayoutEngineBackend::Zircon);
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
    assert_eq!(overlay_root.selected_backend, UiLayoutEngineBackend::Zircon);
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
    slotted.tree.push_layout_slot(
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
    canvas_slot.tree.push_layout_slot(
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
    tree.push_layout_slot(
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
    tree.push_layout_slot(
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
    assert_taffy_native_family(&report, 520, UiLayoutEngineFamily::Flex);
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
    assert_eq!(report.fallback_reason_counts.len(), 4);
    assert_fallback_reason_count(
        &report,
        UiLayoutEngineFallbackReason::ZirconOwnedSemantics,
        2,
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
