use std::path::PathBuf;

use crate::error::HubError;
use crate::team::discover_team_overview;

use super::HubRuntime;

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
        push_non_empty(&mut roots, selected_project_path);
    }
    push_non_empty(&mut roots, source_dir);
    if let Ok(current_dir) = std::env::current_dir() {
        push_non_empty(&mut roots, current_dir);
    }
    if let Some(compiled_repo_root) = compiled_repo_root() {
        push_non_empty(&mut roots, compiled_repo_root);
    }
    roots
}

fn push_non_empty(roots: &mut Vec<PathBuf>, path: PathBuf) {
    if path.as_os_str().is_empty() || roots.iter().any(|root| root == &path) {
        return;
    }
    roots.push(path);
}

fn compiled_repo_root() -> Option<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.to_path_buf())
}
