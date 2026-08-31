use super::super::super::*;
use crate::ui::workbench::autolayout::ShellFrame;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::workspace_docking) fn begin_drawer_resize_capture(
        &mut self,
        x: f32,
        y: f32,
    ) {
        let Some(region) = self
            .shell_pointer_bridge
            .begin_resize(UiPoint::new(x, y))
            .and_then(|route| match route {
                HostShellPointerRoute::Resize(group) => Some(group.region()),
                HostShellPointerRoute::DragTarget(_)
                | HostShellPointerRoute::DocumentEdge(_)
                | HostShellPointerRoute::FloatingWindow(_)
                | HostShellPointerRoute::FloatingWindowEdge { .. } => None,
            })
        else {
            return;
        };
        let workbench_layout_frames = self.workbench_window_bridge.layout_frames();
        let Some(frame) = workbench_layout_frames
            .drawer_shell_frame(region)
            .filter(ui_frame_is_visible)
            .map(shell_frame)
        else {
            self.shell_pointer_bridge.cancel_resize();
            return;
        };
        let base_preferred = match region {
            ShellRegionId::Bottom => frame.height,
            ShellRegionId::Left | ShellRegionId::Right | ShellRegionId::Document => frame.width,
        };
        if base_preferred <= 0.0 {
            self.shell_pointer_bridge.cancel_resize();
            return;
        }

        self.active_drawer_resize = Some(ActiveDrawerResize {
            region,
            start_x: x,
            start_y: y,
            base_preferred,
        });
        self.update_drawer_resize_capture(x, y);
    }
}

fn shell_frame(frame: UiFrame) -> ShellFrame {
    ShellFrame::new(frame.x, frame.y, frame.width, frame.height)
}

fn ui_frame_is_visible(frame: &UiFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}
