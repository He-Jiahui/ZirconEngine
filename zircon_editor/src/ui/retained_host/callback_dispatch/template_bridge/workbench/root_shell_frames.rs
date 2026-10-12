use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BuiltinHostRootShellFrames {
    pub shell_frame: Option<UiFrame>,
    pub menu_bar_frame: Option<UiFrame>,
    pub activity_rail_frame: Option<UiFrame>,
    pub host_page_strip_frame: Option<UiFrame>,
    pub host_body_frame: Option<UiFrame>,
    pub document_host_frame: Option<UiFrame>,
    pub document_tabs_frame: Option<UiFrame>,
    pub pane_surface_frame: Option<UiFrame>,
    pub status_bar_frame: Option<UiFrame>,
}

impl BuiltinHostRootShellFrames {
    // The level editor owns its menu, toolbar, tabs and status. The outer shell
    // contributes only the actual pane surface for the complete document.
    pub(crate) fn componentized_workbench_mount_frame(self, _shell_size: UiSize) -> UiFrame {
        self.pane_surface_frame
            .filter(visible_frame)
            .or_else(|| self.document_host_frame.filter(visible_frame))
            .or_else(|| self.shell_frame.filter(visible_frame))
            .expect("editor shell must expose a visible pane, document host or shell frame")
    }
}

fn visible_frame(frame: &UiFrame) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > f32::EPSILON
        && frame.height > f32::EPSILON
}

#[cfg(test)]
#[path = "tests/root_shell_frames.rs"]
mod tests;
