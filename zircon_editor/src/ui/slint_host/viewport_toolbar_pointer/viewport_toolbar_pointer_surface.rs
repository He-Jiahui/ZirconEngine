use zircon_runtime::ui::layout::UiFrame;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportToolbarPointerSurface {
    pub key: String,
    pub frame: UiFrame,
}
