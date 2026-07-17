use super::super::super::super::data::WelcomePaneData;
use super::super::PaneSurfaceHostContext;

impl PaneSurfaceHostContext<'_> {
    pub(crate) fn set_welcome_pane(&self, value: WelcomePaneData) {
        self.state.borrow_mut().welcome_pane = value;
    }

    pub(crate) fn get_welcome_pane(&self) -> WelcomePaneData {
        self.state.borrow().welcome_pane.clone()
    }

    pub(crate) fn set_welcome_recent_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_hovered_welcome_recent_index(&self, _value: i32) {}
    pub(crate) fn set_hovered_welcome_recent_action(&self, _value: i32) {}
}
