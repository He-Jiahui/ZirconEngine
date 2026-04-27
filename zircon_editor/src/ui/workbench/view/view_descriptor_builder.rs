use crate::ui::workbench::autolayout::PaneConstraints;
use crate::ui::workbench::layout::ActivityDrawerSlot;

use super::{
    ActivityWindowTemplateSpec, DockPolicy, PaneTemplateSpec, PreferredHost, ViewDescriptor,
};

impl ViewDescriptor {
    pub fn with_multi_instance(mut self, multi_instance: bool) -> Self {
        self.multi_instance = multi_instance;
        self
    }

    pub fn with_dock_policy(mut self, dock_policy: DockPolicy) -> Self {
        self.dock_policy = dock_policy;
        self
    }

    pub fn with_preferred_drawer_slot(mut self, slot: ActivityDrawerSlot) -> Self {
        self.preferred_drawer_slot = Some(slot);
        self.preferred_host = PreferredHost::Drawer(slot);
        self
    }

    pub fn with_preferred_host(mut self, preferred_host: PreferredHost) -> Self {
        self.preferred_host = preferred_host;
        self
    }

    pub fn with_icon_key(mut self, icon_key: impl Into<String>) -> Self {
        self.icon_key = icon_key.into();
        self
    }

    pub fn with_default_constraints(mut self, constraints: PaneConstraints) -> Self {
        self.default_constraints = constraints;
        self
    }

    pub fn with_pane_template(mut self, pane_template: PaneTemplateSpec) -> Self {
        self.pane_template = Some(pane_template);
        self
    }

    pub fn with_activity_window_template(
        mut self,
        activity_window_template: ActivityWindowTemplateSpec,
    ) -> Self {
        self.activity_window_template = Some(activity_window_template);
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_capabilities = capabilities.into_iter().map(Into::into).collect();
        self
    }
}
