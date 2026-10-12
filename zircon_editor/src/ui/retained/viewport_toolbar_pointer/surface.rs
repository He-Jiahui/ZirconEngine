use zircon_runtime_interface::ui::layout::UiFrame;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportToolbarPointerSurface {
    pub key: String,
    pub frame: UiFrame,
}
