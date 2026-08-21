pub(super) use crate::ui::retained_host::primitives::{ModelRc, PhysicalSize, VecModel};
pub(super) use crate::ui::retained_host::{
    build_pane_template_surface_frame, callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    to_host_contract_component_showcase_pane_from_host_pane_with_runtime, FloatingWindowData,
    FrameRect, HostChromeControlFrameData, HostChromeTabData, HostClosePromptData,
    HostDocumentDockSurfaceData, HostMenuChromeData, HostMenuChromeItemData,
    HostMenuChromeMenuData, HostMenuStateData, HostPageOverflowMenuStateData, HostResizeLayerData,
    HostSideDockSurfaceData, HostWindowLayoutData, PaneData, PaneSurfaceHostContext, SceneNodeData,
    SceneViewportChromeData, TabData, TemplateNodeFrameData, TemplatePaneNodeData, UiHostContext,
    UiHostWindow,
};
pub(super) use crate::ui::template_runtime::EditorUiHostRuntime;
pub(super) use std::{cell::RefCell, rc::Rc};
pub(super) use zircon_runtime_interface::ui::layout::UiSize;

pub(super) fn host_frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

pub(super) fn host_window_layout_for_test(width: f32, height: f32) -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: host_frame(0.0, 58.0, width, height - 82.0),
        status_bar_frame: host_frame(0.0, height - 24.0, width, 24.0),
        left_region_frame: FrameRect::default(),
        document_region_frame: host_frame(60.0, 58.0, width - 80.0, height - 82.0),
        viewport_content_frame: host_frame(60.0, 118.0, width - 80.0, height - 142.0),
        ..HostWindowLayoutData::default()
    }
}

pub(super) fn scene_pane() -> PaneData {
    PaneData {
        kind: "Scene".into(),
        title: "Scene".into(),
        show_toolbar: true,
        viewport: SceneViewportChromeData {
            mode: "Transform.Move".into(),
            transform_space: "Global".into(),
            display_mode: "Lit".into(),
            grid_mode: "Grid".into(),
            toolbar_surface_frame: Some(viewport_toolbar_surface_frame_for_test()),
            ..SceneViewportChromeData::default()
        },
        ..PaneData::default()
    }
}

pub(super) fn viewport_toolbar_surface_frame_for_test(
) -> std::sync::Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame> {
    let mut bridge = BuiltinViewportToolbarTemplateBridge::new()
        .expect("viewport toolbar template bridge should load in native host tests");
    bridge
        .recompute_layout(UiSize::new(1200.0, 28.0))
        .expect("viewport toolbar template should compute test layout");
    bridge.surface_frame_for_projection_controls(
        "document",
        UiSize::new(1200.0, 28.0),
        |projection_control_id| {
            Some(viewport_toolbar_hit_control_id_for_test(
                projection_control_id,
            ))
        },
    )
}

pub(super) fn viewport_toolbar_hit_control_id_for_test(projection_control_id: &str) -> String {
    match projection_control_id {
        "ActivateSceneMode" => "mode.move",
        "SetTransformSpace" => "space.global",
        "SetProjectionMode" => "projection.perspective",
        "AlignView" => "align.neg_z",
        "SetDisplayMode" => "display.cycle",
        "SetGridMode" => "grid.cycle",
        "SetTranslateSnap" => "snap.translate",
        "SetRotateSnapDegrees" => "snap.rotate",
        "SetScaleSnap" => "snap.scale",
        "SetPreviewLighting" => "toggle.lighting",
        "SetPreviewSkybox" => "toggle.skybox",
        "SetGizmosEnabled" => "toggle.gizmos",
        "FrameSelection" => "frame.selection",
        "EnterPlayMode" => "EnterPlayMode",
        "ExitPlayMode" => "ExitPlayMode",
        _ => projection_control_id,
    }
    .to_string()
}

