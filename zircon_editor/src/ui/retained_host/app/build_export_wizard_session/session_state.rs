mod actions;
mod lookup;
mod polling;

use std::collections::BTreeMap;

use crate::ui::host::{ExportWizardPanelSession, ExportWizardPanelUpdate};

// Owns retained app export-wizard state by profile so the pane projection can
// refresh from host state instead of rebuilding a synthetic view model each frame.
#[derive(Default)]
pub(in crate::ui::retained_host::app) struct DesktopExportWizardSessions {
    pub(super) sessions: BTreeMap<String, ExportWizardPanelSession>,
    last_updates: BTreeMap<String, ExportWizardPanelUpdate>,
}
