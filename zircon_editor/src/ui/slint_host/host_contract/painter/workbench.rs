use slint::{Model, ModelRc};

use super::super::data::{
    FloatingWindowData, FrameRect, HostBottomDockSurfaceData, HostDocumentDockSurfaceData,
    HostSideDockSurfaceData, HostWindowPresentationData, PaneData, TemplatePaneNodeData,
};
use super::frame::HostRgbaFrame;
use super::geometry::{frame_or, is_visible_frame, translated};
use super::primitives::{
    draw_border, draw_border_clipped, draw_label_marker, draw_rect, draw_rect_clipped,
    draw_separator_line, draw_text_bars, draw_text_bars_clipped,
};
use super::template_nodes::{draw_template_nodes, has_template_nodes};

const SHELL_BACKGROUND: [u8; 4] = [18, 20, 25, 255];
const TOP_BAR: [u8; 4] = [31, 35, 44, 255];
const CENTER_BAND: [u8; 4] = [23, 27, 34, 255];
const SIDE_PANEL: [u8; 4] = [27, 32, 40, 255];
const DOCUMENT_PANEL: [u8; 4] = [13, 16, 22, 255];
const VIEWPORT_PANEL: [u8; 4] = [7, 10, 15, 255];
const TOOLBAR: [u8; 4] = [30, 36, 45, 255];
const STATUS_BAR: [u8; 4] = [36, 44, 56, 255];
const FLOATING_SHADOW: [u8; 4] = [4, 6, 10, 180];
const FLOATING_PANEL: [u8; 4] = [29, 35, 44, 255];
const PANE_EMPTY: [u8; 4] = [24, 29, 37, 255];
const SEPARATOR: [u8; 4] = [55, 64, 78, 255];
const ACCENT: [u8; 4] = [92, 156, 255, 255];
const MUTED_TEXT: [u8; 4] = [137, 151, 170, 255];

pub(in crate::ui::slint_host::host_contract) fn paint_host_frame(
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
) -> HostRgbaFrame {
    if width == 0 || height == 0 {
        return HostRgbaFrame::empty(width, height);
    }

    let mut frame = HostRgbaFrame::filled(width, height, SHELL_BACKGROUND);
    let root = resolve_root_frames(width, height, presentation);
    draw_root_skeleton(&mut frame, &root, presentation);
    draw_host_scene(&mut frame, &root, presentation);
    frame
}

fn draw_root_skeleton(
    frame: &mut HostRgbaFrame,
    root: &RootFrames,
    presentation: &HostWindowPresentationData,
) {
    draw_rect(frame, root.top_bar.clone(), TOP_BAR);
    draw_rect(frame, root.center_band.clone(), CENTER_BAND);
    draw_rect(frame, root.left_region.clone(), SIDE_PANEL);
    draw_rect(frame, root.right_region.clone(), SIDE_PANEL);
    draw_rect(frame, root.document_region.clone(), DOCUMENT_PANEL);
    draw_rect(frame, root.bottom_region.clone(), SIDE_PANEL);
    draw_rect(frame, root.viewport_region.clone(), VIEWPORT_PANEL);
    draw_rect(frame, root.status_bar.clone(), STATUS_BAR);

    draw_border(frame, root.left_region.clone(), SEPARATOR);
    draw_border(frame, root.right_region.clone(), SEPARATOR);
    draw_border(frame, root.document_region.clone(), SEPARATOR);
    draw_border(frame, root.bottom_region.clone(), SEPARATOR);
    draw_border(frame, root.viewport_region.clone(), ACCENT);
    draw_separator_line(
        frame,
        0,
        root.top_bar.height.round() as u32,
        frame.width(),
        SEPARATOR,
    );

    draw_project_marker(
        frame,
        &presentation.host_shell.project_path,
        root.top_bar.height,
    );
    draw_label_marker(
        frame,
        &root.viewport_region,
        &presentation.host_shell.viewport_label,
        ACCENT,
    );
    draw_label_marker(
        frame,
        &root.status_bar,
        &presentation.host_shell.status_secondary,
        MUTED_TEXT,
    );
}

fn draw_host_scene(
    frame: &mut HostRgbaFrame,
    root: &RootFrames,
    presentation: &HostWindowPresentationData,
) {
    let scene = &presentation.host_scene_data;
    draw_template_nodes(
        frame,
        &scene.menu_chrome.template_nodes,
        &zero_origin(),
        &root.top_bar,
    );
    draw_template_nodes(
        frame,
        &scene.page_chrome.template_nodes,
        &zero_origin(),
        &root.top_bar,
    );

    draw_side_dock(frame, &scene.left_dock);
    draw_document_dock(frame, &scene.document_dock);
    draw_side_dock(frame, &scene.right_dock);
    draw_bottom_dock(frame, &scene.bottom_dock);
    draw_resize_layer(frame, presentation);
    draw_floating_layer(frame, presentation);

    draw_template_nodes(
        frame,
        &scene.status_bar.template_nodes,
        &scene.status_bar.status_bar_frame,
        &root.status_bar,
    );
}

