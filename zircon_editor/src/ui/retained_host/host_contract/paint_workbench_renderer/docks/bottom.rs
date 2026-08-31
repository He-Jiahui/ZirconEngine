use super::super::super::data::{
    HostBottomDockSurfaceData, HostPaneInteractionStateData, HostTextInputFocusData,
    HostViewportImageSet,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{is_visible_frame, translated};
use super::super::super::paint_primitives::{draw_border, draw_rect};
use super::{palette::current_dock_chrome_palette, pane, panel_header};

pub(in crate::ui::retained_host::host_contract) fn draw_bottom_dock(
    frame: &mut HostRgbaFrame,
    dock: &HostBottomDockSurfaceData,
    interaction: &HostPaneInteractionStateData,
    viewport_images: &HostViewportImageSet,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    if !is_visible_frame(&dock.region_frame) {
        return;
    }
    let palette = current_dock_chrome_palette();
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_bottom_dock_shell");
        draw_rect(frame, dock.region_frame.clone(), palette.shell);
        draw_border(frame, dock.region_frame.clone(), palette.separator);
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
            viewport_images,
            text_input_focus,
        );
    }
}
