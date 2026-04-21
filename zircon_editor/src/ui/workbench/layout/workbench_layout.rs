use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::workbench::autolayout::PaneConstraintOverride;
use crate::ui::workbench::autolayout::ShellRegionId;
use crate::ui::workbench::view::ViewInstanceId;

use super::{
    ActivityDrawerLayout, ActivityDrawerSlot, FloatingWindowLayout, MainHostPageLayout, MainPageId,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkbenchLayout {
    pub active_main_page: MainPageId,
    pub main_pages: Vec<MainHostPageLayout>,
    pub drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    pub floating_windows: Vec<FloatingWindowLayout>,
    #[serde(default)]
    pub region_overrides: BTreeMap<ShellRegionId, PaneConstraintOverride>,
    #[serde(default)]
    pub view_overrides: BTreeMap<ViewInstanceId, PaneConstraintOverride>,
}

impl Default for WorkbenchLayout {
    fn default() -> Self {
        Self {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                document_workspace: super::DocumentNode::default(),
            }],
            drawers: ActivityDrawerSlot::ALL
                .into_iter()
                .map(|slot| (slot, ActivityDrawerLayout::new(slot)))
                .collect(),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        }
    }
}
