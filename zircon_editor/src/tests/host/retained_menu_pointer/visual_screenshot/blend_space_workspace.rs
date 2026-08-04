use super::*;
use std::collections::BTreeSet;
use zircon_runtime_interface::ui::binding::UiEventKind;

mod composite_contracts;
mod preview_viewport;
mod property_editor_rows;
mod responsive_layout;
mod toolbar_surface;
mod transport_actions;

const BLEND_SPACE_NARROW_ARTIFACT: &str = "editor-window-m3-blend-space-workbench-640x520.png";
const BLEND_SPACE_COMPACT_ARTIFACT: &str = "editor-window-m3-blend-space-workbench-900x620.png";
const BLEND_SPACE_WIDE_ARTIFACT: &str = "editor-window-m3-blend-space-workbench-1260x780.png";

#[test]
fn workbench_caption_owns_unreal_compact_typography_contract() {
    let runtime_caption_size =
        zircon_runtime_interface::ui::design_tokens::EditorDesignTokens::workbench_dark()
            .typography
            .caption_size;
    assert!(
        (runtime_caption_size - 10.666667).abs() <= 0.000_01,
        "Workbench caption must project the Runtime text caption metric: {runtime_caption_size}"
    );

    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/components/workbench/primitives/data/workbench_caption.zui"),
    )
    .expect("Workbench caption primitive should be readable");

    for required in [
        "[components.WorkbenchCaption]",
        "component = \"Label\"",
        "font_size = \"$editor.typography.caption.size\"",
        "font_weight = \"$editor.typography.strong.weight\"",
        "text_tone = \"secondary\"",
        "height = { min = 18.0, preferred = 20.0, max = 22.0, stretch = \"Fixed\" }",
    ] {
        assert!(
            source.contains(required),
            "missing shared Unreal compact-caption contract: {required}"
        );
    }
    assert!(
        !source.contains("foreground_color ="),
        "Workbench caption tone must resolve through the shared palette instead of a local RGB override"
    );
}

