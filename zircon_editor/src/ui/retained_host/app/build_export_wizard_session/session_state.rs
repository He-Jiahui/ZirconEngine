mod actions;
mod lookup;
mod polling;

use std::cell::RefCell;
use std::collections::HashMap;

use crate::core::context::ToolSchedulerService;
use crate::core::jobs::EditorJobSystem;
use crate::ui::host::ExportWizardPanelSession;

use super::super::build_export_projection::cache::BuildExportProjectionCache;

// Owns retained app export-wizard state by profile so the pane projection can
// refresh from host state instead of rebuilding a synthetic view model each frame.
pub(in crate::ui::retained_host::app) struct DesktopExportWizardSessions {
    pub(super) jobs: EditorJobSystem,
    pub(super) tools: ToolSchedulerService,
    pub(super) sessions: HashMap<String, ExportWizardPanelSession>,
    projection_cache: RefCell<BuildExportProjectionCache>,
    projection_overlay_generation: u64,
}

impl DesktopExportWizardSessions {
    pub(in crate::ui::retained_host::app) fn new(jobs: EditorJobSystem) -> Self {
        Self::new_with_tools(
            jobs,
            ToolSchedulerService::new(
                crate::core::editor_message::SharedEditorMessageBus::default(),
            ),
        )
    }

    pub(in crate::ui::retained_host::app) fn new_with_tools(
        jobs: EditorJobSystem,
        tools: ToolSchedulerService,
    ) -> Self {
        Self {
            jobs,
            tools,
            sessions: HashMap::new(),
            projection_cache: RefCell::new(BuildExportProjectionCache::default()),
            projection_overlay_generation: 0,
        }
    }

    pub(in crate::ui::retained_host::app) fn projection_cache(
        &self,
    ) -> &RefCell<BuildExportProjectionCache> {
        &self.projection_cache
    }

    pub(in crate::ui::retained_host::app) const fn projection_overlay_generation(&self) -> u64 {
        self.projection_overlay_generation
    }

    pub(in crate::ui::retained_host::app) fn invalidate_projection_source(&mut self) {
        self.projection_overlay_generation = self.projection_overlay_generation.saturating_add(1);
        self.projection_cache.get_mut().invalidate_source();
    }

    pub(in crate::ui::retained_host::app) fn invalidate_projection_overlay(&mut self) {
        self.projection_overlay_generation = self.projection_overlay_generation.saturating_add(1);
        self.projection_cache.get_mut().invalidate_overlay();
    }
}

#[cfg(test)]
#[path = "session_state/hash_index_tests.rs"]
mod hash_index_tests;