pub(super) fn viewport_toolbar_control_frame(
    presentation: &crate::ui::retained_host::HostWindowPresentationData,
    control_id: &str,
) -> FrameRect {
    let toolbar_frame = presentation
        .host_scene_data
        .document_dock
        .pane
        .viewport
        .toolbar_surface_frame
        .as_ref()
        .expect("scene pane should carry a viewport toolbar surface frame");
    let arranged = toolbar_frame
        .arranged_tree
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some(control_id))
        .unwrap_or_else(|| panic!("missing viewport toolbar control frame for {control_id}"));
    host_frame(
        arranged.frame.x,
        arranged.frame.y,
        arranged.frame.width,
        arranged.frame.height,
    )
}

pub(super) fn pane_with_nodes(kind: &str, nodes: Vec<TemplatePaneNodeData>) -> PaneData {
    let mut pane = PaneData {
        kind: kind.into(),
        title: kind.into(),
        ..PaneData::default()
    };
    pane.project_overview.nodes = model_rc(nodes);
    pane.body_surface_frame = build_pane_template_surface_frame(&pane, UiSize::new(1000.0, 1000.0));
    pane
}

pub(super) fn assets_pane_with_nodes(nodes: Vec<TemplatePaneNodeData>) -> PaneData {
    let mut pane = PaneData {
        kind: "Assets".into(),
        title: "Assets".into(),
        ..PaneData::default()
    };
    pane.assets_activity.nodes = model_rc(nodes);
    pane.body_surface_frame = build_pane_template_surface_frame(&pane, UiSize::new(1000.0, 1000.0));
    pane
}

pub(super) fn asset_browser_pane_with_nodes(nodes: Vec<TemplatePaneNodeData>) -> PaneData {
    let mut pane = PaneData {
        kind: "AssetBrowser".into(),
        title: "Asset Browser".into(),
        ..PaneData::default()
    };
    pane.asset_browser.nodes = model_rc(nodes);
    pane.body_surface_frame = build_pane_template_surface_frame(&pane, UiSize::new(1000.0, 1000.0));
    pane
}

pub(super) fn welcome_pane_with_nodes(nodes: Vec<TemplatePaneNodeData>) -> PaneData {
    let mut pane = PaneData {
        kind: "Welcome".into(),
        title: "Welcome".into(),
        ..PaneData::default()
    };
    pane.welcome.nodes = model_rc(nodes);
    pane.body_surface_frame = build_pane_template_surface_frame(&pane, UiSize::new(1000.0, 1000.0));
    pane
}