#[test]
fn blend_space_asset_declares_dense_adaptive_component_structure() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/modules/extensions/animation/\
             workbench_extension_blend_space_workspace.zui",
    ))
    .expect("Blend Space workspace asset should be readable");
    let details = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/animation/\
             workbench_blend_space_details.zui",
    ))
    .expect("shared Blend Space details composite should be readable");

    for required in [
        "component = \"WorkbenchSearchInput\"",
        "component = \"WorkbenchDivider\"",
        "component = \"WorkbenchCaption\"",
        "component = \"WorkbenchSampleGrid\"",
        "component = \"WorkbenchWeightHeatmap\"",
        "component = \"WorkbenchTimelineStrip\"",
        "component = \"WorkbenchSampleWeights\"",
        "component = \"WorkbenchValidationLog\"",
        "component = \"WorkbenchPreviewViewport\"",
        "component = \"WorkbenchBlendSpaceDetails\"",
        "control_id = \"WorkbenchExtensionBlendSpaceCenterWorkArea\"",
        "control_id = \"WorkbenchExtensionBlendSpaceTabs\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleCanvas\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleGrid\"",
        "control_id = \"WorkbenchExtensionBlendSpaceWeightHeatmap\"",
        "control_id = \"WorkbenchExtensionBlendSpacePreviewCard\"",
        "control_id = \"WorkbenchExtensionBlendSpaceOutputPanel\"",
        "control_id = \"WorkbenchExtensionBlendSpacePreviewTimeline\"",
        "control_id = \"WorkbenchExtensionBlendSpaceBottomCompositeRow\"",
        "responsive_min_tier = \"regular\"",
    ] {
        assert!(
            source.contains(required),
            "missing dense workspace contract: {required}"
        );
    }
    for required in [
        "component = \"WorkbenchPropertyRow\"",
        "component = \"WorkbenchTableRow\"",
        "component = \"WorkbenchSectionTitle\"",
    ] {
        assert!(
            details.contains(required),
            "missing extracted dense Details contract: {required}"
        );
    }
    assert!(source.contains("[nodes.blend_space_workspace]\ncomponent = \"HorizontalGroup\""));
    assert!(source.contains("[nodes.blend_space_left]\ncomponent = \"VerticalGroup\""));
    assert!(source.contains("[nodes.blend_space_right]\ncomponent = \"VerticalGroup\""));
    assert!(
        source.matches("stretch = \"Stretch\"").count() >= 24,
        "dense workspace should be governed primarily by relative stretch layout"
    );
    for bounded_width in [
        "width = { min = 188.0, preferred = 220.0, max = 250.0, stretch = \"Fixed\" }",
        "width = { min = 176.0, preferred = 208.0, max = 240.0, stretch = \"Fixed\" }",
        "width = { min = 210.0, preferred = 260.0, max = 310.0, stretch = \"Fixed\" }",
    ] {
        assert!(
            source.contains(bounded_width),
            "bounded side panes must use the shared Fixed axis contract: {bounded_width}"
        );
    }
    assert!(
        source
            .lines()
            .filter(|line| line.trim_start().starts_with("layout ="))
            .all(|line| !line.contains("weight =")),
        "Blend Space layouts must not introduce a private flex-weight dialect outside the shared schema"
    );
    assert!(source.contains(
        "[nodes.blend_space_center]\ncomponent = \"VerticalGroup\"\ncontrol_id = \"WorkbenchExtensionBlendSpaceCenterPanel\""
    ));
    assert!(source.contains(
        "[nodes.blend_space_sample_canvas]\ncomponent = \"VerticalGroup\"\ncontrol_id = \"WorkbenchExtensionBlendSpaceSampleCanvas\""
    ));
    assert!(source.contains(
        "[nodes.blend_space_sample_grid]\ncomponent = \"WorkbenchSampleGrid\"\ncontrol_id = \"WorkbenchExtensionBlendSpaceSampleGrid\""
    ));
    for typed_grid_contract in [
        "x_min = -180.0",
        "x_max = 180.0",
        "y_min = 0.0",
        "y_max = 600.0",
        "x_ticks = [-180.0, -135.0, -90.0, -45.0, 0.0, 45.0, 90.0, 135.0, 180.0]",
        "label = \"Run_Fwd\", selected = true",
    ] {
        assert!(
            source.contains(typed_grid_contract),
            "Blend Space must author typed sample-grid data: {typed_grid_contract}"
        );
    }
    for typed_heatmap_contract in [
        "heatmap_columns = 16",
        "heatmap_rows = 10",
        "heat_sources = [{ x = 0.5, y = 0.58, weight = 1.0, selected = true }",
    ] {
        assert!(
            source.contains(typed_heatmap_contract),
            "Blend Space must author typed weight-heatmap data: {typed_heatmap_contract}"
        );
    }
    for removed_placeholder in [
        "WorkbenchExtensionBlendSpaceForwardPoint",
        "WorkbenchExtensionBlendSpaceNeutralRow",
        "WorkbenchExtensionBlendSpaceBackwardPoint",
        "zircon_engine_style/scene/skeleton.svg",
        "control_id = \"WorkbenchExtensionBlendSpaceOutputLog\"",
        "control_id = \"WorkbenchExtensionBlendSpaceIdleSampleTableRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceWalkSampleTableRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceRunSampleTableRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceDiagonalSampleTableRow\"",
    ] {
        assert!(
            !source.contains(removed_placeholder),
            "sample grid must not fall back to list-row placeholder geometry: {removed_placeholder}"
        );
    }
    assert!(!source.contains("corner_radius = 6.0"));
    for forbidden in [
        "background_color =",
        "border_color =",
        "corner_radius =",
        "font_size =",
        "font_weight =",
        "text = \"/|\\\\\\n |\\n/ \\\\\"",
    ] {
        assert!(
            !source.contains(forbidden),
            "Blend Space composites must consume shared surface/image primitives instead of local visual overrides: {forbidden}"
        );
    }
}

