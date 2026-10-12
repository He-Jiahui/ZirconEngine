// 提供混合空间测试的窗口桥、帧查找、祖先链及宽窄几何断言。
use super::*;
use std::collections::BTreeSet;
use zircon_runtime_interface::ui::binding::UiEventKind;

pub(super) const BLEND_SPACE_NARROW_ARTIFACT: &str =
    "editor-window-m3-blend-space-workbench-640x520.png";
pub(super) const BLEND_SPACE_COMPACT_ARTIFACT: &str =
    "editor-window-m3-blend-space-workbench-900x620.png";
pub(super) const BLEND_SPACE_WIDE_ARTIFACT: &str =
    "editor-window-m3-blend-space-workbench-1260x780.png";

pub(super) fn native_node_reaches_ancestor(
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

pub(super) fn distinct_frame_color_count(
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

pub(super) fn assert_compact_blend_space_geometry(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
) {
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
    assert!(bridge
        .control_frame("WorkbenchExtensionBlendSpaceRightPanel")
        .is_none());
    assert!(bridge
        .control_frame("WorkbenchExtensionBlendSpacePreviewCard")
        .is_none());
    assert!(bridge
        .control_frame("WorkbenchExtensionBlendSpacePreviewButton")
        .is_none());
    assert!(bridge
        .control_frame("WorkbenchExtensionBlendSpaceSampleWeights")
        .is_none());
    assert!(bridge
        .control_frame("WorkbenchExtensionBlendSpaceValidationLog")
        .is_none());
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

pub(super) fn assert_wide_blend_space_geometry(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
) {
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

pub(super) fn required_frame(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> zircon_runtime_interface::ui::layout::UiFrame {
    bridge
        .control_frame(control_id)
        .unwrap_or_else(|| panic!("{control_id} should have a visible adaptive frame"))
}

pub(super) const BLEND_SPACE_SOURCE_URI: &str =
    "res://ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui";

fn activate_blend_space_workspace(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge) {
    bridge
        .dispatch_control_state("MenuSlot2", UiEventKind::Click)
        .expect("Window menu should open through its live template control")
        .expect("Window menu should expose its typed popup binding");
    bridge
        .dispatch_workbench_module_overflow_menu_item_state(
            "UnrealWindowMenuPopup",
            "menu.item.ability",
        )
        .expect("Window menu should mount the selected Ability workspace")
        .expect("Ability menu item should expose its real module selection binding");
    assert!(
        bridge.has_control("WorkbenchAbilityBlendSpaceButton"),
        "the selected Ability workspace must own the live Blend Space gateway"
    );
    bridge
        .dispatch_control_state("WorkbenchAbilityBlendSpaceButton", UiEventKind::Click)
        .expect("Blend Space gateway should mount its canonical extension workspace")
        .expect("Blend Space gateway should expose its existing typed open binding");

    let nodes = crate::ui::retained_host::to_host_contract_workbench_window_nodes(Some(
        bridge.host_projection(),
    ))
    .iter()
    .cloned()
    .collect::<Vec<_>>();
    assert_blend_space_native_source_parent_chain(&nodes, &bridge.surface().surface_frame());
}

pub(super) fn assert_blend_space_native_source_parent_chain(
    nodes: &[TemplatePaneNodeData],
    current_frame: &Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>,
) {
    let pane = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchMainBandViewportPanel")
        .expect("the actual authored viewport panel must own the mounted workspace");
    let viewport = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "UnrealSceneViewViewport")
        .expect("the actual Unreal viewport must contain the selected workspace");
    let workspace = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchExtensionBlendSpaceWorkspace")
        .expect("the canonical Blend Space workspace must be mounted");
    assert_eq!(workspace.parent_node_id, viewport.node_id);
    assert!(
        native_node_reaches_ancestor(nodes, "UnrealSceneViewViewport", pane.node_id.as_str()),
        "the actual Unreal viewport must remain inside its authored viewport panel"
    );

    let source_path = workspace.source_path.as_str().replace('\\', "/");
    let source_relative = source_path
        .strip_prefix("zircon_editor/assets/")
        .expect("the mounted Source must name the physical current editor asset");
    assert_eq!(format!("res://{source_relative}"), BLEND_SPACE_SOURCE_URI);
    assert_eq!(workspace.source_node_id.as_str(), "blend_space_workspace");
    let viewport_id = viewport
        .surface_node_id
        .expect("the viewport must retain its actual Runtime node identity");
    let workspace_id = workspace
        .surface_node_id
        .expect("the workspace must retain its actual Runtime node identity");
    assert_eq!(
        current_frame
            .arranged_tree
            .get(workspace_id)
            .expect("the mounted root must exist in the current arranged tree")
            .parent,
        Some(viewport_id),
    );

    for control_id in [
        "UnrealSceneViewViewport",
        "WorkbenchExtensionBlendSpaceWorkspace",
        "WorkbenchExtensionBlendSpaceSearch",
        "WorkbenchExtensionBlendSpaceSampleCanvas",
        "WorkbenchExtensionBlendSpaceSampleGrid",
        "WorkbenchExtensionBlendSpaceApplyButton",
    ] {
        let node = nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("{control_id} must exist in the native model"));
        let source = node
            .source_surface_frame
            .as_ref()
            .unwrap_or_else(|| panic!("{control_id} must retain its actual source frame"));
        assert!(
            Arc::ptr_eq(source, current_frame),
            "{control_id} must publish the current bridge Surface frame"
        );
        let node_id = node
            .surface_node_id
            .unwrap_or_else(|| panic!("{control_id} must retain its Runtime node ID"));
        assert!(
            current_frame.arranged_tree.get(node_id).is_some(),
            "{control_id} must refer to a live node in that same published frame"
        );
    }
}

pub(super) fn open_blend_space_bridge(
    width: u32,
    height: u32,
) -> BuiltinWorkbenchWindowTemplateSurfaceBridge {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut bridge = workbench_window_bridge_for_visual_artifact(&model, width, height);
    activate_blend_space_workspace(&mut bridge);
    bridge
}

pub(in super::super) fn blend_space_window(width: u32, height: u32) -> UiHostWindow {
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
    activate_blend_space_workspace(&mut bridge);
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
        Default::default(),
    );
    let presentation = ui.get_host_presentation();
    let nodes = (0..presentation.workbench_window_nodes.row_count())
        .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
        .collect::<Vec<_>>();
    assert_blend_space_native_source_parent_chain(&nodes, &bridge.surface().surface_frame());
    ui
}
