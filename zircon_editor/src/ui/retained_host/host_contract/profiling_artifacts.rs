use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::ui::{layout::UiPoint, surface::UiSurfaceFrame};

use super::data::{
    FloatingWindowData, FrameRect, HostBottomDockSurfaceData, HostChromeTabData,
    HostDocumentDockSurfaceData, HostSideDockSurfaceData, HostWindowPresentationData, PaneData,
    TemplatePaneNodeData,
};
use super::painter::paint_host_frame;
use super::presenter::HostPresenterBackend;
use super::profiling_hit_routes::route_contains_profile_frame;
use crate::ui::retained_host::primitives::{ModelRc, PhysicalSize};

const GEOMETRY_FILE: &str = "ui_profile_geometry.json";
const REFERENCE_SCREENSHOT_FILE: &str = "screenshot_reference.png";

pub(super) fn export_present_artifacts(
    presentation: &HostWindowPresentationData,
    size: &PhysicalSize,
    backend: HostPresenterBackend,
) {
    if !profile_capture_enabled() {
        return;
    }
    let Some(export_dir) = profile_export_dir() else {
        return;
    };
    if fs::create_dir_all(&export_dir).is_err() || is_forced_softbuffer_screenshot_run() {
        return;
    }

    let geometry = UiProfileGeometry::from_presentation(presentation, size, backend);
    if let Ok(bytes) = serde_json::to_vec_pretty(&geometry) {
        let _ = fs::write(export_dir.join(GEOMETRY_FILE), bytes);
    }

    if profile_screenshot_capture_enabled() {
        let frame = paint_host_frame(size.width, size.height, presentation);
        let _ = image::save_buffer_with_format(
            export_dir.join(REFERENCE_SCREENSHOT_FILE),
            frame.as_bytes(),
            frame.width(),
            frame.height(),
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        );
    }
}

#[derive(Serialize)]
struct UiProfileGeometry {
    schema_version: u32,
    presenter_backend: &'static str,
    window_client_size: UiProfileSize,
    layout: UiProfileLayout,
    resize_splitters: Vec<UiProfileNamedFrame>,
    document_tabs: Vec<UiProfileTabFrame>,
    drawer_tabs: Vec<UiProfileTabFrame>,
    host_page_tabs: Vec<UiProfileTabFrame>,
    activity_rail_buttons: Vec<UiProfileNamedFrame>,
    viewport_frame: Option<UiProfileFrame>,
    viewport_toolbar_controls: Vec<UiProfileNamedFrame>,
    template_controls: Vec<UiProfileNamedFrame>,
    clickable_frames: Vec<UiProfileNamedFrame>,
    hit_samples: Vec<UiProfileHitSample>,
}

