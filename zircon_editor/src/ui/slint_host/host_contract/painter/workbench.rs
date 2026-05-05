use slint::{Model, ModelRc};

use super::super::data::{
    FloatingWindowData, FrameRect, HostBottomDockSurfaceData, HostDocumentDockSurfaceData,
    HostPaneInteractionStateData, HostSideDockSurfaceData, HostViewportImageData,
    HostWindowLayoutData, HostWindowPresentationData, PaneData, TemplatePaneNodeData,
};
use super::frame::HostRgbaFrame;
use super::diagnostics_overlay::debug_refresh_overlay_frame;
use super::geometry::{frame_from_template, frame_or, intersect, is_visible_frame, translated};
use super::primitives::{
    draw_border, draw_border_clipped, draw_label_marker, draw_rect, draw_rect_clipped,
    draw_rgba_image_clipped, draw_separator_line, draw_text_bars, draw_text_bars_clipped,
};
use super::template_nodes::{draw_template_nodes, has_template_nodes};
use crate::ui::slint_host::hierarchy_pointer::constants::{
    ROW_GAP, ROW_HEIGHT, ROW_WIDTH_INSET, ROW_X, ROW_Y,
};

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
const HIERARCHY_ROW: [u8; 4] = [30, 36, 45, 255];
const HIERARCHY_ROW_HOVERED: [u8; 4] = [39, 48, 62, 255];
const HIERARCHY_ROW_SELECTED: [u8; 4] = [54, 83, 130, 255];
const HIERARCHY_ROW_INDENT: f32 = 14.0;
const HIERARCHY_ROW_TEXT_X: f32 = 8.0;
const HIERARCHY_ROW_TEXT_Y: f32 = 4.0;
const ASSET_TREE_ROW_HOVERED: [u8; 4] = [54, 83, 130, 120];
const ACTIVITY_ASSET_TREE_ROW_CONTROL: &str = "AssetsActivityTreeRowPanel";
const BROWSER_ASSET_TREE_ROW_CONTROL: &str = "AssetBrowserSourcesRowPanel";

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

pub(in crate::ui::slint_host::host_contract) fn repaint_host_frame_region(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    damage: &FrameRect,
) -> Option<FrameRect> {
    if frame.width() == 0 || frame.height() == 0 {
        return None;
    }
    let frame_bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: frame.width() as f32,
        height: frame.height() as f32,
    };
    let damage = intersect(damage, &frame_bounds)?;

    // Slate-style fast path: the retained backbuffer is the authoritative previous
    // frame, and the active paint clip makes every painter operation respect damage.
    let previous_clip = frame.replace_paint_clip(Some(damage.clone()));
    frame.fill_rect(&damage, SHELL_BACKGROUND);
    let root = resolve_root_frames(frame.width(), frame.height(), presentation);
    draw_root_skeleton(frame, &root, presentation);
    draw_host_scene(frame, &root, presentation);
    frame.replace_paint_clip(previous_clip);
    Some(damage)
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
    draw_debug_refresh_rate_marker(
        frame,
        &root.top_bar,
        &presentation.host_shell.debug_refresh_rate,
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
    let viewport_image = presentation.viewport_image.as_ref();
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

    draw_side_dock(
        frame,
        &scene.left_dock,
        &presentation.pane_interaction_state,
        viewport_image,
    );
    draw_document_dock(
        frame,
        &scene.document_dock,
        &presentation.pane_interaction_state,
        viewport_image,
    );
    draw_side_dock(
        frame,
        &scene.right_dock,
        &presentation.pane_interaction_state,
        viewport_image,
    );
    draw_bottom_dock(
        frame,
        &scene.bottom_dock,
        &presentation.pane_interaction_state,
        viewport_image,
    );
    draw_resize_layer(frame, presentation);
    draw_floating_layer(
        frame,
        presentation,
        &presentation.pane_interaction_state,
        viewport_image,
    );
    draw_open_menu_popup(frame, presentation);

    draw_template_nodes(
        frame,
        &scene.status_bar.template_nodes,
        &scene.status_bar.status_bar_frame,
        &root.status_bar,
    );
}

fn draw_open_menu_popup(frame: &mut HostRgbaFrame, presentation: &HostWindowPresentationData) {
    let menu_index = presentation.menu_state.open_menu_index;
    if menu_index < 0 {
        return;
    }
    let menu_index = menu_index as usize;
    let scene = &presentation.host_scene_data;
    let Some(menu_frame) = scene.menu_chrome.menu_frames.row_data(menu_index) else {
        return;
    };
    let Some(menu) = scene.menu_chrome.menus.row_data(menu_index) else {
        return;
    };
    let popup = FrameRect {
        x: menu_frame.frame.x,
        y: menu_frame.frame.y + menu_frame.frame.height + 3.0,
        width: menu.popup_width_px.max(menu_frame.frame.width).max(1.0),
        height: menu
            .popup_height_px
            .max(presentation.menu_state.window_menu_popup_height_px)
            .max(1.0),
    };
    if !is_visible_frame(&popup) {
        return;
    }
    draw_rect(frame, popup.clone(), TOP_BAR);
    draw_border(frame, popup.clone(), ACCENT);
    draw_template_nodes(frame, &menu.popup_nodes, &popup, &popup);
}

