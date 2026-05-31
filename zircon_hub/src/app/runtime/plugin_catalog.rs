use crate::error::HubError;
use crate::plugins::discover_plugin_catalog_with_project_roots;

use super::HubRuntime;

impl HubRuntime {
    pub(super) fn refresh_plugin_catalog(&mut self) -> Result<(), HubError> {
        self.plugin_catalog = discover_plugin_catalog_with_project_roots(
            self.selected_project_catalog_root().into_iter(),
            self.source_engine_catalog_roots(),
        )?;
        Ok(())
    }
}