#[test]
fn blend_space_preview_timeline_uses_shared_typed_canvas_and_preserves_actions() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/modules/extensions/animation/\
             workbench_extension_blend_space_workspace.zui",
    ))
    .expect("Blend Space workspace asset should be readable");
    let primitive = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/primitives/data/workbench_timeline_strip.zui",
    ))
    .expect("timeline strip primitive should be readable");
    let details = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/animation/\
             workbench_blend_space_details.zui",
    ))
    .expect("shared Blend Space details composite should be readable");

    for required in [
        "[components.WorkbenchTimelineStrip]",
        "component = \"Canvas\"",
        "component_variant = \"timeline-strip\"",
        "duration = 3.0",
        "current_time = 3.0",
        "tick_interval = 0.5",
        "track_label = \"Run_Fwd\"",
        "time = 2.0, label = \"Run_Fwd\", selected = true",
    ] {
        assert!(
            primitive.contains(required) || source.contains(required),
            "missing typed preview-timeline contract: {required}"
        );
    }
    for preserved_action in [
        "route = \"workbench.extension.blend_space.samples_tab.select\"",
        "route = \"workbench.extension.blend_space.axes_tab.select\"",
        "route = \"workbench.extension.blend_space.preview_tab.select\"",
        "route = \"workbench.extension.blend_space.idle_run_row.select\"",
        "route = \"workbench.extension.blend_space.strafe_row.select\"",
        "route = \"workbench.extension.blend_space.sprint_row.select\"",
        "route = \"workbench.extension.blend_space.output.select\"",
        "route = \"workbench.extension.blend_space.preview.invoke\"",
        "route = \"workbench.extension.blend_space.apply.invoke\"",
        "route = \"workbench.extension.blend_space.idle_sample_table_row.select\"",
        "route = \"workbench.extension.blend_space.walk_sample_table_row.select\"",
        "route = \"workbench.extension.blend_space.run_sample_table_row.select\"",
        "route = \"workbench.extension.blend_space.diagonal_sample_table_row.select\"",
    ] {
        assert!(
            source.contains(preserved_action) || details.contains(preserved_action),
            "timeline/sample refactor must preserve existing interaction route: {preserved_action}"
        );
    }
    for deprecated_route in [
        "route = \"workbench.extension.blend_space.samples\"",
        "route = \"workbench.extension.blend_space.axes\"",
        "route = \"workbench.extension.blend_space.preview_tab\"",
        "route = \"workbench.extension.blend_space.asset.idle_run\"",
        "route = \"workbench.extension.blend_space.asset.strafe\"",
        "route = \"workbench.extension.blend_space.asset.sprint\"",
        "route = \"workbench.extension.blend_space.output\"",
        "route = \"workbench.extension.blend_space.preview\"",
        "route = \"workbench.extension.blend_space.apply\"",
    ] {
        assert!(
            !source.contains(deprecated_route),
            "workspace must use its canonical binding action, not {deprecated_route}"
        );
    }
}

#[test]
fn blend_space_wide_details_include_dense_sample_rows() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/modules/extensions/animation/\
             workbench_extension_blend_space_workspace.zui",
    ))
    .expect("Blend Space workspace asset should be readable");
    let details = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/animation/\
             workbench_blend_space_details.zui",
    ))
    .expect("shared Blend Space details composite should be readable");

    for required in [
        "[components.WorkbenchBlendSpaceDetails]",
        "component = \"WorkbenchSectionTitle\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSamplesTitle\"",
        "text = \"SAMPLES (8)\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleRunForwardRow\"",
        "options = [\"Run_Fwd\", \"0\", \"600\", \"1.00\"]",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleStrafeLeftRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleStrafeRightRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleIdleRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceAxisGroupTitle\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleDetailTitle\"",
    ] {
        assert!(
            details.contains(required),
            "missing reference-density sample table contract: {required}"
        );
    }
    for required in [
        "workbench_blend_space_details.zui#WorkbenchBlendSpaceDetails",
        "component = \"WorkbenchBlendSpaceDetails\"",
        "control_id = \"WorkbenchExtensionBlendSpaceDetails\"",
    ] {
        assert!(
            source.contains(required),
            "wide workspace must compose the shared Details asset: {required}"
        );
    }
    for preserved_route in [
        "workbench.extension.blend_space.run_sample_table_row.select",
        "workbench.extension.blend_space.walk_sample_table_row.select",
        "workbench.extension.blend_space.diagonal_sample_table_row.select",
        "workbench.extension.blend_space.idle_sample_table_row.select",
        "workbench.extension.blend_space.asset.edit",
        "workbench.extension.blend_space.asset.commit",
        "workbench.extension.blend_space.x_axis.edit",
        "workbench.extension.blend_space.x_axis.commit",
        "workbench.extension.blend_space.interpolation.edit",
        "workbench.extension.blend_space.interpolation.commit",
    ] {
        assert!(
            details.contains(preserved_route),
            "Details extraction must preserve the authored route: {preserved_route}"
        );
    }
    for deprecated_route in [
        "workbench.extension.blend_space.sample.run",
        "workbench.extension.blend_space.sample.walk",
        "workbench.extension.blend_space.sample.diagonal",
        "workbench.extension.blend_space.sample.idle",
    ] {
        assert!(
            !details.contains(deprecated_route),
            "Details extraction must use its canonical binding action, not {deprecated_route}"
        );
    }
    for forbidden in [
        "background_color =",
        "border_color =",
        "foreground_color =",
        "font_size =",
        "font_weight =",
    ] {
        assert!(
            !details.contains(forbidden),
            "Details composite must inherit shared primitive visuals: {forbidden}"
        );
    }
}

