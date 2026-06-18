use zircon_runtime_interface::ui::layout::UiFrame;

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
