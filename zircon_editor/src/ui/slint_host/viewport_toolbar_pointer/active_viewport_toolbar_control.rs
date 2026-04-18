use zircon_ui::UiFrame;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ActiveViewportToolbarControl {
    pub(super) action_key: String,
    pub(super) frame: UiFrame,
}
