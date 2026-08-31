use super::*;

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
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(300), UiNodeId::new(301), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(0, 0)),
    );
    tree.push_layout_slot(
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
    tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(320), UiNodeId::new(321), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(0, 0)),
    );
    tree.push_layout_slot(
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
    tree.push_layout_slot(
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
    tree.push_layout_slot(
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
    tree.push_layout_slot(
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
