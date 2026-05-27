use crate::error::HubError;

use super::HubRuntime;

impl HubRuntime {
    pub(super) fn refresh_source_scoped_views(&mut self) -> Result<(), HubError> {
        self.refresh_asset_catalog()?;
        self.refresh_learn_catalog()?;
        self.refresh_plugin_catalog()?;
        self.refresh_team_overview()
    }
}