#[test]
fn blend_space_workspace_adapts_between_compact_and_wide_windows() {
    let compact = open_blend_space_bridge(900, 620);
    let wide = open_blend_space_bridge(1260, 780);

    assert_compact_blend_space_geometry(&compact);
    assert_wide_blend_space_geometry(&wide);

    let compact_center = required_frame(&compact, "WorkbenchExtensionBlendSpaceCenterPanel");
    let wide_center = required_frame(&wide, "WorkbenchExtensionBlendSpaceCenterPanel");
    assert!(
        compact_center.width >= 560.0,
        "compact tier should preserve a useful primary editor lane: {compact_center:?}"
    );
    assert!(
        wide_center.width >= compact_center.width,
        "wide tier should add secondary panes without shrinking the primary editor lane: compact={compact_center:?}, wide={wide_center:?}"
    );
}

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

fn assert_blend_space_native_parent_chain_and_paint(ui: &UiHostWindow, width: u32, height: u32) {
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
        assert_eq!(timeline.timeline_strip.duration, 3.0);
        assert_eq!(timeline.timeline_strip.current_time, 3.0);
        assert_eq!(timeline.timeline_strip.tick_interval, 0.5);
        assert_eq!(timeline.timeline_strip.track_label.as_str(), "Run_Fwd");
        assert_eq!(timeline.timeline_strip.keys.row_count(), 2);
        assert!(
            (0..timeline.timeline_strip.keys.row_count()).any(|row| {
                timeline
                    .timeline_strip
                    .keys
                    .row_data(row)
                    .is_some_and(|key| key.selected && key.time == 2.0)
            }),
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
            (heatmap.weight_heatmap.columns, heatmap.weight_heatmap.rows),
            (16, 10)
        );
        assert_eq!(heatmap.weight_heatmap.sources.row_count(), 5);
        assert!(
            (0..heatmap.weight_heatmap.sources.row_count()).any(|row| {
                heatmap
                    .weight_heatmap
                    .sources
                    .row_data(row)
                    .is_some_and(|source| source.selected && source.weight == 1.0)
            }),
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

fn native_node_reaches_ancestor(
    nodes: &[TemplatePaneNodeData],
    control_id: &str,
    ancestor_node_id: &str,
) -> bool {
    let nodes_by_id = nodes
        .iter()
        .map(|node| (node.node_id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let Some(node) = nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)
    else {
        return false;
    };
    let mut current_node_id = node.node_id.as_str();
    let mut visited = BTreeSet::new();
    loop {
        if current_node_id == ancestor_node_id {
            return true;
        }
        if !visited.insert(current_node_id) {
            return false;
        }
        let Some(current) = nodes_by_id.get(current_node_id) else {
            return false;
        };
        if current.parent_node_id.is_empty() {
            return false;
        }
        current_node_id = current.parent_node_id.as_str();
    }
}

fn distinct_frame_color_count(
    bytes: &[u8],
    frame_width: u32,
    frame: &TemplateNodeFrameData,
) -> usize {
    let left = frame.x.max(0.0).floor() as u32;
    let top = frame.y.max(0.0).floor() as u32;
    let right = (frame.x + frame.width).max(0.0).ceil() as u32;
    let bottom = (frame.y + frame.height).max(0.0).ceil() as u32;
    let mut colors = BTreeSet::new();
    for y in top..bottom {
        for x in left..right {
            let offset = ((y * frame_width + x) * 4) as usize;
            if let Some(pixel) = bytes.get(offset..offset + 4) {
                colors.insert([pixel[0], pixel[1], pixel[2], pixel[3]]);
            }
        }
    }
    colors.len()
}

fn assert_compact_blend_space_geometry(bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge) {
    let workspace = required_frame(bridge, "WorkbenchExtensionBlendSpaceWorkspace");
    let work_area = required_frame(bridge, "WorkbenchExtensionBlendSpaceCenterWorkArea");
    let output = required_frame(bridge, "WorkbenchExtensionBlendSpaceOutputPanel");
    let left = required_frame(bridge, "WorkbenchExtensionBlendSpaceLeftPanel");
    let center = required_frame(bridge, "WorkbenchExtensionBlendSpaceCenterPanel");
    let canvas = required_frame(bridge, "WorkbenchExtensionBlendSpaceSampleCanvas");
    let grid = required_frame(bridge, "WorkbenchExtensionBlendSpaceSampleGrid");
    let timeline = required_frame(bridge, "WorkbenchExtensionBlendSpacePreviewTimeline");

    assert!(left.x >= workspace.x && left.right() <= center.x + 0.5);
    assert!(center.right() <= workspace.right() + 0.5);
    assert!(canvas.x >= work_area.x && canvas.right() <= work_area.right() + 0.5);
    assert!(grid.x >= canvas.x && grid.right() <= canvas.right() + 0.5);
    assert!(grid.y >= canvas.y && grid.bottom() <= canvas.bottom() + 0.5);
    assert!(work_area.bottom() <= output.y + 0.5);
    assert!(output.bottom() <= center.bottom() + 0.5);
    assert!(timeline.x >= output.x && timeline.right() <= output.right() + 0.5);
    assert!(timeline.y >= output.y && timeline.bottom() <= output.bottom() + 0.5);
    assert!(
        bridge
            .control_frame("WorkbenchExtensionBlendSpaceRightPanel")
            .is_none()
    );
    assert!(
        bridge
            .control_frame("WorkbenchExtensionBlendSpacePreviewCard")
            .is_none()
    );
    assert!(
        bridge
            .control_frame("WorkbenchExtensionBlendSpacePreviewButton")
            .is_none()
    );
    assert!(
        bridge
            .control_frame("WorkbenchExtensionBlendSpaceSampleWeights")
            .is_none()
    );
    assert!(
        bridge
            .control_frame("WorkbenchExtensionBlendSpaceValidationLog")
            .is_none()
    );
    assert!(
        canvas.height > 150.0,
        "compact sample canvas should preserve a useful plotting area beside the visible timeline: {canvas:?}"
    );

    for control_id in [
        "WorkbenchExtensionBlendSpaceSearch",
        "WorkbenchExtensionBlendSpaceSamplesTab",
        "WorkbenchExtensionBlendSpaceApplyButton",
    ] {
        let control = required_frame(bridge, control_id);
        assert!(
            (28.0..=32.0).contains(&control.height),
            "{control_id} should stay on the 28-32 px control rhythm: {control:?}"
        );
    }
}

fn assert_wide_blend_space_geometry(bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge) {
    let workspace = required_frame(bridge, "WorkbenchExtensionBlendSpaceWorkspace");
    let work_area = required_frame(bridge, "WorkbenchExtensionBlendSpaceCenterWorkArea");
    let output = required_frame(bridge, "WorkbenchExtensionBlendSpaceOutputPanel");
    let left = required_frame(bridge, "WorkbenchExtensionBlendSpaceLeftPanel");
    let center = required_frame(bridge, "WorkbenchExtensionBlendSpaceCenterPanel");
    let right = required_frame(bridge, "WorkbenchExtensionBlendSpaceRightPanel");
    let canvas = required_frame(bridge, "WorkbenchExtensionBlendSpaceSampleCanvas");
    let grid = required_frame(bridge, "WorkbenchExtensionBlendSpaceSampleGrid");
    let preview = required_frame(bridge, "WorkbenchExtensionBlendSpacePreviewCard");
    let timeline = required_frame(bridge, "WorkbenchExtensionBlendSpacePreviewTimeline");
    let sample_weights = required_frame(bridge, "WorkbenchExtensionBlendSpaceSampleWeights");
    let validation_log = required_frame(bridge, "WorkbenchExtensionBlendSpaceValidationLog");

    assert!(left.x >= workspace.x && left.right() <= center.x + 0.5);
    assert!(center.right() <= right.x + 0.5);
    assert!(right.right() <= workspace.right() + 0.5);
    assert!(canvas.x >= work_area.x && canvas.right() <= preview.x + 0.5);
    assert!(grid.x >= canvas.x && grid.right() <= canvas.right() + 0.5);
    assert!(grid.y >= canvas.y && grid.bottom() <= canvas.bottom() + 0.5);
    assert!(preview.right() <= work_area.right() + 0.5);
    assert!(work_area.bottom() <= output.y + 0.5);
    assert!(output.bottom() <= center.bottom() + 0.5);
    assert!(timeline.x >= output.x && timeline.right() <= output.right() + 0.5);
    assert!(timeline.y >= output.y && timeline.bottom() <= output.bottom() + 0.5);
    assert!(output.right() <= sample_weights.x + 0.5);
    assert!(sample_weights.right() <= validation_log.x + 0.5);
    assert!(validation_log.right() <= center.right() + 0.5);
    assert!((sample_weights.y - output.y).abs() <= 0.5);
    assert!((validation_log.y - output.y).abs() <= 0.5);
    assert!((sample_weights.height - output.height).abs() <= 0.5);
    assert!((validation_log.height - output.height).abs() <= 0.5);
    assert!(
        output.width >= sample_weights.width,
        "wide timeline should keep at least the sample-weights width after diagnostic panes mount: output={output:?}, weights={sample_weights:?}"
    );
    assert!(
        output.width >= 246.0,
        "wide timeline should keep enough relative space for six Unreal-density transport controls: {output:?}"
    );
    assert!(canvas.width > preview.width * 1.35);
    assert!(
        canvas.height > 150.0,
        "wide sample canvas should preserve a useful plotting area beside the visible timeline: {canvas:?}"
    );

    for control_id in [
        "WorkbenchExtensionBlendSpaceSearch",
        "WorkbenchExtensionBlendSpaceSamplesTab",
        "WorkbenchExtensionBlendSpacePreviewButton",
        "WorkbenchExtensionBlendSpaceApplyButton",
    ] {
        let control = required_frame(bridge, control_id);
        assert!((28.0..=32.0).contains(&control.height));
    }
}

fn required_frame(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> zircon_runtime_interface::ui::layout::UiFrame {
    bridge
        .control_frame(control_id)
        .unwrap_or_else(|| panic!("{control_id} should have a visible adaptive frame"))
}

fn open_blend_space_bridge(width: u32, height: u32) -> BuiltinWorkbenchWindowTemplateSurfaceBridge {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut bridge = workbench_window_bridge_for_visual_artifact(&model, width, height);
    bridge
        .dispatch_control_state("WorkbenchAbilityBlendSpaceButton", UiEventKind::Click)
        .expect("Blend Space control should update production template state")
        .expect("Blend Space production control should expose its open binding");
    bridge
}

pub(super) fn blend_space_window(width: u32, height: u32) -> UiHostWindow {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let shell_size = ShellSizePx::new(width as f32, height as f32);
    let metrics = WorkbenchChromeMetrics::default();
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        1.0,
        &metrics,
        None,
    );
    let floating_windows = build_floating_window_projection_bundle(&model, None, &metrics, &[]);
    let mut bridge = workbench_window_bridge_for_visual_artifact(&model, width, height);
    bridge
        .dispatch_control_state("WorkbenchAbilityBlendSpaceButton", UiEventKind::Click)
        .expect("Blend Space production control should update the template")
        .expect("Blend Space production control should expose its open binding");
    for control_id in [
        "WorkbenchExtensionBlendSpaceSearch",
        "WorkbenchExtensionBlendSpaceSampleCanvas",
        "WorkbenchExtensionBlendSpaceApplyButton",
    ] {
        let node = bridge
            .host_projection()
            .nodes
            .iter()
            .find(|node| node.control_id.as_deref() == Some(control_id))
            .unwrap_or_else(|| panic!("{control_id} should exist in the retained projection"));
        eprintln!(
            "blend-space-retained-node: control={control_id}, component={}, role={:?}, text={:?}, value={:?}, properties={:?}",
            node.component, node.component_role, node.text, node.value_text, node.properties,
        );
    }

    let ui = UiHostWindow::new().expect("workbench shell should instantiate for screenshot");
    ui.show()
        .expect("workbench shell should show for screenshot");
    ui.window().set_size(PhysicalSize::new(width, height));
    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &BTreeMap::new(),
        &BTreeMap::new(),
        None,
        &ModulePluginsPaneViewData::default(),
        &BuildExportPaneViewData::default(),
        None,
        Some(bridge.host_projection()),
        bridge.layout_frames(),
        &floating_windows,
        None,
    );
    let presentation = ui.get_host_presentation();
    assert!(
        (0..presentation.workbench_window_nodes.row_count())
            .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
            .any(|node| node.control_id.as_str() == "WorkbenchExtensionBlendSpaceWorkspace"),
        "Blend Space screenshot must carry the activated extension workspace into the native host presentation"
    );
    ui
}
