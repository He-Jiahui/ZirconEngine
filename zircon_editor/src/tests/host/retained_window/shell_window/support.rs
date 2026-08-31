pub(super) use std::rc::Rc;

pub(super) use crate::ui::retained_host::primitives::{
    ModelRc, PhysicalSize, SharedString, VecModel,
};
pub(super) use crate::ui::retained_host::{
    build_pane_template_surface_frame, compile_welcome_pane_layout,
    paint_runtime_render_commands_for_test, FloatingWindowData, FrameRect,
    HostBottomDockSurfaceData, HostChromeControlFrameData, HostDocumentDockSurfaceData,
    HostFloatingWindowLayerData, HostMenuChromeData, HostMenuChromeMenuData, HostMenuStateData,
    HostSideDockSurfaceData, HostStatusBarData, HostWindowLayoutData, HostWindowShellData,
    NewProjectFormData, PaneData, RecentProjectData, SceneViewportChromeData,
    TemplateNodeFrameData, TemplatePaneNodeData, UiHostContext, UiHostWindow, WelcomePaneData,
    STARTUP_REFRESH_DIAGNOSTICS_OVERLAY,
};
pub(super) use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiSize},
    surface::{
        UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiTextAlign, UiTextRenderMode,
        UiTextWrap, UiVisualAssetRef,
    },
};

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
        left_region_frame: host_frame(0.0, 58.0, 72.0, height - 82.0),
        document_region_frame: host_frame(72.0, 58.0, width - 72.0, height - 82.0),
        viewport_content_frame: host_frame(88.0, 90.0, width - 104.0, height - 124.0),
        ..HostWindowLayoutData::default()
    }
}

pub(super) fn scene_test_layout() -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: host_frame(0.0, 58.0, 420.0, 178.0),
        status_bar_frame: host_frame(0.0, 236.0, 420.0, 24.0),
        left_region_frame: host_frame(0.0, 58.0, 76.0, 178.0),
        document_region_frame: host_frame(76.0, 58.0, 250.0, 178.0),
        right_region_frame: host_frame(326.0, 58.0, 94.0, 178.0),
        bottom_region_frame: host_frame(76.0, 202.0, 250.0, 34.0),
        viewport_content_frame: host_frame(76.0, 90.0, 250.0, 146.0),
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
            ..SceneViewportChromeData::default()
        },
        ..PaneData::default()
    }
}

pub(super) fn pane_with_nodes(kind: &str, nodes: Vec<TemplatePaneNodeData>) -> PaneData {
    let node_model = model_rc(nodes);
    let mut pane = PaneData {
        kind: kind.into(),
        title: kind.into(),
        ..PaneData::default()
    };
    match kind {
        "Hierarchy" => pane.hierarchy.nodes = node_model,
        "Inspector" => pane.inspector.nodes = node_model,
        "Console" => pane.console.nodes = node_model,
        "Assets" => pane.assets_activity.nodes = node_model,
        "AssetBrowser" => pane.asset_browser.nodes = node_model,
        "Project" => pane.project_overview.nodes = node_model,
        "RuntimeDiagnostics" => pane.runtime_diagnostics.nodes = node_model,
        "ModulePlugins" => pane.module_plugins.nodes = node_model,
        "BuildExport" => pane.build_export.nodes = node_model,
        "UiAssetEditor" => pane.ui_asset.nodes = node_model,
        "AnimationSequenceEditor" | "AnimationGraphEditor" => pane.animation.nodes = node_model,
        _ => {}
    }
    pane
}

pub(super) fn welcome_pane_with_content() -> PaneData {
    let mut pane = PaneData {
        kind: "Welcome".into(),
        title: "Welcome".into(),
        welcome: WelcomePaneData {
            title: "Open or Create".into(),
            subtitle: "Recent projects and a renderable empty-project template".into(),
            status_message: "No recent project".into(),
            form: NewProjectFormData {
                project_name: "ZirconProject".into(),
                location: "C:/Users/Tester/Documents/ZirconProjects".into(),
                project_path_preview: "C:/Users/Tester/Documents/ZirconProjects/ZirconProject"
                    .into(),
                template_label: "Renderable Empty".into(),
                validation_message: "Project settings are valid".into(),
                can_create: true,
                can_open_existing: true,
                browse_supported: true,
            },
            recent_projects: model_rc(vec![RecentProjectData {
                display_name: "ZirconProject4".into(),
                path: "C:/Users/Tester/Documents/ZirconProjects/ZirconProject4".into(),
                last_opened_label: "Reopened".into(),
                status_label: "".into(),
                invalid: false,
            }]),
            nodes: model_rc(vec![
                template_node("WelcomeOuterPanel", "Panel", "", 16.0, 12.0, 516.0, 220.0),
                template_node("WelcomeRecentPanel", "Panel", "", 16.0, 12.0, 180.0, 220.0),
                template_node(
                    "WelcomeRecentHeaderPanel",
                    "Panel",
                    "",
                    16.0,
                    24.0,
                    180.0,
                    54.0,
                ),
                template_node(
                    "WelcomeRecentListPanel",
                    "Panel",
                    "",
                    26.0,
                    92.0,
                    160.0,
                    130.0,
                ),
                template_node("WelcomeMainPanel", "Panel", "", 196.0, 12.0, 336.0, 220.0),
                template_node("WelcomeHeroPanel", "Panel", "", 224.0, 24.0, 280.0, 54.0),
                template_node("WelcomeStatusPanel", "Panel", "", 224.0, 84.0, 280.0, 30.0),
                template_node(
                    "WelcomeNewProjectHeaderPanel",
                    "Panel",
                    "",
                    224.0,
                    124.0,
                    280.0,
                    34.0,
                ),
                template_node(
                    "WelcomeProjectNameField",
                    "Panel",
                    "",
                    224.0,
                    162.0,
                    280.0,
                    44.0,
                ),
                template_node(
                    "WelcomeLocationField",
                    "Panel",
                    "",
                    224.0,
                    212.0,
                    280.0,
                    44.0,
                ),
                template_node(
                    "WelcomePreviewPanel",
                    "Panel",
                    "",
                    224.0,
                    262.0,
                    280.0,
                    50.0,
                ),
                template_node(
                    "WelcomeValidationPanel",
                    "Panel",
                    "",
                    224.0,
                    318.0,
                    280.0,
                    24.0,
                ),
                template_node("WelcomeActionsRow", "Panel", "", 224.0, 346.0, 280.0, 32.0),
            ]),
        },
        ..PaneData::default()
    };
    pane.welcome.layout = compile_welcome_pane_layout(&pane.welcome.nodes);
    pane
}