fn draw_side_dock(frame: &mut HostRgbaFrame, dock: &HostSideDockSurfaceData) {
    if !is_visible_frame(&dock.region_frame) {
        return;
    }
    draw_rect(frame, dock.region_frame.clone(), SIDE_PANEL);
    draw_border(frame, dock.region_frame.clone(), SEPARATOR);

    let rail_origin = if dock.rail_before_panel {
        FrameRect {
            x: dock.region_frame.x,
            y: dock.region_frame.y,
            width: dock.rail_width_px,
            height: dock.region_frame.height,
        }
    } else {
        FrameRect {
            x: dock.region_frame.x + dock.panel_width_px,
            y: dock.region_frame.y,
            width: dock.rail_width_px,
            height: dock.region_frame.height,
        }
    };
    let panel_origin = if dock.rail_before_panel {
        FrameRect {
            x: dock.region_frame.x + dock.rail_width_px,
            y: dock.region_frame.y,
            width: dock.panel_width_px,
            height: dock.region_frame.height,
        }
    } else {
        FrameRect {
            x: dock.region_frame.x,
            y: dock.region_frame.y,
            width: dock.panel_width_px,
            height: dock.region_frame.height,
        }
    };

    if is_visible_frame(&rail_origin) {
        draw_rect(frame, rail_origin.clone(), TOP_BAR);
        draw_template_nodes(frame, &dock.rail_nodes, &rail_origin, &rail_origin);
        draw_active_rail_marker(frame, dock, &rail_origin);
    }
    draw_panel_header(frame, &dock.header_nodes, &panel_origin, &dock.header_frame);

    let content = translated(&dock.content_frame, panel_origin.x, panel_origin.y);
    draw_pane(frame, &dock.pane, &content);
}

fn draw_document_dock(frame: &mut HostRgbaFrame, dock: &HostDocumentDockSurfaceData) {
    if !is_visible_frame(&dock.region_frame) {
        return;
    }
    draw_rect(frame, dock.region_frame.clone(), DOCUMENT_PANEL);
    draw_border(frame, dock.region_frame.clone(), SEPARATOR);
    draw_panel_header(
        frame,
        &dock.header_nodes,
        &dock.region_frame,
        &dock.header_frame,
    );
    let content = translated(
        &dock.content_frame,
        dock.region_frame.x,
        dock.region_frame.y,
    );
    draw_pane(frame, &dock.pane, &content);
}

fn draw_bottom_dock(frame: &mut HostRgbaFrame, dock: &HostBottomDockSurfaceData) {
    if !is_visible_frame(&dock.region_frame) {
        return;
    }
    draw_rect(frame, dock.region_frame.clone(), SIDE_PANEL);
    draw_border(frame, dock.region_frame.clone(), SEPARATOR);
    draw_panel_header(
        frame,
        &dock.header_nodes,
        &dock.region_frame,
        &dock.header_frame,
    );
    let content = translated(
        &dock.content_frame,
        dock.region_frame.x,
        dock.region_frame.y,
    );
    draw_pane(frame, &dock.pane, &content);
}

fn draw_panel_header(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    header_frame: &FrameRect,
) {
    let header = translated(header_frame, origin.x, origin.y);
    if !is_visible_frame(&header) {
        return;
    }
    draw_rect(frame, header.clone(), TOP_BAR);
    draw_template_nodes(frame, nodes, origin, &header);
    draw_separator_line(
        frame,
        header.x.max(0.0) as u32,
        (header.y + header.height - 1.0).max(0.0) as u32,
        header.width.max(0.0) as u32,
        SEPARATOR,
    );
}

fn draw_pane(frame: &mut HostRgbaFrame, pane: &PaneData, content: &FrameRect) {
    if !is_visible_frame(content) {
        return;
    }
    let pane_color = match pane.kind.as_str() {
        "Scene" | "Game" => VIEWPORT_PANEL,
        _ => PANE_EMPTY,
    };
    draw_rect(frame, content.clone(), pane_color);

    let body = if matches!(pane.kind.as_str(), "Scene" | "Game") && pane.show_toolbar {
        let toolbar = FrameRect {
            x: content.x,
            y: content.y,
            width: content.width,
            height: 28.0_f32.min(content.height),
        };
        draw_viewport_toolbar(frame, pane, &toolbar, content);
        FrameRect {
            x: content.x,
            y: content.y + toolbar.height,
            width: content.width,
            height: (content.height - toolbar.height).max(0.0),
        }
    } else {
        content.clone()
    };

    let painted_nodes = draw_pane_template_nodes(frame, pane, &body, content);
    if !painted_nodes {
        draw_pane_fallback(frame, pane, &body, content);
    }
}

