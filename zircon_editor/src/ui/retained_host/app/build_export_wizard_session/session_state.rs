mod actions;
mod lookup;
mod polling;

use std::collections::BTreeMap;

use crate::core::jobs::EditorJobSystem;
use crate::ui::host::ExportWizardPanelSession;

// Owns retained app export-wizard state by profile so the pane projection can
// refresh from host state instead of rebuilding a synthetic view model each frame.
pub(in crate::ui::retained_host::app) struct DesktopExportWizardSessions {
    pub(super) jobs: EditorJobSystem,
    pub(super) sessions: BTreeMap<String, ExportWizardPanelSession>,
}

impl DesktopExportWizardSessions {
    pub(in crate::ui::retained_host::app) fn new(jobs: EditorJobSystem) -> Self {
        Self {
            jobs,
            sessions: BTreeMap::new(),
        }
    }
}
