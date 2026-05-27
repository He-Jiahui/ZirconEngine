use std::path::PathBuf;

use crate::error::HubError;
use crate::team::discover_team_overview;

use super::{
    root_paths::{push_development_roots, push_unique_root},
    HubRuntime,
};

impl HubRuntime {
    pub(super) fn refresh_team_overview(&mut self) -> Result<(), HubError> {
        self.team_overview = discover_team_overview(team_overview_roots(
            self.selected_project_path.clone(),
            self.config.settings.default_source_dir.clone(),
        ))?;
        Ok(())
    }
}

fn team_overview_roots(
    selected_project_path: Option<PathBuf>,
    source_dir: PathBuf,
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(selected_project_path) = selected_project_path {
        push_unique_root(&mut roots, selected_project_path);
    }
    push_development_roots(&mut roots, source_dir);
    roots
}
