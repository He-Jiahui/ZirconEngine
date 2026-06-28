use super::super::data::HostWindowPresentationData;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_workbench_renderer as workbench;

pub(in crate::ui::retained_host::host_contract) fn draw_workbench_presentation_commands(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    if workbench::draws_componentized_workbench_window(presentation) {
        workbench::draw_componentized_workbench_window(frame, presentation);
    } else {
        workbench::draw_host_workbench_window(frame, presentation);
    }
}