pub(super) fn component_showcase_pane_with_runtime_projection(
    runtime: &EditorUiHostRuntime,
    width: f32,
    height: f32,
) -> PaneData {
    use crate::ui::layouts::windows::workbench_host_window as host_window;
    use crate::ui::workbench::view::{
        PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace,
    };

    let fixture = crate::ui::workbench::fixture::default_preview_fixture();
    let chrome = fixture.build_chrome();
    let body_spec = PaneBodySpec::new(
        "res://ui/editor/component_showcase.zui",
        PanePayloadKind::UiComponentShowcaseV1,
        PaneRouteNamespace::UiComponentShowcase,
        PaneInteractionMode::TemplateOnly,
    );
    let body = host_window::build_pane_body_presentation(
        &body_spec,
        &host_window::PanePayloadBuildContext::new(&chrome),
    );
    let workbench_pane = host_window::PaneData {
        id: "component-showcase".into(),
        slot: "component-showcase-slot".into(),
        kind: "UiComponentShowcase".into(),
        title: "UI Component Showcase".into(),
        icon_key: "ui-components".into(),
        subtitle: "Runtime components".into(),
        info: "".into(),
        show_empty: false,
        empty_title: "".into(),
        empty_body: "".into(),
        primary_action_label: "".into(),
        primary_action_id: "".into(),
        secondary_action_label: "".into(),
        secondary_action_id: "".into(),
        secondary_hint: "".into(),
        show_toolbar: false,
        viewport: crate::ui::layouts::views::blank_viewport_chrome(),
        native_body: host_window::PaneNativeBodyData {
            hierarchy: host_window::HierarchyPaneViewData::default(),
            inspector: host_window::InspectorPaneViewData::default(),
            console: host_window::ConsolePaneViewData::default(),
            assets_activity: host_window::AssetsActivityPaneViewData::default(),
            asset_browser: host_window::AssetBrowserPaneViewData::default(),
            project_overview: host_window::ProjectOverviewPaneViewData::default(),
            performance_timeline: host_window::PerformanceTimelinePaneViewData::default(),
            module_plugins: host_window::ModulePluginsPaneViewData::default(),
            build_export: host_window::BuildExportPaneViewData::default(),
            generated_bottom: host_window::GeneratedBottomPaneViewData::default(),
            ui_asset: crate::ui::asset_editor::UiAssetEditorPanePresentation::default(),
            animation: host_window::AnimationEditorPaneViewData::default(),
        },
        pane_presentation: Some(host_window::PanePresentation::new(
            host_window::PaneShellPresentation::new(
                "UI Component Showcase",
                "ui-components",
                "Runtime components",
                "",
                None,
                false,
                crate::ui::layouts::views::blank_viewport_chrome(),
            ),
            body,
        )),
    };
    let projected = to_host_contract_component_showcase_pane_from_host_pane_with_runtime(
        &workbench_pane,
        host_window::PaneContentSize::new(width, height),
        runtime,
    );

    let mut pane = PaneData {
        kind: "UiComponentShowcase".into(),
        title: "UI Component Showcase".into(),
        project_overview: projected,
        ..PaneData::default()
    };
    pane.body_surface_frame = build_pane_template_surface_frame(&pane, UiSize::new(width, height));
    pane
}

pub(super) fn template_node_by_control_id(
    pane: &PaneData,
    control_id: &str,
) -> TemplatePaneNodeData {
    (0..pane.project_overview.nodes.row_count())
        .filter_map(|row| pane.project_overview.nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("projected pane should expose {control_id}"))
}

pub(super) fn hierarchy_pane(nodes: Vec<SceneNodeData>) -> PaneData {
    let mut pane = PaneData {
        kind: "Hierarchy".into(),
        title: "Hierarchy".into(),
        ..PaneData::default()
    };
    pane.hierarchy.hierarchy_nodes = model_rc(nodes);
    pane
}

pub(super) fn hierarchy_pane_with_template_nodes(
    nodes: Vec<SceneNodeData>,
    template_nodes: Vec<TemplatePaneNodeData>,
) -> PaneData {
    let mut pane = hierarchy_pane(nodes);
    pane.hierarchy.nodes = model_rc(template_nodes);
    pane.body_surface_frame = build_pane_template_surface_frame(&pane, UiSize::new(1000.0, 1000.0));
    pane
}

pub(super) fn asset_tree_pane() -> PaneData {
    let mut pane = PaneData {
        kind: "Assets".into(),
        title: "Assets".into(),
        ..PaneData::default()
    };
    pane.assets_activity.nodes = model_rc(vec![template_node(
        "AssetsActivityTreeRowPanel",
        "Panel",
        "Assets",
        8.0,
        57.0,
        220.0,
        28.0,
    )]);
    pane
}

pub(super) fn scene_node(id: &str, name: &str, depth: i32, selected: bool) -> SceneNodeData {
    SceneNodeData {
        id: id.into(),
        name: name.into(),
        depth,
        selected,
    }
}

pub(super) fn template_node_with_action(
    control_id: &str,
    role: &str,
    text: &str,
    action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        action_id: action_id.into(),
        dispatch_kind: "click".into(),
        button_variant: "primary".into(),
        ..template_node(control_id, role, text, x, y, width, height)
    }
}

