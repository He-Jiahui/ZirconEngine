use super::*;

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
fn taffy_layout_pass_preserves_fractional_fixed_extents() {
    let mut tree = tree_with_root(
        40,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
    );
    insert_child(&mut tree, 40, fixed_node(41, Some(20.0), Some(30.5)));

    let report = compute_layout_tree(&mut tree, UiSize::new(80.0, 40.0)).unwrap();

    assert_eq!(frame(&tree, 41), UiFrame::new(0.0, 0.0, 20.0, 30.5));
    assert_taffy_native_family(&report, 40, UiLayoutEngineFamily::Flex);
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
fn standalone_layout_reuses_one_operation_session_across_text_leaves() {
    let mut tree = tree_with_root(130, UiContainerKind::BlockBox);
    for (node_id, text) in [
        (131, "First label"),
        (132, "Second label"),
        (133, "Third label"),
    ] {
        insert_child(
            &mut tree,
            130,
            node(node_id).with_template_metadata(metadata_with_attributes(
                "Label",
                &format!(
                    r#"
text = "{text}"
font_size = 12.0
line_height = 15.0
"#
                ),
            )),
        );
    }

    let constructions_before = crate::text::current_thread_text_layout_session_construction_count();
    compute_layout_tree(&mut tree, UiSize::new(240.0, 80.0)).expect("layout should compute");
    let constructions_after = crate::text::current_thread_text_layout_session_construction_count();

    assert_eq!(
        constructions_after.saturating_sub(constructions_before),
        1,
        "one standalone tree layout owns one operation-local text session, not one per leaf"
    );
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
