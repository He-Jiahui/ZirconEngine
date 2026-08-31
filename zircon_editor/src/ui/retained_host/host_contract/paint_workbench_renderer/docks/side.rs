mod frames;

use super::super::super::data::{
    HostPaneInteractionStateData, HostSideDockSurfaceData, HostTextInputFocusData,
    HostViewportImageSet,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{is_visible_frame, translated};
use super::super::super::paint_primitives::{draw_border, draw_rect};
use super::super::super::paint_template_nodes::draw_template_nodes;
use super::{palette::current_dock_chrome_palette, pane, panel_header, rail};
use frames::side_dock_frames;

pub(in crate::ui::retained_host::host_contract) fn draw_side_dock(
    frame: &mut HostRgbaFrame,
    dock: &HostSideDockSurfaceData,
    interaction: &HostPaneInteractionStateData,
    viewport_images: &HostViewportImageSet,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    if !is_visible_frame(&dock.region_frame) {
        return;
    }
    let palette = current_dock_chrome_palette();
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_side_dock_shell");
        draw_rect(frame, dock.region_frame.clone(), palette.shell);
        draw_border(frame, dock.region_frame.clone(), palette.separator);
    }

    let frames = side_dock_frames(dock);

    if is_visible_frame(&frames.rail_origin) {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_side_dock_rail");
        draw_rect(frame, frames.rail_origin.clone(), palette.header);
        draw_template_nodes(
            frame,
            &dock.rail_nodes,
            &frames.rail_origin,
            &frames.rail_origin,
            None,
        );
        rail::draw_active_rail_marker(frame, dock, &frames.rail_origin);
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_side_dock_header");
        panel_header::draw_panel_header(
            frame,
            &dock.header_nodes,
            &frames.panel_origin,
            &dock.header_frame,
        );
    }

    let content = translated(
        &dock.content_frame,
        frames.panel_origin.x,
        frames.panel_origin.y,
    );
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_side_dock_pane");
        pane::draw_pane(
            frame,
            &dock.pane,
            &content,
            interaction,
            viewport_images,
            text_input_focus,
        );
    }
}