pub(super) fn template_node_with_binding(
    control_id: &str,
    role: &str,
    text: &str,
    binding_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        binding_id: binding_id.into(),
        button_variant: "primary".into(),
        ..template_node(control_id, role, text, x, y, width, height)
    }
}

pub(super) fn welcome_text_node(
    control_id: &str,
    edit_action_id: &str,
    value_text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        component_role: "input-field".into(),
        dispatch_kind: "welcome_text".into(),
        action_id: edit_action_id.into(),
        edit_action_id: edit_action_id.into(),
        value_text: value_text.into(),
        surface_variant: "inset".into(),
        ..template_node(control_id, "LineEdit", value_text, x, y, width, height)
    }
}

pub(super) fn template_input_node(
    control_id: &str,
    value_text: &str,
    edit_action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        component_role: "input-field".into(),
        edit_action_id: edit_action_id.into(),
        value_text: value_text.into(),
        surface_variant: "inset".into(),
        ..template_node(control_id, "InputField", value_text, x, y, width, height)
    }
}

pub(super) fn template_input_node_with_binding(
    control_id: &str,
    value_text: &str,
    binding_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        component_role: "input-field".into(),
        binding_id: binding_id.into(),
        value_text: value_text.into(),
        surface_variant: "inset".into(),
        ..template_node(control_id, "InputField", value_text, x, y, width, height)
    }
}

pub(super) fn template_input_node_with_commit(
    control_id: &str,
    value_text: &str,
    edit_action_id: &str,
    commit_action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        commit_action_id: commit_action_id.into(),
        ..template_input_node(control_id, value_text, edit_action_id, x, y, width, height)
    }
}

pub(super) fn template_input_node_commit_only(
    control_id: &str,
    value_text: &str,
    edit_action_id: &str,
    commit_action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        dispatch_kind: "commit_only".into(),
        commit_action_id: commit_action_id.into(),
        ..template_input_node(control_id, value_text, edit_action_id, x, y, width, height)
    }
}

pub(super) fn welcome_button_node(
    control_id: &str,
    action_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        dispatch_kind: "welcome".into(),
        action_id: action_id.into(),
        button_variant: "primary".into(),
        ..template_node(control_id, "Button", text, x, y, width, height)
    }
}

pub(super) fn template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: format!("{control_id}.node").into(),
        control_id: control_id.into(),
        role: role.into(),
        text: text.into(),
        surface_variant: "panel".into(),
        border_width: 1.0,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn control_frame(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> HostChromeControlFrameData {
    HostChromeControlFrameData {
        control_id: control_id.into(),
        frame: host_frame(x, y, width, height),
    }
}

pub(super) fn chrome_tab(
    control_id: &str,
    title: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> HostChromeTabData {
    HostChromeTabData {
        control_id: control_id.into(),
        tab: tab_data(control_id, title),
        frame: host_frame(x, y, width, height),
        close_frame: host_frame(x + width - 20.0, y + 4.0, 16.0, 16.0),
    }
}

pub(super) fn tab_data(id: &str, title: &str) -> TabData {
    TabData {
        id: id.into(),
        title: title.into(),
        active: true,
        closeable: true,
        ..TabData::default()
    }
}

pub(super) fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

pub(super) fn changed_pixel_count(
    width: u32,
    left: &[u8],
    right: &[u8],
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
) -> usize {
    let x1 = x.saturating_add(region_width).min(width);
    let y1 = y
        .saturating_add(region_height)
        .min((left.len() / 4 / width as usize) as u32)
        .min((right.len() / 4 / width as usize) as u32);
    (y..y1)
        .flat_map(|row| (x..x1).map(move |column| (column, row)))
        .filter(|(column, row)| {
            pixel(width, left, *column, *row) != pixel(width, right, *column, *row)
        })
        .count()
}

pub(super) fn pixel(width: u32, bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}
