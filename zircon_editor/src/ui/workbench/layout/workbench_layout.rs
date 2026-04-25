use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::workbench::autolayout::PaneConstraintOverride;
use crate::ui::workbench::autolayout::ShellRegionId;
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

use super::{
    ActivityDrawerLayout, ActivityDrawerSlot, ActivityWindowHostMode, ActivityWindowId,
    ActivityWindowLayout, FloatingWindowLayout, MainHostPageLayout, MainPageId,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkbenchLayout {
    pub active_main_page: MainPageId,
    pub main_pages: Vec<MainHostPageLayout>,
    pub drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    #[serde(default = "default_activity_windows")]
    pub activity_windows: BTreeMap<ActivityWindowId, ActivityWindowLayout>,
    pub floating_windows: Vec<FloatingWindowLayout>,
    #[serde(default)]
    pub region_overrides: BTreeMap<ShellRegionId, PaneConstraintOverride>,
    #[serde(default)]
    pub view_overrides: BTreeMap<ViewInstanceId, PaneConstraintOverride>,
}

impl Default for WorkbenchLayout {
    fn default() -> Self {
        let drawers = default_drawers();
        Self {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                document_workspace: super::DocumentNode::default(),
            }],
            drawers: drawers.clone(),
            activity_windows: default_activity_windows_with_drawers(drawers),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        }
    }
}

impl WorkbenchLayout {
    pub fn activity_windows(&self) -> BTreeMap<ActivityWindowId, ActivityWindowLayout> {
        if self.activity_windows.is_empty() {
            default_activity_windows_with_drawers(self.drawers.clone())
        } else {
            self.activity_windows.clone()
        }
    }

    pub fn default_activity_window_mut(&mut self) -> Option<&mut ActivityWindowLayout> {
        self.activity_windows
            .get_mut(&ActivityWindowId::new("window:workbench"))
    }
}

fn default_drawers() -> BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout> {
    ActivityDrawerSlot::ALL
        .into_iter()
        .map(|slot| (slot, ActivityDrawerLayout::new(slot)))
        .collect()
}

fn default_activity_windows_with_drawers(
    drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
) -> BTreeMap<ActivityWindowId, ActivityWindowLayout> {
    let window_id = ActivityWindowId::new("window:workbench");
    [(
        window_id.clone(),
        ActivityWindowLayout {
            window_id,
            descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
            host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
            activity_drawers: drawers,
            content_workspace: super::DocumentNode::default(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
    )]
    .into_iter()
    .collect()
}

fn default_activity_windows() -> BTreeMap<ActivityWindowId, ActivityWindowLayout> {
    default_activity_windows_with_drawers(default_drawers())
}
