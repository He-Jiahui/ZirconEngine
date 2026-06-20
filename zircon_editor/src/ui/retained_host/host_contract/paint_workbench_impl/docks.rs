mod floating_windows;
mod pane;
mod panel_header;
mod rail;
mod viewport_toolbar;

use super::super::data::{
    FrameRect, HostBottomDockSurfaceData, HostDocumentDockSurfaceData,
    HostPaneInteractionStateData, HostSideDockSurfaceData, HostTextInputFocusData,
    HostViewportImageData, HostWindowPresentationData,
};
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::{is_visible_frame, translated};
use super::super::paint_primitives::{draw_border, draw_rect};
use super::super::paint_template_nodes::draw_template_nodes;
use super::{DOCUMENT_PANEL, SEPARATOR, SIDE_PANEL, TOP_BAR};

pub(in crate::ui::retained_host::host_contract) fn draw_side_dock(
    frame: &mut HostRgbaFrame,
    dock: &HostSideDockSurfaceData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    if !is_visible_frame(&dock.region_frame) {
        return;
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_side_dock_shell");
        draw_rect(frame, dock.region_frame.clone(), SIDE_PANEL);
        draw_border(frame, dock.region_frame.clone(), SEPARATOR);
    }

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
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_side_dock_rail");
        draw_rect(frame, rail_origin.clone(), TOP_BAR);
        draw_template_nodes(frame, &dock.rail_nodes, &rail_origin, &rail_origin, None);
        rail::draw_active_rail_marker(frame, dock, &rail_origin);
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_side_dock_header");
        panel_header::draw_panel_header(
            frame,
            &dock.header_nodes,
            &panel_origin,
            &dock.header_frame,
        );
    }

    let content = translated(&dock.content_frame, panel_origin.x, panel_origin.y);
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_side_dock_pane");
        pane::draw_pane(
            frame,
            &dock.pane,
            &content,
            interaction,
            viewport_image,
            text_input_focus,
        );
    }
}

pub(in crate::ui::retained_host::host_contract) fn draw_document_dock(
    frame: &mut HostRgbaFrame,
    dock: &HostDocumentDockSurfaceData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    if !is_visible_frame(&dock.region_frame) {
        return;
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_document_dock_shell");
        draw_rect(frame, dock.region_frame.clone(), DOCUMENT_PANEL);
        draw_border(frame, dock.region_frame.clone(), SEPARATOR);
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_document_dock_header");
        panel_header::draw_panel_header(
            frame,
            &dock.header_nodes,
            &dock.region_frame,
            &dock.header_frame,
        );
    }
    let content = translated(
        &dock.content_frame,
        dock.region_frame.x,
        dock.region_frame.y,
    );
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_document_dock_pane");
        pane::draw_pane(
            frame,
            &dock.pane,
            &content,
            interaction,
            viewport_image,
            text_input_focus,
        );
    }
}

pub(in crate::ui::retained_host::host_contract) fn draw_bottom_dock(
    frame: &mut HostRgbaFrame,
    dock: &HostBottomDockSurfaceData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    if !is_visible_frame(&dock.region_frame) {
        return;
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_bottom_dock_shell");
        draw_rect(frame, dock.region_frame.clone(), SIDE_PANEL);
        draw_border(frame, dock.region_frame.clone(), SEPARATOR);
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_bottom_dock_header");
        panel_header::draw_panel_header(
            frame,
            &dock.header_nodes,
            &dock.region_frame,
            &dock.header_frame,
        );
    }
    let content = translated(
        &dock.content_frame,
        dock.region_frame.x,
        dock.region_frame.y,
    );
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_bottom_dock_pane");
        pane::draw_pane(
            frame,
            &dock.pane,
            &content,
            interaction,
            viewport_image,
            text_input_focus,
        );
    }
}

pub(in crate::ui::retained_host::host_contract) fn draw_floating_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    floating_windows::draw_floating_layer(
        frame,
        presentation,
        interaction,
        viewport_image,
        text_input_focus,
    );
}
