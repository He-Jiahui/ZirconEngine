use super::data::HostWindowPresentationData;
use super::paint_frame::HostRgbaFrame;
use super::paint_theme::PALETTE;

mod docks;
mod menus;
mod native_panes;
mod root_frames;
mod scene_layers;
mod skeleton;
mod welcome;

use root_frames::resolve_root_frames;
use scene_layers::draw_host_scene;
use skeleton::draw_root_skeleton;

pub(in crate::ui::retained_host::host_contract) use scene_layers::{
    draw_componentized_workbench_window, draws_componentized_workbench_window,
};

const TOP_BAR: [u8; 4] = PALETTE.popup;
const CENTER_BAND: [u8; 4] = [23, 27, 34, 255];
const SIDE_PANEL: [u8; 4] = PALETTE.surface_inset;
const DOCUMENT_PANEL: [u8; 4] = [13, 16, 22, 255];
const VIEWPORT_PANEL: [u8; 4] = [7, 10, 15, 255];
const TOOLBAR: [u8; 4] = PALETTE.surface;
const STATUS_BAR: [u8; 4] = PALETTE.surface_hover;
const FLOATING_SHADOW: [u8; 4] = [4, 6, 10, 180];
const FLOATING_PANEL: [u8; 4] = PALETTE.surface;
const PANE_EMPTY: [u8; 4] = PALETTE.surface_inset;
const SEPARATOR: [u8; 4] = PALETTE.border;
const ACCENT: [u8; 4] = PALETTE.focus_ring;
const MUTED_TEXT: [u8; 4] = PALETTE.text_muted;

pub(in crate::ui::retained_host::host_contract) fn draw_legacy_workbench_window(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let root = resolve_root_frames(frame.width(), frame.height(), presentation);
    draw_root_skeleton(frame, &root, presentation);
    draw_host_scene(frame, &root, presentation);
}

pub(in crate::ui::retained_host::host_contract) fn draw_legacy_workbench_window_profiled(
    frame: &mut HostRgbaFrame,
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
    _resolve_scope: &'static str,
    _skeleton_scope: &'static str,
    _scene_scope: &'static str,
) {
    let root = {
        zircon_runtime::profile_scope!("editor", "host_painter", _resolve_scope);
        resolve_root_frames(width, height, presentation)
    };
    {
        zircon_runtime::profile_scope!("editor", "host_painter", _skeleton_scope);
        draw_root_skeleton(frame, &root, presentation);
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", _scene_scope);
        draw_host_scene(frame, &root, presentation);
    }
}

fn first_non_empty<'a>(values: &[&'a str]) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("")
}
