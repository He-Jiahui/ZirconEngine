use super::welcome_recent_pointer_action::WelcomeRecentPointerAction;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WelcomeRecentPointerState {
    pub hovered_item_index: Option<usize>,
    pub hovered_action: Option<WelcomeRecentPointerAction>,
    pub scroll_offset: f32,
}
