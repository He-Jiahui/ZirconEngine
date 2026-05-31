use crate::error::HubError;
use crate::team::discover_team_overview;

use super::{root_paths::push_unique_root, HubRuntime};

impl HubRuntime {
    pub(super) fn refresh_team_overview(&mut self) -> Result<(), HubError> {
        let mut roots = Vec::new();
        if let Some(project_root) = self.selected_project_catalog_root() {
            push_unique_root(&mut roots, project_root);
        }
        for source_root in self.source_engine_catalog_roots() {
            push_unique_root(&mut roots, source_root);
        }
        self.team_overview = discover_team_overview(roots)?;
        Ok(())
    }
}
