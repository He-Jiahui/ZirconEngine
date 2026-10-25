use super::super::data::HostWindowPresentationData;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_workbench_renderer as workbench;

pub(in crate::ui::retained_host::host_contract) fn draw_workbench_presentation_commands(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    workbench::draw_componentized_workbench_window(frame, presentation);
    if !presentation.close_prompt.visible && !presentation.asset_deletion_blocker.visible {
        if let Some(popup) = presentation.canvas_anchor_popup.as_ref() {
            super::super::paint_template_nodes::draw_template_nodes(
                frame,
                &popup.nodes,
                &super::super::data::FrameRect::default(),
                &popup.bounds,
                None,
            );
        }
    }
}
