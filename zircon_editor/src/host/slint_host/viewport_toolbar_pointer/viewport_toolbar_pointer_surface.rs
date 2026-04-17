use zircon_ui::UiFrame;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportToolbarPointerSurface {
    pub key: String,
    pub frame: UiFrame,
}
