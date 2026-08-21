use super::support::*;
use super::*;

#[test]
#[ignore = "writes real Blend Space workbench screenshots under docs/tests/editor"]
fn capture_blend_space_workspace_visual_artifacts() {
    std::env::set_var("SLINT_BACKEND", "software");

    let narrow = blend_space_window(640, 520);
    save_window_snapshot(&narrow, BLEND_SPACE_NARROW_ARTIFACT);
    assert_blend_space_native_parent_chain_and_paint(&narrow, 640, 520);

    let compact = blend_space_window(900, 620);
    save_window_snapshot(&compact, BLEND_SPACE_COMPACT_ARTIFACT);
    assert_blend_space_native_parent_chain_and_paint(&compact, 900, 620);

    let wide = blend_space_window(1260, 780);
    save_window_snapshot(&wide, BLEND_SPACE_WIDE_ARTIFACT);
    assert_blend_space_native_parent_chain_and_paint(&wide, 1260, 780);
}

pub(in super::super) fn assert_blend_space_native_parent_chain_and_paint(
    ui: &UiHostWindow,
    width: u32,
    height: u32,
) {
    let presentation = ui.get_host_presentation();
    let nodes = (0..presentation.workbench_window_nodes.row_count())
        .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
        .collect::<Vec<_>>();
    let host = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchExtensionModuleWorkspacesHost")
        .expect("extension workspace host should be present in the native model");
    let workspace = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchExtensionBlendSpaceWorkspace")
        .expect("Blend Space workspace should be present in the native model");
    let canvas = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchExtensionBlendSpaceSampleCanvas")
        .expect("Blend Space sample canvas should be present in the native model");
    let grid = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchExtensionBlendSpaceSampleGrid")
        .expect("Blend Space sample grid should be present in the native model");
    assert_eq!(grid.component_variant.as_str(), "sample-grid");
    let sample_grid = &grid.sample_grid.generation;
    assert_eq!(sample_grid.x_axis_label(), "Direction (deg)");
    assert_eq!(sample_grid.y_axis_label(), "Speed (cm/s)");
    assert_eq!(sample_grid.x_ticks().len(), 9);
    assert_eq!(sample_grid.y_ticks().len(), 6);
    assert_eq!(sample_grid.points().len(), 8);
    assert!(
        sample_grid
            .points()
            .iter()
            .any(|point| point.selected() && point.label() == "Run_Fwd"),
        "Blend Space sample grid should preserve the selected authored sample"
    );
    if width >= 900 {
        let timeline = nodes
            .iter()
            .find(|node| node.control_id.as_str() == "WorkbenchExtensionBlendSpacePreviewTimeline")
            .expect("regular and wide Blend Space should expose the typed preview timeline");
        assert_eq!(timeline.component_variant.as_str(), "timeline-strip");
        let generation = &timeline.timeline_strip.generation;
        assert_eq!(generation.duration(), 3.0);
        assert_eq!(generation.current_time(), 3.0);
        assert_eq!(generation.tick_interval(), 0.5);
        assert_eq!(generation.track_label(), "Run_Fwd");
        assert_eq!(generation.keys().len(), 2);
        assert!(
            generation
                .keys()
                .iter()
                .any(|key| key.selected() && key.time() == 2.0),
            "preview timeline should preserve the selected authored key"
        );
    }
    if width >= 1_200 {
        let heatmap = nodes
            .iter()
            .find(|node| node.control_id.as_str() == "WorkbenchExtensionBlendSpaceWeightHeatmap")
            .expect("wide Blend Space should expose the weight heatmap in the native model");
        assert_eq!(heatmap.component_variant.as_str(), "weight-heatmap");
        assert_eq!(
            (
                heatmap.weight_heatmap.generation.columns(),
                heatmap.weight_heatmap.generation.rows()
            ),
            (16, 10)
        );
        assert_eq!(heatmap.weight_heatmap.generation.sources().len(), 5);
        assert!(
            heatmap
                .weight_heatmap
                .generation
                .sources()
                .iter()
                .any(|source| source.selected() && source.weight() == 1.0),
            "wide Blend Space heatmap should preserve the selected authored source"
        );
    }
    eprintln!(
        "blend-space-native-chain width={width}: host=({}, parent={}), workspace=({}, parent={}), canvas=({}, parent={}), grid=({}, parent={}), total_nodes={}, root_overlay_nodes={}",
        host.node_id,
        host.parent_node_id,
        workspace.node_id,
        workspace.parent_node_id,
        canvas.node_id,
        canvas.parent_node_id,
        grid.node_id,
        grid.parent_node_id,
        nodes.len(),
        presentation.root_template_nodes.row_count(),
    );

    for control_id in [
        "WorkbenchExtensionBlendSpaceWorkspace",
        "WorkbenchExtensionBlendSpaceSearch",
        "WorkbenchExtensionBlendSpaceSampleCanvas",
        "WorkbenchExtensionBlendSpaceSampleGrid",
        "WorkbenchExtensionBlendSpaceApplyButton",
    ] {
        assert!(
            native_node_reaches_ancestor(&nodes, control_id, host.node_id.as_str()),
            "{control_id} must remain inside the extension host's native parent chain"
        );
    }

    let painted = paint_host_frame_for_test(width, height, &presentation);
    let direct_template = paint_template_nodes_for_test_with_background(
        width,
        height,
        [241, 17, 193, 255],
        presentation.workbench_window_nodes.clone(),
    );
    let isolated_extension = paint_componentized_extension_workspace_for_test(
        width,
        height,
        [241, 17, 193, 255],
        &presentation,
    );
    let paint_evidence = [
        ("WorkbenchExtensionBlendSpaceSearch", 4usize),
        ("WorkbenchExtensionBlendSpaceSampleGrid", 8usize),
        ("WorkbenchExtensionBlendSpaceApplyButton", 4usize),
    ]
    .map(|(control_id, minimum_distinct_colors)| {
        let node = nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("{control_id} should exist in the native model"));
        (
            control_id,
            minimum_distinct_colors,
            distinct_frame_color_count(&painted, width, &node.frame),
            distinct_frame_color_count(&direct_template, width, &node.frame),
            distinct_frame_color_count(&isolated_extension, width, &node.frame),
            node,
        )
    });
    for (control_id, _, distinct_colors, direct_colors, isolated_colors, node) in &paint_evidence {
        let row = nodes
            .iter()
            .position(|candidate| candidate.node_id == node.node_id)
            .unwrap_or(usize::MAX);
        let parent_row = nodes
            .iter()
            .position(|candidate| candidate.node_id == node.parent_node_id)
            .unwrap_or(usize::MAX);
        eprintln!(
            "blend-space-native-paint width={width}: control={control_id}, row={row}, parent_row={parent_row}, colors={distinct_colors}, direct_colors={direct_colors}, isolated_colors={isolated_colors}, role={}, component_role={}, surface={}, button={}, text='{}', value='{}', frame=({}, {}, {}, {}), has_clip={}, clip=({}, {}, {}, {})",
            node.role,
            node.component_role,
            node.surface_variant,
            node.button_variant,
            node.text,
            node.value_text,
            node.frame.x,
            node.frame.y,
            node.frame.width,
            node.frame.height,
            node.has_clip_frame,
            node.clip_frame.x,
            node.clip_frame.y,
            node.clip_frame.width,
            node.clip_frame.height,
        );
    }
    for (control_id, minimum_distinct_colors, distinct_colors, _, _, node) in paint_evidence {
        assert!(
            distinct_colors >= minimum_distinct_colors,
            "{control_id} must paint native surface/text detail: distinct_colors={distinct_colors}, frame=({}, {}, {}, {})",
            node.frame.x,
            node.frame.y,
            node.frame.width,
            node.frame.height,
        );
    }
}
