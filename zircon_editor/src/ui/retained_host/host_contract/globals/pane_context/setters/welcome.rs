use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::data::{ProjectOverviewData, RecentProjectData, WelcomePaneData};
use super::super::PaneSurfaceHostContext;

impl PaneSurfaceHostContext<'_> {
    pub(crate) fn set_recent_projects(&self, _value: ModelRc<RecentProjectData>) {}
    pub(crate) fn set_project_overview(&self, _value: ProjectOverviewData) {}

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
