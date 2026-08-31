use super::super::super::super::data::HostWindowPresentationData;
use super::super::super::super::paint_asset_deletion_blocker::draw_asset_deletion_blocker;
use super::super::super::super::paint_close_prompt::draw_close_prompt;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::menus;

pub(in super::super) fn draw_menu_and_prompt_layers(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_open_menu_popup");
        menus::draw_open_menu_popup(frame, presentation);
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_asset_deletion_blocker");
        draw_asset_deletion_blocker(frame, presentation);
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_close_prompt");
        draw_close_prompt(frame, presentation);
    }
}