fn draw_side_dock(
    frame: &mut HostRgbaFrame,
    dock: &HostSideDockSurfaceData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
) {
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
    draw_pane(frame, &dock.pane, &content, interaction, viewport_image);
}

fn draw_document_dock(
    frame: &mut HostRgbaFrame,
    dock: &HostDocumentDockSurfaceData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
) {
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
    draw_pane(frame, &dock.pane, &content, interaction, viewport_image);
}

fn draw_bottom_dock(
    frame: &mut HostRgbaFrame,
    dock: &HostBottomDockSurfaceData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
) {
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
    draw_pane(frame, &dock.pane, &content, interaction, viewport_image);
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

fn draw_pane(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    content: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
) {
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

    let painted_viewport = draw_viewport_image(frame, pane, &body, content, viewport_image);
    let painted_nodes = draw_pane_template_nodes(frame, pane, &body, content);
    let painted_native = draw_native_pane_content(frame, pane, &body, content, interaction);
    if !painted_viewport && !painted_nodes && !painted_native {
        draw_pane_fallback(frame, pane, &body, content);
    }
}

fn draw_viewport_image(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    viewport_image: Option<&HostViewportImageData>,
) -> bool {
    if !matches!(pane.kind.as_str(), "Scene" | "Game") {
        return false;
    }
    let Some(image) = viewport_image.filter(|image| image.is_valid()) else {
        return false;
    };
    draw_rgba_image_clipped(
        frame,
        body.clone(),
        Some(clip),
        image.width,
        image.height,
        &image.rgba,
    )
}

fn draw_native_pane_content(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    match pane.kind.as_str() {
        "Hierarchy" => draw_hierarchy_rows(frame, pane, body, clip, interaction),
        "Assets" => draw_asset_tree_hover_overlay(
            frame,
            &pane.assets_activity.nodes,
            body,
            clip,
            ACTIVITY_ASSET_TREE_ROW_CONTROL,
            interaction.activity_asset_tree_hovered_index,
            interaction.activity_asset_tree_scroll_px,
        ),
        "AssetBrowser" => draw_asset_tree_hover_overlay(
            frame,
            &pane.asset_browser.nodes,
            body,
            clip,
            BROWSER_ASSET_TREE_ROW_CONTROL,
            interaction.browser_asset_tree_hovered_index,
            interaction.browser_asset_tree_scroll_px,
        ),
        _ => false,
    }
}

fn draw_hierarchy_rows(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let node_count = pane.hierarchy.hierarchy_nodes.row_count();
    if node_count == 0 {
        return false;
    }
    let viewport = hierarchy_viewport_frame(pane, body);
    let Some(row_clip) = intersect(&viewport, clip) else {
        return false;
    };
    let row_width = (viewport.width - ROW_WIDTH_INSET).max(0.0);
    let scroll_px = interaction.hierarchy_scroll_px.max(0.0);

    for index in 0..node_count {
        let Some(node) = pane.hierarchy.hierarchy_nodes.row_data(index) else {
            continue;
        };
        let row = FrameRect {
            x: viewport.x + ROW_X,
            y: viewport.y + ROW_Y + index as f32 * (ROW_HEIGHT + ROW_GAP) - scroll_px,
            width: row_width,
            height: ROW_HEIGHT,
        };
        if intersect(&row, &row_clip).is_none() {
            continue;
        }
        let color = if interaction.hovered_hierarchy_index == index as i32 {
            HIERARCHY_ROW_HOVERED
        } else if node.selected {
            HIERARCHY_ROW_SELECTED
        } else {
            HIERARCHY_ROW
        };
        draw_rect_clipped(frame, row.clone(), Some(&row_clip), color);
        if node.selected {
            draw_border_clipped(frame, row.clone(), Some(&row_clip), ACCENT);
        }
        let indent = node.depth.max(0) as f32 * HIERARCHY_ROW_INDENT;
        draw_text_bars_clipped(
            frame,
            row.x + HIERARCHY_ROW_TEXT_X + indent.min(row.width * 0.5),
            row.y + HIERARCHY_ROW_TEXT_Y,
            &node.name,
            Some(&row_clip),
            MUTED_TEXT,
        );
    }
    true
}

fn hierarchy_viewport_frame(pane: &PaneData, body: &FrameRect) -> FrameRect {
    (0..pane.hierarchy.nodes.row_count())
        .filter_map(|row| pane.hierarchy.nodes.row_data(row))
        .find_map(|node| {
            matches!(
                node.control_id.as_str(),
                "HierarchyListPanel" | "HierarchyTreeSlotAnchor"
            )
            .then(|| translated(&frame_from_template(&node.frame), body.x, body.y))
            .filter(is_visible_frame)
        })
        .unwrap_or_else(|| body.clone())
}

fn draw_asset_tree_hover_overlay(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
    clip: &FrameRect,
    row_control_id: &str,
    hovered_index: i32,
    scroll_px: f32,
) -> bool {
    if hovered_index < 0 {
        return false;
    }
    let Some(row) = asset_tree_row_frame(
        nodes,
        body,
        row_control_id,
        hovered_index as usize,
        scroll_px.max(0.0),
    ) else {
        return false;
    };
    if intersect(&row, clip).is_none() {
        return false;
    }
    draw_rect_clipped(frame, row.clone(), Some(clip), ASSET_TREE_ROW_HOVERED);
    draw_border_clipped(frame, row, Some(clip), ACCENT);
    true
}

fn asset_tree_row_frame(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
    row_control_id: &str,
    hovered_index: usize,
    scroll_px: f32,
) -> Option<FrameRect> {
    let mut asset_row_index = 0;
    for row in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        if !matches_asset_tree_row(node.control_id.as_str(), row_control_id) {
            continue;
        }
        if asset_row_index == hovered_index {
            let mut frame = translated(&frame_from_template(&node.frame), body.x, body.y);
            frame.y -= scroll_px;
            return Some(frame);
        }
        asset_row_index += 1;
    }
    None
}

