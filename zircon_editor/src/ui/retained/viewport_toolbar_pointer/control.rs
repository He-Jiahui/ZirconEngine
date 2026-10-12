use zircon_runtime_interface::ui::layout::UiFrame;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ViewportToolbarPointerControl {
    pub(super) action_key: String,
    pub(super) frame: UiFrame,
}