impl UiProfileGeometry {
    fn from_presentation(
        presentation: &HostWindowPresentationData,
        size: &PhysicalSize,
        backend: HostPresenterBackend,
    ) -> Self {
        let scene = &presentation.host_scene_data;
        let mut resize_splitters = Vec::new();
        push_named_frame(
            &mut resize_splitters,
            "resize.left_splitter",
            "resize_splitter",
            "left",
            scene.resize_layer.left_splitter_frame.clone(),
            None,
        );
        push_named_frame(
            &mut resize_splitters,
            "resize.right_splitter",
            "resize_splitter",
            "right",
            scene.resize_layer.right_splitter_frame.clone(),
            None,
        );
        push_named_frame(
            &mut resize_splitters,
            "resize.bottom_splitter",
            "resize_splitter",
            "bottom",
            scene.resize_layer.bottom_splitter_frame.clone(),
            None,
        );

        let document_tabs = collect_document_tabs(&scene.document_dock);
        let mut drawer_tabs = Vec::new();
        collect_side_dock_tabs("left", &scene.left_dock, &mut drawer_tabs);
        collect_side_dock_tabs("right", &scene.right_dock, &mut drawer_tabs);
        collect_bottom_dock_tabs("bottom", &scene.bottom_dock, &mut drawer_tabs);
        for row in 0..scene.floating_layer.floating_windows.row_count() {
            if let Some(window) = scene.floating_layer.floating_windows.row_data(row) {
                collect_floating_window_tabs(&window, &mut drawer_tabs);
            }
        }

        let host_page_tabs = collect_host_page_tabs(&scene.page_chrome.tab_frames);
        let mut activity_rail_buttons = Vec::new();
        collect_activity_rail_buttons("left", &scene.left_dock, &mut activity_rail_buttons);
        collect_activity_rail_buttons("right", &scene.right_dock, &mut activity_rail_buttons);

        let mut viewport_toolbar_controls = Vec::new();
        let mut template_controls = Vec::new();
        collect_pane_profile_frames(
            "document",
            &scene.document_dock.pane,
            &translated(
                &scene.document_dock.content_frame,
                scene.document_dock.region_frame.x,
                scene.document_dock.region_frame.y,
            ),
            &mut viewport_toolbar_controls,
            &mut template_controls,
        );
        collect_pane_profile_frames(
            "left",
            &scene.left_dock.pane,
            &side_dock_content_frame(&scene.left_dock),
            &mut viewport_toolbar_controls,
            &mut template_controls,
        );
        collect_pane_profile_frames(
            "right",
            &scene.right_dock.pane,
            &side_dock_content_frame(&scene.right_dock),
            &mut viewport_toolbar_controls,
            &mut template_controls,
        );
        collect_pane_profile_frames(
            "bottom",
            &scene.bottom_dock.pane,
            &translated(
                &scene.bottom_dock.content_frame,
                scene.bottom_dock.region_frame.x,
                scene.bottom_dock.region_frame.y,
            ),
            &mut viewport_toolbar_controls,
            &mut template_controls,
        );
        for row in 0..scene.floating_layer.floating_windows.row_count() {
            if let Some(window) = scene.floating_layer.floating_windows.row_data(row) {
                collect_pane_profile_frames(
                    window.window_id.as_str(),
                    &window.active_pane,
                    &floating_window_content_frame(&window.frame, &window.header_frame),
                    &mut viewport_toolbar_controls,
                    &mut template_controls,
                );
            }
        }

        let mut clickable_frames = Vec::new();
        clickable_frames.extend(resize_splitters.iter().cloned());
        clickable_frames.extend(document_tabs.iter().map(UiProfileNamedFrame::from_tab));
        clickable_frames.extend(drawer_tabs.iter().map(UiProfileNamedFrame::from_tab));
        clickable_frames.extend(host_page_tabs.iter().map(UiProfileNamedFrame::from_tab));
        clickable_frames.extend(activity_rail_buttons.iter().cloned());
        clickable_frames.extend(viewport_toolbar_controls.iter().cloned());
        clickable_frames.extend(template_controls.iter().cloned());

        let hit_samples = clickable_frames
            .iter()
            .flat_map(|frame| hit_samples_for_frame(frame, presentation))
            .collect();

        Self {
            schema_version: 1,
            presenter_backend: backend.label(),
            window_client_size: UiProfileSize {
                width: size.width,
                height: size.height,
            },
            layout: UiProfileLayout {
                center_band: scene.layout.center_band_frame.clone().into(),
                document_region: scene.layout.document_region_frame.clone().into(),
                left_region: scene.layout.left_region_frame.clone().into(),
                right_region: scene.layout.right_region_frame.clone().into(),
                bottom_region: scene.layout.bottom_region_frame.clone().into(),
                status_bar: scene.layout.status_bar_frame.clone().into(),
            },
            resize_splitters,
            document_tabs,
            drawer_tabs,
            host_page_tabs,
            activity_rail_buttons,
            viewport_frame: visible_profile_frame(&presentation.host_layout.viewport_content_frame),
            viewport_toolbar_controls,
            template_controls,
            clickable_frames,
            hit_samples,
        }
    }
}

#[derive(Clone, Serialize)]
struct UiProfileNamedFrame {
    id: String,
    kind: String,
    surface: String,
    frame: UiProfileFrame,
    #[serde(skip_serializing_if = "Option::is_none")]
    clip: Option<UiProfileFrame>,
}

impl UiProfileNamedFrame {
    fn from_tab(tab: &UiProfileTabFrame) -> Self {
        Self {
            id: tab.id.clone(),
            kind: tab.kind.clone(),
            surface: tab.surface.clone(),
            frame: tab.frame.clone(),
            clip: None,
        }
    }
}

#[derive(Clone, Serialize)]
struct UiProfileTabFrame {
    id: String,
    title: String,
    kind: String,
    surface: String,
    frame: UiProfileFrame,
    close_frame: UiProfileFrame,
    active: bool,
}