fn draw_pane_template_nodes(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
) -> bool {
    match pane.kind.as_str() {
        "Hierarchy" => draw_if_present(frame, &pane.hierarchy.nodes, body, clip),
        "Inspector" => draw_if_present(frame, &pane.inspector.nodes, body, clip),
        "Console" => draw_if_present(frame, &pane.console.nodes, body, clip),
        "Assets" => draw_if_present(frame, &pane.assets_activity.nodes, body, clip),
        "AssetBrowser" => draw_if_present(frame, &pane.asset_browser.nodes, body, clip),
        "Project" | "UiComponentShowcase" => {
            draw_if_present(frame, &pane.project_overview.nodes, body, clip)
        }
        "ModulePlugins" | "RuntimeDiagnostics" => {
            draw_if_present(frame, &pane.module_plugins.nodes, body, clip)
        }
        "BuildExport" => draw_if_present(frame, &pane.build_export.nodes, body, clip),
        "UiAssetEditor" => draw_if_present(frame, &pane.ui_asset.nodes, body, clip),
        "AnimationSequenceEditor" | "AnimationGraphEditor" => {
            draw_if_present(frame, &pane.animation.nodes, body, clip)
        }
        _ => false,
    }
}

fn draw_if_present(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
) -> bool {
    has_template_nodes(nodes) && draw_template_nodes(frame, nodes, origin, clip)
}

fn draw_viewport_toolbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    toolbar: &FrameRect,
    clip: &FrameRect,
) {
    if !is_visible_frame(toolbar) {
        return;
    }
    draw_rect_clipped(frame, toolbar.clone(), Some(clip), TOOLBAR);
    draw_border_clipped(frame, toolbar.clone(), Some(clip), SEPARATOR);
    for (index, label) in [
        pane.viewport.tool.as_str(),
        pane.viewport.transform_space.as_str(),
        pane.viewport.display_mode.as_str(),
        pane.viewport.grid_mode.as_str(),
    ]
    .into_iter()
    .enumerate()
    {
        draw_text_bars_clipped(
            frame,
            toolbar.x + 10.0 + index as f32 * 62.0,
            toolbar.y + 12.0,
            label,
            Some(clip),
            MUTED_TEXT,
        );
    }
}

fn draw_pane_fallback(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
) {
    let label = first_non_empty(&[
        pane.title.as_str(),
        pane.kind.as_str(),
        pane.subtitle.as_str(),
        pane.info.as_str(),
    ]);
    draw_text_bars_clipped(
        frame,
        body.x + 12.0,
        body.y + 16.0,
        label,
        Some(clip),
        MUTED_TEXT,
    );
}

fn draw_active_rail_marker(
    frame: &mut HostRgbaFrame,
    dock: &HostSideDockSurfaceData,
    rail_origin: &FrameRect,
) {
    if dock.rail_active_control_id.is_empty() {
        return;
    }
    for row in 0..dock.rail_button_frames.row_count() {
        let Some(control) = dock.rail_button_frames.row_data(row) else {
            continue;
        };
        if control.control_id.as_str() == dock.rail_active_control_id.as_str()
            || dock
                .rail_active_control_id
                .as_str()
                .ends_with(control.control_id.as_str())
        {
            let marker = translated(&control.frame, rail_origin.x, rail_origin.y);
            draw_border_clipped(frame, marker, Some(rail_origin), ACCENT);
        }
    }
}

fn draw_resize_layer(frame: &mut HostRgbaFrame, presentation: &HostWindowPresentationData) {
    let resize = &presentation.host_scene_data.resize_layer;
    for splitter in [
        &resize.left_splitter_frame,
        &resize.right_splitter_frame,
        &resize.bottom_splitter_frame,
    ] {
        if is_visible_frame(splitter) {
            draw_rect(frame, splitter.clone(), [79, 92, 112, 255]);
        }
    }
}

fn draw_floating_layer(frame: &mut HostRgbaFrame, presentation: &HostWindowPresentationData) {
    let windows = &presentation.host_scene_data.floating_layer.floating_windows;
    for row in 0..windows.row_count() {
        let Some(window) = windows.row_data(row) else {
            continue;
        };
        draw_floating_window(frame, &window);
    }
}

