use serde::{Deserialize, Serialize};

use crate::ui::binding::UiBindingUpdateReport;
use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiNavigationRoute;

use super::UiNavigationDispatchInvocation;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiNavigationDispatchResult {
    pub route: UiNavigationRoute,
    pub invocations: Vec<UiNavigationDispatchInvocation>,
    pub handled_by: Option<UiNodeId>,
    pub focus_changed_to: Option<UiNodeId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub binding_reports: Vec<UiBindingUpdateReport>,
}

impl UiNavigationDispatchResult {
    pub fn new(route: UiNavigationRoute) -> Self {
        Self {
            route,
            invocations: Vec::new(),
            handled_by: None,
            focus_changed_to: None,
            binding_reports: Vec::new(),
        }
    }

    pub fn record_binding_report(&mut self, report: UiBindingUpdateReport) {
        if !report.updates.is_empty()
            || report.applied_count > 0
            || report.unchanged_count > 0
            || report.rejected_count > 0
        {
            self.binding_reports.push(report);
        }
    }
}