#[derive(Clone, Serialize)]
struct UiProfileHitSample {
    id: String,
    kind: String,
    surface: String,
    sample: String,
    point: UiProfilePoint,
    expected_hit: bool,
    route_hit: bool,
}

#[derive(Clone, Serialize)]
struct UiProfileFrame {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl From<FrameRect> for UiProfileFrame {
    fn from(frame: FrameRect) -> Self {
        Self {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}

impl From<&FrameRect> for UiProfileFrame {
    fn from(frame: &FrameRect) -> Self {
        Self {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}

#[derive(Clone, Serialize)]
struct UiProfilePoint {
    x: f32,
    y: f32,
}

#[derive(Serialize)]
struct UiProfileSize {
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct UiProfileLayout {
    center_band: UiProfileFrame,
    document_region: UiProfileFrame,
    left_region: UiProfileFrame,
    right_region: UiProfileFrame,
    bottom_region: UiProfileFrame,
    status_bar: UiProfileFrame,
}

fn collect_document_tabs(dock: &HostDocumentDockSurfaceData) -> Vec<UiProfileTabFrame> {
    let header = translated(&dock.header_frame, dock.region_frame.x, dock.region_frame.y);
    collect_tabs(
        "document_tab",
        dock.surface_key.as_str(),
        &dock.tab_frames,
        &header,
    )
}

fn collect_side_dock_tabs(
    surface: &str,
    dock: &HostSideDockSurfaceData,
    out: &mut Vec<UiProfileTabFrame>,
) {
    let header = translated(&dock.header_frame, dock.region_frame.x, dock.region_frame.y);
    out.extend(collect_tabs(
        "drawer_tab",
        surface,
        &dock.tab_frames,
        &header,
    ));
}

fn collect_bottom_dock_tabs(
    surface: &str,
    dock: &HostBottomDockSurfaceData,
    out: &mut Vec<UiProfileTabFrame>,
) {
    let header = translated(&dock.header_frame, dock.region_frame.x, dock.region_frame.y);
    out.extend(collect_tabs(
        "drawer_tab",
        surface,
        &dock.tab_frames,
        &header,
    ));
}

fn collect_floating_window_tabs(window: &FloatingWindowData, out: &mut Vec<UiProfileTabFrame>) {
    let header = translated(&window.header_frame, window.frame.x, window.frame.y);
    out.extend(collect_tabs(
        "floating_tab",
        window.window_id.as_str(),
        &window.tab_frames,
        &header,
    ));
}

fn collect_host_page_tabs(tabs: &ModelRc<HostChromeTabData>) -> Vec<UiProfileTabFrame> {
    collect_tabs("host_page_tab", "host_page", tabs, &FrameRect::default())
}

fn collect_tabs(
    kind: &str,
    surface: &str,
    tabs: &ModelRc<HostChromeTabData>,
    origin: &FrameRect,
) -> Vec<UiProfileTabFrame> {
    let mut out = Vec::new();
    for row in 0..tabs.row_count() {
        let Some(tab) = tabs.row_data(row) else {
            continue;
        };
        let frame = translated(&tab.frame, origin.x, origin.y);
        if !is_visible_frame(&frame) {
            continue;
        }
        out.push(UiProfileTabFrame {
            id: tab.control_id.to_string(),
            title: tab.tab.title.to_string(),
            kind: kind.to_string(),
            surface: surface.to_string(),
            frame: frame.into(),
            close_frame: translated(&tab.close_frame, origin.x, origin.y).into(),
            active: tab.tab.active,
        });
    }
    out
}

fn collect_activity_rail_buttons(
    surface: &str,
    dock: &HostSideDockSurfaceData,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    if dock.rail_width_px <= 0.0 || !is_visible_frame(&dock.region_frame) {
        return;
    }
    let rail_x = if dock.rail_before_panel {
        dock.region_frame.x
    } else {
        dock.region_frame.x + (dock.region_frame.width - dock.rail_width_px).max(0.0)
    };
    let rail = FrameRect {
        x: rail_x,
        y: dock.region_frame.y,
        width: dock.rail_width_px.min(dock.region_frame.width.max(0.0)),
        height: dock.region_frame.height,
    };
    for row in 0..dock.rail_button_frames.row_count() {
        let Some(button) = dock.rail_button_frames.row_data(row) else {
            continue;
        };
        let frame = translated(&button.frame, rail.x, rail.y);
        push_named_frame(
            out,
            format!("activity_rail.{surface}.{}", button.control_id).as_str(),
            "activity_rail_button",
            surface,
            frame,
            None,
        );
    }
}

fn collect_pane_profile_frames(
    surface: &str,
    pane: &PaneData,
    content: &FrameRect,
    viewport_toolbar_controls: &mut Vec<UiProfileNamedFrame>,
    template_controls: &mut Vec<UiProfileNamedFrame>,
) {
    if !is_visible_frame(content) {
        return;
    }
    let mut body = content.clone();
    if matches!(pane.kind.as_str(), "Scene" | "Game") && pane.show_toolbar {
        let toolbar_height = 28.0_f32.min(content.height);
        let toolbar = FrameRect {
            x: content.x,
            y: content.y,
            width: content.width,
            height: toolbar_height,
        };
        collect_surface_frame_controls(
            "viewport_toolbar_control",
            surface,
            &toolbar,
            pane.viewport.toolbar_surface_frame.as_ref(),
            viewport_toolbar_controls,
        );
        body.y += toolbar_height;
        body.height = (body.height - toolbar_height).max(0.0);
    }
    collect_template_node_controls(surface, pane, &body, template_controls);
}

fn collect_template_node_controls(
    surface: &str,
    pane: &PaneData,
    body: &FrameRect,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    let Some(nodes) = pane_template_nodes(pane) else {
        return;
    };
    for row in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        if !is_dispatchable_template_node(&node) {
            continue;
        }
        let frame = translated_template_frame(&node.frame, body.x, body.y);
        let clip = node
            .has_clip_frame
            .then(|| translated_template_frame(&node.clip_frame, body.x, body.y).into());
        let effective_frame = if let Some(clip_frame) = clip.as_ref() {
            let Some(frame) = intersect_profile_frame(&frame, clip_frame) else {
                continue;
            };
            frame
        } else {
            frame.clone().into()
        };
        if !is_visible_profile_frame(&effective_frame) {
            continue;
        }
        push_named_profile_frame(
            out,
            format!("template.{surface}.{}", node.control_id).as_str(),
            "template_control",
            surface,
            effective_frame,
            clip,
        );
    }
}

fn collect_surface_frame_controls(
    kind: &str,
    surface: &str,
    origin: &FrameRect,
    surface_frame: Option<&UiSurfaceFrame>,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    let Some(surface_frame) = surface_frame else {
        return;
    };
    for node in &surface_frame.arranged_tree.nodes {
        if !node.supports_pointer() {
            continue;
        }
        let Some(control_id) = node.control_id.as_deref() else {
            continue;
        };
        let frame = FrameRect {
            x: origin.x + node.frame.x,
            y: origin.y + node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        };
        let clip = FrameRect {
            x: origin.x + node.clip_frame.x,
            y: origin.y + node.clip_frame.y,
            width: node.clip_frame.width,
            height: node.clip_frame.height,
        };
        let Some(effective_frame) = intersect_frames(&frame, &clip) else {
            continue;
        };
        let center = effective_frame.center_point();
        let local_center = UiPoint::new(center.x - origin.x, center.y - origin.y);
        let route_is_top_hit = hit_test_surface_frame(surface_frame, local_center)
            .top_hit
            .and_then(|node_id| surface_frame.arranged_tree.get(node_id))
            .and_then(|hit_node| hit_node.control_id.as_deref())
            .is_some_and(|hit_control_id| hit_control_id == control_id);
        if !route_is_top_hit {
            continue;
        }
        push_named_profile_frame(
            out,
            format!("{kind}.{surface}.{control_id}").as_str(),
            kind,
            surface,
            effective_frame.into(),
            Some(clip.into()),
        );
    }
}

fn pane_template_nodes(pane: &PaneData) -> Option<&ModelRc<TemplatePaneNodeData>> {
    match pane.kind.as_str() {
        "Hierarchy" => Some(&pane.hierarchy.nodes),
        "Inspector" => Some(&pane.inspector.nodes),
        "Console" => Some(&pane.console.nodes),
        "Assets" => Some(&pane.assets_activity.nodes),
        "AssetBrowser" => Some(&pane.asset_browser.nodes),
        "Welcome" => Some(&pane.welcome.nodes),
        "Project" | "UiComponentShowcase" => Some(&pane.project_overview.nodes),
        "RuntimeDiagnostics" => Some(&pane.runtime_diagnostics.nodes),
        "PerformanceTimeline" => Some(&pane.performance_timeline.nodes),
        "ModulePlugins" => Some(&pane.module_plugins.nodes),
        "BuildExport" => Some(&pane.build_export.nodes),
        "UiAssetEditor" => Some(&pane.ui_asset.nodes),
        "AnimationSequenceEditor" | "AnimationGraphEditor" => Some(&pane.animation.nodes),
        _ => None,
    }
}

fn is_dispatchable_template_node(node: &TemplatePaneNodeData) -> bool {
    !node.disabled
        && !node.control_id.is_empty()
        && (!node.action_id.is_empty()
            || !node.binding_id.is_empty()
            || !node.dispatch_kind.is_empty()
            || !node.edit_action_id.is_empty()
            || !node.commit_action_id.is_empty()
            || matches!(node.component_role.as_str(), "input-field" | "number-field"))
}

fn hit_samples_for_frame(
    frame: &UiProfileNamedFrame,
    presentation: &HostWindowPresentationData,
) -> Vec<UiProfileHitSample> {
    let mut samples = Vec::new();
    let center = frame.frame.center();
    samples.push(UiProfileHitSample {
        id: frame.id.clone(),
        kind: frame.kind.clone(),
        surface: frame.surface.clone(),
        sample: "center".to_string(),
        expected_hit: true,
        route_hit: profile_route_hit(presentation, frame, &center),
        point: center,
    });
    let outside_left = UiProfilePoint {
        x: frame.frame.x - 3.0,
        y: frame.frame.y + frame.frame.height * 0.5,
    };
    samples.push(UiProfileHitSample {
        id: frame.id.clone(),
        kind: frame.kind.clone(),
        surface: frame.surface.clone(),
        sample: "outside_left".to_string(),
        expected_hit: false,
        route_hit: profile_route_hit(presentation, frame, &outside_left),
        point: outside_left,
    });
    let outside_bottom = UiProfilePoint {
        x: frame.frame.x + frame.frame.width * 0.5,
        y: frame.frame.y + frame.frame.height + 3.0,
    };
    samples.push(UiProfileHitSample {
        id: frame.id.clone(),
        kind: frame.kind.clone(),
        surface: frame.surface.clone(),
        sample: "outside_bottom".to_string(),
        expected_hit: false,
        route_hit: profile_route_hit(presentation, frame, &outside_bottom),
        point: outside_bottom,
    });
    samples
}

fn profile_route_hit(
    presentation: &HostWindowPresentationData,
    frame: &UiProfileNamedFrame,
    point: &UiProfilePoint,
) -> bool {
    route_contains_profile_frame(
        presentation,
        frame.kind.as_str(),
        frame.id.as_str(),
        frame.surface.as_str(),
        point.x,
        point.y,
    )
}

impl UiProfileFrame {
    fn center(&self) -> UiProfilePoint {
        UiProfilePoint {
            x: self.x + self.width * 0.5,
            y: self.y + self.height * 0.5,
        }
    }
}

impl FrameRect {
    fn center_point(&self) -> UiProfilePoint {
        UiProfilePoint {
            x: self.x + self.width * 0.5,
            y: self.y + self.height * 0.5,
        }
    }
}

fn push_named_frame(
    out: &mut Vec<UiProfileNamedFrame>,
    id: &str,
    kind: &str,
    surface: &str,
    frame: FrameRect,
    clip: Option<FrameRect>,
) {
    if !is_visible_frame(&frame) {
        return;
    }
    push_named_profile_frame(out, id, kind, surface, frame.into(), clip.map(Into::into));
}

fn push_named_profile_frame(
    out: &mut Vec<UiProfileNamedFrame>,
    id: &str,
    kind: &str,
    surface: &str,
    frame: UiProfileFrame,
    clip: Option<UiProfileFrame>,
) {
    if !is_visible_profile_frame(&frame) {
        return;
    }
    out.push(UiProfileNamedFrame {
        id: id.to_string(),
        kind: kind.to_string(),
        surface: surface.to_string(),
        frame,
        clip,
    });
}

fn visible_profile_frame(frame: &FrameRect) -> Option<UiProfileFrame> {
    is_visible_frame(frame).then(|| frame.into())
}

fn is_visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

fn is_visible_profile_frame(frame: &UiProfileFrame) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

fn intersect_profile_frame(left: &FrameRect, right: &UiProfileFrame) -> Option<UiProfileFrame> {
    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = (left.x + left.width).min(right.x + right.width);
    let y1 = (left.y + left.height).min(right.y + right.height);
    (x1 > x0 && y1 > y0).then(|| UiProfileFrame {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    })
}

fn intersect_frames(left: &FrameRect, right: &FrameRect) -> Option<FrameRect> {
    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = (left.x + left.width).min(right.x + right.width);
    let y1 = (left.y + left.height).min(right.y + right.height);
    (x1 > x0 && y1 > y0).then(|| FrameRect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    })
}

fn side_dock_content_frame(dock: &HostSideDockSurfaceData) -> FrameRect {
    let panel_x = if dock.rail_before_panel {
        dock.region_frame.x + dock.rail_width_px
    } else {
        dock.region_frame.x
    };
    translated(&dock.content_frame, panel_x, dock.region_frame.y)
}

fn floating_window_content_frame(frame: &FrameRect, header: &FrameRect) -> FrameRect {
    FrameRect {
        x: frame.x + 1.0,
        y: frame.y + header.height.max(0.0) + 1.0,
        width: (frame.width - 2.0).max(0.0),
        height: (frame.height - header.height.max(0.0) - 2.0).max(0.0),
    }
}

fn translated(frame: &FrameRect, origin_x: f32, origin_y: f32) -> FrameRect {
    FrameRect {
        x: frame.x + origin_x,
        y: frame.y + origin_y,
        width: frame.width,
        height: frame.height,
    }
}

fn translated_template_frame(
    frame: &super::data::TemplateNodeFrameData,
    origin_x: f32,
    origin_y: f32,
) -> FrameRect {
    FrameRect {
        x: frame.x + origin_x,
        y: frame.y + origin_y,
        width: frame.width,
        height: frame.height,
    }
}

fn profile_capture_enabled() -> bool {
    env_truthy("ZIRCON_PROFILE_CAPTURE")
}

fn profile_screenshot_capture_enabled() -> bool {
    env_truthy("ZIRCON_PROFILE_CAPTURE_SCREENSHOTS")
}

fn is_forced_softbuffer_screenshot_run() -> bool {
    env_truthy("ZIRCON_PROFILE_FORCE_SOFTBUFFER") && !profile_capture_enabled()
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.as_str(),
                "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
            )
        })
        .unwrap_or(false)
}

fn profile_export_dir() -> Option<PathBuf> {
    let output_root = std::env::var("ZIRCON_PROFILE_OUTPUT_ROOT").ok()?;
    let session_id = std::env::var("ZIRCON_PROFILE_SESSION").unwrap_or_else(|_| "local".into());
    Some(PathBuf::from(output_root).join(sanitize_session_id(&session_id)))
}

fn sanitize_session_id(session_id: &str) -> String {
    session_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::primitives::VecModel;
    use std::rc::Rc;
    use zircon_runtime::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
        layout::UiFrame,
        tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    };

