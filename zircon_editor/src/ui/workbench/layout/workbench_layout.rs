use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewDescriptorId;

use super::document_node_id::DocumentNodeIdRepair;
use super::{
    ActivityDrawerLayout, ActivityDrawerSlot, ActivityWindowHostMode, ActivityWindowId,
    ActivityWindowLayout, DocumentNode, FloatingWindowLayout, MainHostPageLayout, MainPageId,
};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLayout {
    pub active_main_page: MainPageId,
    pub main_pages: Vec<MainHostPageLayout>,
    pub activity_windows: BTreeMap<ActivityWindowId, ActivityWindowLayout>,
    pub floating_windows: Vec<FloatingWindowLayout>,
}

mod deserialize;

impl Default for WorkbenchLayout {
    fn default() -> Self {
        let drawers = default_drawers();
        Self {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                activity_window: ActivityWindowId::workbench(),
            }],
            activity_windows: default_activity_windows(drawers),
            floating_windows: Vec::new(),
        }
    }
}

impl WorkbenchLayout {
    pub(crate) fn normalize_document_node_ids(&mut self) {
        self.normalize_document_node_ids_with_priority(None);
    }

    pub(crate) fn normalize_document_node_ids_prioritizing(
        &mut self,
        activity_window_id: &ActivityWindowId,
    ) {
        self.normalize_document_node_ids_with_priority(Some(activity_window_id));
    }

    fn normalize_document_node_ids_with_priority(&mut self, priority: Option<&ActivityWindowId>) {
        let mut reserved = HashSet::new();
        for window in self.activity_windows.values() {
            window.content_workspace.collect_node_ids(&mut reserved);
        }
        for window in &self.floating_windows {
            window.workspace.collect_node_ids(&mut reserved);
        }

        let mut repair = DocumentNodeIdRepair::new(reserved);
        if let Some(window) = priority.and_then(|id| self.activity_windows.get_mut(id)) {
            window.content_workspace.normalize_node_ids(&mut repair);
        }
        for (id, window) in &mut self.activity_windows {
            if priority == Some(id) {
                continue;
            }
            window.content_workspace.normalize_node_ids(&mut repair);
        }
        for window in &mut self.floating_windows {
            window.workspace.normalize_node_ids(&mut repair);
        }
    }

    pub fn activity_windows(&self) -> &BTreeMap<ActivityWindowId, ActivityWindowLayout> {
        &self.activity_windows
    }

    pub fn content_workspace_for_page(&self, page_id: &MainPageId) -> Option<&DocumentNode> {
        let window_id = self
            .main_pages
            .iter()
            .find(|page| page.id() == page_id)?
            .activity_window_id()?;
        self.activity_windows
            .get(window_id)
            .map(|window| &window.content_workspace)
    }

    pub fn content_workspace_for_page_mut(
        &mut self,
        page_id: &MainPageId,
    ) -> Option<&mut DocumentNode> {
        let window_id = self
            .main_pages
            .iter()
            .find(|page| page.id() == page_id)?
            .activity_window_id()?
            .clone();
        self.activity_windows
            .get_mut(&window_id)
            .map(|window| &mut window.content_workspace)
    }

    pub(crate) fn ensure_workbench_content_workspace(&mut self) -> &mut DocumentNode {
        let window_id = self
            .main_pages
            .iter()
            .find_map(|page| page.activity_window_id().cloned())
            .unwrap_or_else(|| {
                let window_id = ActivityWindowId::workbench();
                self.main_pages.insert(
                    0,
                    MainHostPageLayout::WorkbenchPage {
                        id: MainPageId::workbench(),
                        title: "Workbench".to_string(),
                        activity_window: window_id.clone(),
                    },
                );
                window_id
            });
        &mut self
            .activity_windows
            .entry(window_id.clone())
            .or_insert_with(|| default_activity_window(window_id, default_drawers()))
            .content_workspace
    }

    pub fn active_activity_window_drawers(
        &self,
    ) -> &BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout> {
        static EMPTY_ACTIVITY_DRAWERS: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout> =
            BTreeMap::new();

        self.active_activity_window()
            .map(|window| &window.activity_drawers)
            .unwrap_or(&EMPTY_ACTIVITY_DRAWERS)
    }

    pub fn default_activity_window_mut(&mut self) -> Option<&mut ActivityWindowLayout> {
        self.activity_windows
            .get_mut(&ActivityWindowId::workbench())
    }

    pub fn active_activity_window_id(&self) -> Option<ActivityWindowId> {
        self.main_pages
            .iter()
            .find(|page| page.id() == &self.active_main_page)
            .and_then(|page| page.activity_window_id().cloned())
    }

    pub fn active_activity_window(&self) -> Option<&ActivityWindowLayout> {
        let window_id = self.active_activity_window_id()?;
        self.activity_windows.get(&window_id)
    }

    pub fn active_activity_window_mut(&mut self) -> Option<&mut ActivityWindowLayout> {
        let window_id = self.active_activity_window_id()?;
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

fn default_activity_windows(
    drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
) -> BTreeMap<ActivityWindowId, ActivityWindowLayout> {
    let window_id = ActivityWindowId::workbench();
    [(
        window_id.clone(),
        default_activity_window(window_id, drawers),
    )]
    .into_iter()
    .collect()
}

fn default_activity_window(
    window_id: ActivityWindowId,
    drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
) -> ActivityWindowLayout {
    ActivityWindowLayout {
        window_id,
        descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
        host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
        activity_drawers: drawers,
        content_workspace: DocumentNode::default(),
        menu_overflow_mode: Default::default(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    }
}
