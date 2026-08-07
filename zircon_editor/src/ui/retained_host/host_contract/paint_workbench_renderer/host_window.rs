use super::super::data::{paint_pane_interaction_state, HostWindowPresentationData};
use super::super::paint_frame::HostRgbaFrame;
use super::root_frames::resolve_root_frames;
use super::scene_layers::draw_host_scene;
use super::skeleton::draw_root_skeleton;

pub(in crate::ui::retained_host::host_contract) fn draw_host_workbench_window(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let interaction = paint_pane_interaction_state(presentation);
    frame.set_pane_interaction_state(&interaction);
    let root = resolve_root_frames(frame.width(), frame.height(), presentation);
    draw_root_skeleton(frame, &root, presentation);
    draw_host_scene(frame, &root, presentation);
}

pub(in crate::ui::retained_host::host_contract) fn draw_host_workbench_window_profiled(
    frame: &mut HostRgbaFrame,
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
    _resolve_scope: &'static str,
    _skeleton_scope: &'static str,
    _scene_scope: &'static str,
) {
    let interaction = paint_pane_interaction_state(presentation);
    frame.set_pane_interaction_state(&interaction);
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