fn matches_asset_tree_row(control_id: &str, row_control_id: &str) -> bool {
    control_id
        .rsplit('/')
        .next()
        .is_some_and(|leaf| leaf == row_control_id)
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

fn draw_floating_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
) {
    let windows = &presentation.host_scene_data.floating_layer.floating_windows;
    for row in 0..windows.row_count() {
        let Some(window) = windows.row_data(row) else {
            continue;
        };
        draw_floating_window(frame, &window, interaction, viewport_image);
    }
}

fn draw_floating_window(
    frame: &mut HostRgbaFrame,
    window: &FloatingWindowData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
) {
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
    draw_pane(
        frame,
        &window.active_pane,
        &body,
        interaction,
        viewport_image,
    );
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

fn draw_debug_refresh_rate_marker(frame: &mut HostRgbaFrame, top_bar: &FrameRect, label: &str) {
    let Some(marker) = debug_refresh_overlay_frame(top_bar, label) else {
        return;
    };
    draw_rect_clipped(frame, marker.clone(), Some(top_bar), [18, 24, 34, 210]);
    draw_border_clipped(frame, marker.clone(), Some(top_bar), ACCENT);
    draw_text_bars_clipped(frame, marker.x + 7.0, marker.y + 5.0, label, Some(&marker), ACCENT);
}

fn resolve_root_frames(
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
) -> RootFrames {
    let scene_layout = &presentation.host_scene_data.layout;
    let layout = if has_visible_root_frame(scene_layout) {
        scene_layout
    } else {
        &presentation.host_layout
    };
    let top_bar_height =
        if layout.center_band_frame.y.is_finite() && layout.center_band_frame.y > 1.0 {
            layout.center_band_frame.y
        } else {
            38.0_f32.min(height as f32 * 0.25)
        };
    let fallback_status_height = 24.0_f32.min(height as f32 * 0.2);
    let status_bar = frame_or(
        &layout.status_bar_frame,
        FrameRect {
            x: 0.0,
            y: (height as f32 - fallback_status_height).max(top_bar_height),
            width: width as f32,
            height: fallback_status_height,
        },
    );
    let center_band = frame_or(
        &layout.center_band_frame,
        FrameRect {
            x: 0.0,
            y: top_bar_height,
            width: width as f32,
            height: (status_bar.y - top_bar_height).max(1.0),
        },
    );
    let left_region = frame_or(
        &layout.left_region_frame,
        FrameRect {
            x: 0.0,
            y: center_band.y,
            width: (width as f32 * 0.22).min(260.0),
            height: center_band.height,
        },
    );
    let right_region = frame_or(&layout.right_region_frame, FrameRect::default());
    let bottom_region = frame_or(&layout.bottom_region_frame, FrameRect::default());
    let document_region = frame_or(
        &layout.document_region_frame,
        FrameRect {
            x: left_region.x + left_region.width,
            y: center_band.y,
            width: (width as f32 - left_region.width).max(1.0),
            height: center_band.height,
        },
    );
    let viewport_region = frame_or(
        &layout.viewport_content_frame,
        FrameRect {
            x: document_region.x + 16.0,
            y: document_region.y + 28.0,
            width: (document_region.width - 32.0).max(1.0),
            height: (document_region.height - 56.0).max(1.0),
        },
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

fn has_visible_root_frame(layout: &HostWindowLayoutData) -> bool {
    is_visible_frame(&layout.center_band_frame)
        || is_visible_frame(&layout.status_bar_frame)
        || is_visible_frame(&layout.document_region_frame)
        || is_visible_frame(&layout.viewport_content_frame)
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
