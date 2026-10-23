use super::super::data::HostWindowPresentationData;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_workbench_renderer as workbench;

#[path = "play_transition.rs"]
mod play_transition;

pub(in crate::ui::retained_host::host_contract) fn draw_workbench_presentation_commands(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    workbench::draw_componentized_workbench_window(frame, presentation);
    play_transition::draw(frame, presentation);
}