    #[test]
    fn profile_geometry_exports_absolute_splitter_and_tab_frames() {
        let mut presentation = HostWindowPresentationData::default();
        presentation
            .host_scene_data
            .resize_layer
            .left_splitter_frame = FrameRect {
            x: 100.0,
            y: 50.0,
            width: 4.0,
            height: 500.0,
        };
        presentation.host_scene_data.document_dock.region_frame = FrameRect {
            x: 120.0,
            y: 80.0,
            width: 600.0,
            height: 400.0,
        };
        presentation.host_scene_data.document_dock.header_frame = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 32.0,
        };
        presentation.host_scene_data.document_dock.tab_frames =
            ModelRc::from(Rc::new(VecModel::from(vec![HostChromeTabData {
                control_id: "scene-tab".into(),
                frame: FrameRect {
                    x: 12.0,
                    y: 4.0,
                    width: 120.0,
                    height: 24.0,
                },
                ..HostChromeTabData::default()
            }])));

        let geometry = UiProfileGeometry::from_presentation(
            &presentation,
            &PhysicalSize::new(1280, 720),
            HostPresenterBackend::Gpu,
        );

        assert_eq!(geometry.presenter_backend, "gpu");
        assert_eq!(geometry.resize_splitters.len(), 1);
        assert_eq!(geometry.document_tabs.len(), 1);
        assert_eq!(geometry.document_tabs[0].frame.x, 132.0);
        assert_eq!(geometry.document_tabs[0].frame.y, 84.0);
        assert!(geometry
            .hit_samples
            .iter()
            .any(|sample| sample.id == "scene-tab" && sample.expected_hit));
    }

    #[test]
    fn profile_geometry_omits_template_controls_disjoint_from_clip() {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_scene_data.document_dock.region_frame = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 240.0,
        };
        presentation.host_scene_data.document_dock.content_frame = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 240.0,
        };
        presentation.host_scene_data.document_dock.pane.kind = "Project".into();
        presentation
            .host_scene_data
            .document_dock
            .pane
            .project_overview
            .nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
            control_id: "OffClipAction".into(),
            action_id: "Project/OffClipAction".into(),
            frame: super::super::data::TemplateNodeFrameData {
                x: 120.0,
                y: 120.0,
                width: 80.0,
                height: 24.0,
            },
            has_clip_frame: true,
            clip_frame: super::super::data::TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 24.0,
            },
            ..TemplatePaneNodeData::default()
        }])));

        let geometry = UiProfileGeometry::from_presentation(
            &presentation,
            &PhysicalSize::new(320, 240),
            HostPresenterBackend::Gpu,
        );

        assert!(geometry
            .template_controls
            .iter()
            .all(|frame| { frame.id != "template.document.OffClipAction" }));
        assert!(geometry
            .hit_samples
            .iter()
            .all(|sample| sample.id != "template.document.OffClipAction"));
    }

    #[test]
    fn profile_geometry_clips_viewport_toolbar_controls_to_surface_clip() {
        let surface_frame = viewport_toolbar_surface_frame_for_test(vec![(
            2,
            "partial",
            UiFrame::new(90.0, 0.0, 30.0, 20.0),
        )]);
        let mut controls = Vec::new();

        collect_surface_frame_controls(
            "viewport_toolbar_control",
            "document",
            &FrameRect {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 28.0,
            },
            Some(&surface_frame),
            &mut controls,
        );

        assert_eq!(controls.len(), 1);
        assert_eq!(controls[0].frame.x, 100.0);
        assert_eq!(controls[0].frame.width, 10.0);
    }

    #[test]
    fn profile_geometry_omits_viewport_toolbar_controls_not_top_hit_at_center() {
        let surface_frame = viewport_toolbar_surface_frame_for_test(vec![
            (2, "covered", UiFrame::new(0.0, 0.0, 80.0, 20.0)),
            (3, "top", UiFrame::new(0.0, 0.0, 80.0, 20.0)),
        ]);
        let mut controls = Vec::new();

        collect_surface_frame_controls(
            "viewport_toolbar_control",
            "document",
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 28.0,
            },
            Some(&surface_frame),
            &mut controls,
        );

        assert!(controls
            .iter()
            .all(|frame| frame.id != "viewport_toolbar_control.document.covered"));
        assert!(controls
            .iter()
            .any(|frame| frame.id == "viewport_toolbar_control.document.top"));
    }

    fn viewport_toolbar_surface_frame_for_test(nodes: Vec<(u64, &str, UiFrame)>) -> UiSurfaceFrame {
        let mut surface = UiSurface::new(UiTreeId::new("test.viewport_toolbar_profile"));
        let root_frame = UiFrame::new(0.0, 0.0, 100.0, 28.0);
        let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(root_frame)
            .with_clip_to_bounds(true)
            .with_input_policy(UiInputPolicy::Ignore);
        root.layout_cache.clip_frame = Some(root_frame);
        surface.tree.insert_root(root);

        for (node_id, control_id, frame) in nodes {
            let node = UiTreeNode::new(
                UiNodeId::new(node_id),
                UiNodePath::new(format!("root/{control_id}")),
            )
            .with_frame(frame)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: true,
                pressed: false,
                checked: false,
                dirty: false,
            })
            .with_input_policy(UiInputPolicy::Receive)
            .with_template_metadata(UiTemplateNodeMetadata {
                control_id: Some(control_id.to_string()),
                ..Default::default()
            });
            surface.tree.insert_child(UiNodeId::new(1), node).unwrap();
        }
        surface.rebuild();
        surface.surface_frame()
    }
}
