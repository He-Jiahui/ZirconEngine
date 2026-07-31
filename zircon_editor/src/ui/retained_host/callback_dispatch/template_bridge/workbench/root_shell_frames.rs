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
    // Menu/page chrome stays host-owned; the componentized Workbench owns the
    // remaining height, including its own toolbar and bottom status region.
    pub(crate) fn componentized_workbench_mount_frame(self, shell_size: UiSize) -> UiFrame {
        let mount_y = self
            .host_body_frame
            .filter(visible_frame)
            .map(|frame| frame.y)
            .or_else(|| {
                self.host_page_strip_frame
                    .filter(visible_frame)
                    .map(UiFrame::bottom)
            })
            .or_else(|| {
                self.menu_bar_frame
                    .filter(visible_frame)
                    .map(UiFrame::bottom)
            })
            .unwrap_or(0.0)
            .clamp(0.0, shell_size.height.max(0.0));
        UiFrame::new(
            0.0,
            mount_y,
            shell_size.width.max(0.0),
            (shell_size.height - mount_y).max(0.0),
        )
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
mod tests {
    use super::*;

    #[test]
    fn componentized_workbench_mount_starts_after_host_chrome_and_keeps_window_bottom() {
        let frames = BuiltinHostRootShellFrames {
            menu_bar_frame: Some(UiFrame::new(0.0, 0.0, 1280.0, 24.0)),
            host_page_strip_frame: Some(UiFrame::new(0.0, 24.0, 1280.0, 32.0)),
            host_body_frame: Some(UiFrame::new(0.0, 57.0, 1280.0, 639.0)),
            ..Default::default()
        };

        assert_eq!(
            frames.componentized_workbench_mount_frame(UiSize::new(1280.0, 720.0)),
            UiFrame::new(0.0, 57.0, 1280.0, 663.0)
        );
    }
}
