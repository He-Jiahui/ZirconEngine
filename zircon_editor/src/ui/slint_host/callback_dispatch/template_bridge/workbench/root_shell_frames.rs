use zircon_runtime::ui::layout::UiFrame;

use crate::ui::workbench::autolayout::ShellRegionId;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BuiltinWorkbenchRootShellFrames {
    pub shell_frame: Option<UiFrame>,
    pub menu_bar_frame: Option<UiFrame>,
    pub activity_rail_frame: Option<UiFrame>,
    pub host_page_strip_frame: Option<UiFrame>,
    pub workbench_body_frame: Option<UiFrame>,
    pub left_drawer_shell_frame: Option<UiFrame>,
    pub left_drawer_header_frame: Option<UiFrame>,
    pub left_drawer_content_frame: Option<UiFrame>,
    pub right_drawer_shell_frame: Option<UiFrame>,
    pub right_drawer_header_frame: Option<UiFrame>,
    pub right_drawer_content_frame: Option<UiFrame>,
    pub bottom_drawer_shell_frame: Option<UiFrame>,
    pub bottom_drawer_header_frame: Option<UiFrame>,
    pub bottom_drawer_content_frame: Option<UiFrame>,
    pub document_host_frame: Option<UiFrame>,
    pub document_tabs_frame: Option<UiFrame>,
    pub pane_surface_frame: Option<UiFrame>,
    pub status_bar_frame: Option<UiFrame>,
}

impl BuiltinWorkbenchRootShellFrames {
    pub(crate) fn drawer_shell_frame(&self, region: ShellRegionId) -> Option<UiFrame> {
        match region {
            ShellRegionId::Left => self.left_drawer_shell_frame,
            ShellRegionId::Right => self.right_drawer_shell_frame,
            ShellRegionId::Bottom => self.bottom_drawer_shell_frame,
            ShellRegionId::Document => None,
        }
    }

    pub(crate) fn drawer_header_frame(&self, region: ShellRegionId) -> Option<UiFrame> {
        match region {
            ShellRegionId::Left => self.left_drawer_header_frame,
            ShellRegionId::Right => self.right_drawer_header_frame,
            ShellRegionId::Bottom => self.bottom_drawer_header_frame,
            ShellRegionId::Document => None,
        }
    }

    pub(crate) fn drawer_content_frame(&self, region: ShellRegionId) -> Option<UiFrame> {
        match region {
            ShellRegionId::Left => self.left_drawer_content_frame,
            ShellRegionId::Right => self.right_drawer_content_frame,
            ShellRegionId::Bottom => self.bottom_drawer_content_frame,
            ShellRegionId::Document => None,
        }
    }
}