fn draw_floating_window(frame: &mut HostRgbaFrame, window: &FloatingWindowData) {
    if !is_visible_frame(&window.frame) {
        return;
    }
    let shadow = FrameRect {
        x: window.frame.x + 4.0,
        y: window.frame.y + 5.0,
        width: window.frame.width,
        height: window.frame.height,
    };
    draw_rect(frame, shadow, FLOATING_SHADOW);
    draw_rect(frame, window.frame.clone(), FLOATING_PANEL);
    draw_border(frame, window.frame.clone(), ACCENT);

    let header = translated(&window.header_frame, window.frame.x, window.frame.y);
    if is_visible_frame(&header) {
        draw_rect(frame, header.clone(), TOP_BAR);
        draw_template_nodes(frame, &window.header_nodes, &window.frame, &header);
    }

    let body = FrameRect {
        x: window.frame.x + 1.0,
        y: header
            .y
            .max(window.frame.y)
            .saturating_add_f32(header.height.max(0.0) + 1.0),
        width: (window.frame.width - 2.0).max(0.0),
        height: (window.frame.height - header.height.max(0.0) - 2.0).max(0.0),
    };
    draw_pane(frame, &window.active_pane, &body);
}

fn draw_project_marker(frame: &mut HostRgbaFrame, project_path: &str, top_bar_height: f32) {
    draw_rect(
        frame,
        FrameRect {
            x: 12.0,
            y: (top_bar_height * 0.5 - 6.0).max(4.0),
            width: 18.0,
            height: 12.0,
        },
        ACCENT,
    );
    draw_text_bars(
        frame,
        40.0,
        (top_bar_height * 0.5 - 5.0).max(5.0),
        project_path,
        MUTED_TEXT,
    );
}

fn resolve_root_frames(
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
) -> RootFrames {
    let layout = &presentation.host_layout;
    let scene_layout = &presentation.host_scene_data.layout;
    let top_bar_height = if is_visible_frame(&scene_layout.center_band_frame) {
        scene_layout.center_band_frame.y
    } else if layout.center_band_frame.y.is_finite() && layout.center_band_frame.y > 1.0 {
        layout.center_band_frame.y
    } else {
        38.0_f32.min(height as f32 * 0.25)
    };
    let fallback_status_height = 24.0_f32.min(height as f32 * 0.2);
    let status_bar = frame_or(
        &scene_layout.status_bar_frame,
        frame_or(
            &layout.status_bar_frame,
            FrameRect {
                x: 0.0,
                y: (height as f32 - fallback_status_height).max(top_bar_height),
                width: width as f32,
                height: fallback_status_height,
            },
        ),
    );
    let center_band = frame_or(
        &scene_layout.center_band_frame,
        frame_or(
            &layout.center_band_frame,
            FrameRect {
                x: 0.0,
                y: top_bar_height,
                width: width as f32,
                height: (status_bar.y - top_bar_height).max(1.0),
            },
        ),
    );
    let left_region = frame_or(
        &scene_layout.left_region_frame,
        frame_or(
            &layout.left_region_frame,
            FrameRect {
                x: 0.0,
                y: center_band.y,
                width: (width as f32 * 0.22).min(260.0),
                height: center_band.height,
            },
        ),
    );
    let right_region = frame_or(
        &scene_layout.right_region_frame,
        frame_or(&layout.right_region_frame, FrameRect::default()),
    );
    let bottom_region = frame_or(
        &scene_layout.bottom_region_frame,
        frame_or(&layout.bottom_region_frame, FrameRect::default()),
    );
    let document_region = frame_or(
        &scene_layout.document_region_frame,
        frame_or(
            &layout.document_region_frame,
            FrameRect {
                x: left_region.x + left_region.width,
                y: center_band.y,
                width: (width as f32 - left_region.width).max(1.0),
                height: center_band.height,
            },
        ),
    );
    let viewport_region = frame_or(
        &scene_layout.viewport_content_frame,
        frame_or(
            &layout.viewport_content_frame,
            FrameRect {
                x: document_region.x + 16.0,
                y: document_region.y + 28.0,
                width: (document_region.width - 32.0).max(1.0),
                height: (document_region.height - 56.0).max(1.0),
            },
        ),
    );
    RootFrames {
        top_bar: FrameRect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: top_bar_height,
        },
        center_band,
        status_bar,
        left_region,
        right_region,
        bottom_region,
        document_region,
        viewport_region,
    }
}

fn zero_origin() -> FrameRect {
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    }
}

fn first_non_empty<'a>(values: &[&'a str]) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("")
}

trait SaturatingAddF32 {
    fn saturating_add_f32(self, value: f32) -> f32;
}

impl SaturatingAddF32 for f32 {
    fn saturating_add_f32(self, value: f32) -> f32 {
        let result = self + value;
        if result.is_finite() {
            result
        } else {
            self
        }
    }
}

struct RootFrames {
    top_bar: FrameRect,
    center_band: FrameRect,
    status_bar: FrameRect,
    left_region: FrameRect,
    right_region: FrameRect,
    bottom_region: FrameRect,
    document_region: FrameRect,
    viewport_region: FrameRect,
}
