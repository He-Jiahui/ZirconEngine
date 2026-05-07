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
    #[serde(default)]
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
                activity_window: ActivityWindowId::workbench(),
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
            let mut windows = self.activity_windows.clone();
            for window in windows.values_mut() {
                window.activity_drawers =
                    canonical_activity_drawers(std::mem::take(&mut window.activity_drawers));
            }
            windows
        }
    }

    pub fn active_activity_window_drawers(
        &self,
    ) -> BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout> {
        let Some(active_window_id) = self.active_activity_window_id() else {
            return BTreeMap::new();
        };
        self.activity_windows()
            .get(&active_window_id)
            .map(|window| window.activity_drawers.clone())
            .unwrap_or_default()
    }

    pub(crate) fn sync_legacy_drawers_from_active_activity_window(&mut self) {
        self.drawers = self.active_activity_window_drawers();
    }

    pub fn default_activity_window_mut(&mut self) -> Option<&mut ActivityWindowLayout> {
        if self.activity_windows.is_empty() {
            self.activity_windows = default_activity_windows_with_drawers(self.drawers.clone());
        }
        self.activity_windows
            .get_mut(&ActivityWindowId::workbench())
    }

    pub fn active_activity_window_id(&self) -> Option<ActivityWindowId> {
        self.main_pages
            .iter()
            .find(|page| page.id() == &self.active_main_page)
            .and_then(|page| page.activity_window_id().cloned())
    }

    pub fn active_activity_window_mut(&mut self) -> Option<&mut ActivityWindowLayout> {
        let window_id = self.active_activity_window_id()?;
        if self.activity_windows.is_empty() && window_id == ActivityWindowId::workbench() {
            self.activity_windows = default_activity_windows_with_drawers(self.drawers.clone());
        }
        self.activity_windows.get_mut(&window_id)
    }

    pub fn page_id_for_activity_window(&self, window_id: &ActivityWindowId) -> Option<MainPageId> {
        self.main_pages
            .iter()
            .find(|page| page.activity_window_id() == Some(window_id))
            .map(|page| page.id().clone())
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
    activity_windows_from_legacy_drawers(drawers)
}

pub(crate) fn activity_windows_from_legacy_drawers(
    drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
) -> BTreeMap<ActivityWindowId, ActivityWindowLayout> {
    let window_id = ActivityWindowId::workbench();
    let drawers = canonical_activity_drawers(drawers);
    [(
        window_id.clone(),
        ActivityWindowLayout {
            window_id,
            descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
            host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
            activity_drawers: drawers,
            content_workspace: super::DocumentNode::default(),
            menu_overflow_mode: Default::default(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
    )]
    .into_iter()
    .collect()
}

pub(crate) fn canonical_activity_drawers(
    drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
) -> BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout> {
    let mut canonical = BTreeMap::new();
    for (slot, mut drawer) in drawers {
        let slot = slot.canonical();
        drawer.slot = slot;
        if let Some(existing) = canonical.get_mut(&slot) {
            merge_drawer_layout(existing, drawer);
        } else {
            canonical.insert(slot, drawer);
        }
    }
    canonical
}

fn merge_drawer_layout(existing: &mut ActivityDrawerLayout, incoming: ActivityDrawerLayout) {
    for tab in incoming.tab_stack.tabs {
        if !existing.tab_stack.tabs.contains(&tab) {
            existing.tab_stack.tabs.push(tab);
        }
    }

    if existing.active_view.is_none() && incoming.active_view.is_some() {
        existing.active_view = incoming.active_view.clone();
        existing.tab_stack.active_tab = incoming
            .tab_stack
            .active_tab
            .or_else(|| existing.active_view.clone());
        existing.mode = incoming.mode;
        existing.extent = incoming.extent;
    }
    existing.visible |= incoming.visible;
}
