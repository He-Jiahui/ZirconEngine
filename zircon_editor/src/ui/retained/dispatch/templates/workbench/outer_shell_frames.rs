use zircon_runtime_interface::ui::layout::UiFrame;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BuiltinHostOuterShellFrames {
    pub shell_frame: Option<UiFrame>,
    pub menu_bar_frame: Option<UiFrame>,
    pub host_page_strip_frame: Option<UiFrame>,
}