pub(super) fn selected_template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        selected: true,
        focused: true,
        ..template_node(control_id, role, text, x, y, width, height)
    }
}

pub(super) fn primary_template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        button_variant: "primary".into(),
        ..template_node(control_id, role, text, x, y, width, height)
    }
}

pub(super) fn disabled_template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        disabled: true,
        ..template_node(control_id, role, text, x, y, width, height)
    }
}

pub(super) fn icon_state_node(
    control_id: &str,
    x: f32,
    y: f32,
    selected: bool,
    hovered: bool,
    pressed: bool,
    disabled: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        role: "IconButton".into(),
        icon_name: "options-outline".into(),
        has_preview_image: true,
        preview_image: solid_test_icon_image(),
        selected,
        hovered,
        pressed,
        disabled,
        border_width: 1.0,
        corner_radius: 5.0,
        ..template_node(control_id, "IconButton", "", x, y, 32.0, 32.0)
    }
}

pub(super) fn solid_test_icon_image() -> crate::ui::retained_host::primitives::Image {
    let pixels = [[255, 255, 255, 255]; 4].concat();
    crate::ui::retained_host::primitives::Image::from_rgba8(
        crate::ui::retained_host::primitives::SharedPixelBuffer::<
            crate::ui::retained_host::primitives::Rgba8Pixel,
        >::clone_from_slice(&pixels, 2, 2),
    )
}

pub(super) fn muted_label_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        surface_variant: "".into(),
        border_width: 0.0,
        text_tone: "muted".into(),
        ..template_node(control_id, role, text, x, y, width, height)
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

pub(super) fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
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

pub(super) fn runtime_quad_command(
    node_id: u64,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    z_index: i32,
    background: &str,
    border: &str,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Quad,
        frame: UiFrame::new(x, y, width, height),
        clip_frame: None,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(background.to_string()),
            border_color: Some(border.to_string()),
            border_width: 1.0,
            ..runtime_style()
        },
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    }
}

pub(super) fn runtime_text_command(
    node_id: u64,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: &str,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(x, y, width, height),
        clip_frame: Some(UiFrame::new(x, y, width, height)),
        z_index: 8,
        style: UiResolvedStyle {
            foreground_color: Some("#fedcba".to_string()),
            font_size: 12.0,
            line_height: 14.0,
            text_align: UiTextAlign::Left,
            wrap: UiTextWrap::None,
            text_render_mode: UiTextRenderMode::Auto,
            ..runtime_style()
        },
        text_layout: None,
        text: Some(text.to_string()),
        image: None,
        opacity: 1.0,
    }
}

pub(super) fn lit_row_count(
    width: u32,
    bytes: &[u8],
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
) -> usize {
    let x1 = x.saturating_add(region_width).min(width);
    let y1 = y
        .saturating_add(region_height)
        .min((bytes.len() / 4 / width as usize) as u32);
    (y..y1)
        .filter(|row| (x..x1).any(|column| pixel(width, bytes, column, *row) != [0, 0, 0, 255]))
        .count()
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

pub(super) fn has_antialias_pixel(bytes: &[u8], foreground: [u8; 4]) -> bool {
    bytes.chunks_exact(4).any(|pixel| {
        let pixel = [pixel[0], pixel[1], pixel[2], pixel[3]];
        pixel != [0, 0, 0, 255] && pixel != foreground
    })
}

pub(super) fn runtime_image_command(
    node_id: u64,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> UiRenderCommand {
    runtime_image_command_with_asset(
        node_id,
        x,
        y,
        width,
        height,
        UiVisualAssetRef::Icon("options-outline".to_string()),
    )
}

pub(super) fn runtime_image_command_with_asset(
    node_id: u64,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    image: UiVisualAssetRef,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Image,
        frame: UiFrame::new(x, y, width, height),
        clip_frame: None,
        z_index: 2,
        style: runtime_style(),
        text_layout: None,
        text: None,
        image: Some(image),
        opacity: 1.0,
    }
}

pub(super) fn runtime_style() -> UiResolvedStyle {
    UiResolvedStyle::default()
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
