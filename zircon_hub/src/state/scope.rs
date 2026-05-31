use std::path::{Path, PathBuf};

use crate::engines::SourceEngineInstall;
use crate::projects::{metadata_for_path, project_paths_match, ProjectMetadataMap, RecentProject};

/// Canonical action/view scope derived once from the Hub snapshot inputs.
/// UI and runtime code should consult this model instead of independently
/// guessing whether a page action targets a selected project, a fallback project,
/// an active Source Engine, or the whole Hub installation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HubScope {
    pub project: ProjectScope,
    pub source_engine: SourceEngineScope,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectScope {
    Selected(ProjectScopeProject),
    StaleSelection { requested_path: PathBuf },
    LatestRecent(ProjectScopeProject),
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectScopeProject {
    pub display_name: String,
    pub path: PathBuf,
    pub engine_id: Option<String>,
    pub engine_state: ProjectEngineScopeState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectEngineScopeState {
    Ready,
    MissingBinding,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceEngineScope {
    ProjectBound(SourceEngineScopeEngine),
    ProjectUnbound {
        project_name: String,
    },
    ProjectEngineUnavailable {
        project_name: String,
        engine_id: String,
    },
    Active(SourceEngineScopeEngine),
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceEngineScopeEngine {
    pub id: String,
    pub display_name: String,
}

impl HubScope {
    pub fn resolve(
        selected_project_path: Option<&Path>,
        recent_projects: &[RecentProject],
        project_metadata: &ProjectMetadataMap,
        engines: &[SourceEngineInstall],
        active_engine_id: Option<&str>,
    ) -> Self {
        let project = resolve_project_scope(
            selected_project_path,
            recent_projects,
            project_metadata,
            engines,
        );
        // Source Engine scope is intentionally derived after project scope so a
        // selected project can prevent accidental active-engine fallback.
        let source_engine = resolve_source_engine_scope(&project, engines, active_engine_id);
        Self {
            project,
            source_engine,
        }
    }

    pub fn selected_project(&self) -> Option<&ProjectScopeProject> {
        match &self.project {
            ProjectScope::Selected(project) => Some(project),
            ProjectScope::StaleSelection { .. }
            | ProjectScope::LatestRecent(_)
            | ProjectScope::None => None,
        }
    }

    pub fn selected_or_latest_project(&self) -> Option<&ProjectScopeProject> {
        match &self.project {
            ProjectScope::Selected(project) | ProjectScope::LatestRecent(project) => Some(project),
            ProjectScope::StaleSelection { .. } | ProjectScope::None => None,
        }
    }

    pub fn has_stale_selected_project(&self) -> bool {
        matches!(self.project, ProjectScope::StaleSelection { .. })
    }
}

impl ProjectScopeProject {
    pub fn can_build(&self) -> bool {
        self.engine_state == ProjectEngineScopeState::Ready
    }
}

impl SourceEngineScope {
    pub fn engine_id(&self) -> Option<&str> {
        match self {
            Self::ProjectBound(engine) | Self::Active(engine) => Some(&engine.id),
            Self::ProjectUnbound { .. } | Self::ProjectEngineUnavailable { .. } | Self::None => {
                None
            }
        }
    }
}

fn resolve_project_scope(
    selected_project_path: Option<&Path>,
    recent_projects: &[RecentProject],
    project_metadata: &ProjectMetadataMap,
    engines: &[SourceEngineInstall],
) -> ProjectScope {
    if let Some(selected_path) = selected_project_path {
        if let Some(project) = recent_projects
            .iter()
            .find(|project| project_paths_match(&project.path, selected_path))
        {
            return ProjectScope::Selected(project_scope_project(
                project,
                project_metadata,
                engines,
            ));
        }
        return ProjectScope::StaleSelection {
            requested_path: selected_path.to_path_buf(),
        };
    }

    recent_projects
        .iter()
        .max_by_key(|project| project.last_opened_unix_ms)
        .map(|project| {
            ProjectScope::LatestRecent(project_scope_project(project, project_metadata, engines))
        })
        .unwrap_or(ProjectScope::None)
}

fn project_scope_project(
    project: &RecentProject,
    project_metadata: &ProjectMetadataMap,
    engines: &[SourceEngineInstall],
) -> ProjectScopeProject {
    let engine_id = metadata_for_path(project_metadata, &project.path)
        .and_then(|metadata| metadata.engine_id.clone());
    let engine_state = match engine_id.as_deref() {
        None => ProjectEngineScopeState::MissingBinding,
        Some(id) if engines.iter().any(|engine| engine.id == id) => ProjectEngineScopeState::Ready,
        Some(_) => ProjectEngineScopeState::Unavailable,
    };
    ProjectScopeProject {
        display_name: project_display_name(project),
        path: project.path.clone(),
        engine_id,
        engine_state,
    }
}

fn resolve_source_engine_scope(
    project: &ProjectScope,
    engines: &[SourceEngineInstall],
    active_engine_id: Option<&str>,
) -> SourceEngineScope {
    match project {
        ProjectScope::Selected(project) => match project.engine_state {
            ProjectEngineScopeState::Ready => project
                .engine_id
                .as_deref()
                .and_then(|engine_id| engines.iter().find(|engine| engine.id == engine_id))
                .map(source_engine_scope_engine)
                .map(SourceEngineScope::ProjectBound)
                .unwrap_or_else(|| SourceEngineScope::ProjectEngineUnavailable {
                    project_name: project.display_name.clone(),
                    engine_id: project.engine_id.clone().unwrap_or_default(),
                }),
            ProjectEngineScopeState::MissingBinding => SourceEngineScope::ProjectUnbound {
                project_name: project.display_name.clone(),
            },
            ProjectEngineScopeState::Unavailable => SourceEngineScope::ProjectEngineUnavailable {
                project_name: project.display_name.clone(),
                engine_id: project.engine_id.clone().unwrap_or_default(),
            },
        },
        ProjectScope::StaleSelection { .. }
        | ProjectScope::LatestRecent(_)
        | ProjectScope::None => engines
            .iter()
            .find(|engine| active_engine_id == Some(engine.id.as_str()))
            .or_else(|| engines.first())
            .map(source_engine_scope_engine)
            .map(SourceEngineScope::Active)
            .unwrap_or(SourceEngineScope::None),
    }
}

fn source_engine_scope_engine(engine: &SourceEngineInstall) -> SourceEngineScopeEngine {
    SourceEngineScopeEngine {
        id: engine.id.clone(),
        display_name: engine.display_name.clone(),
    }
}

fn project_display_name(project: &RecentProject) -> String {
    if project.display_name.trim().is_empty() {
        return project
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Zircon Project")
            .to_string();
    }
    project.display_name.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projects::{metadata_for_path_mut, ProjectMetadataMap, RecentProject};

    fn engine(id: &str) -> SourceEngineInstall {
        SourceEngineInstall {
            id: id.to_string(),
            display_name: format!("{id} Engine"),
            source_dir: PathBuf::from(format!("E:/{id}")),
            output_dir: PathBuf::from(format!("E:/out/{id}")),
            last_build_unix_ms: None,
            build_history: Vec::new(),
        }
    }

    #[test]
    fn scope_prefers_selected_project_and_project_bound_engine() {
        let projects = vec![
            RecentProject::new("Latest", "E:/Projects/Latest", 20),
            RecentProject::new("Selected", "E:/Projects/Selected", 10),
        ];
        let engines = vec![engine("local")];
        let mut metadata = ProjectMetadataMap::new();
        metadata_for_path_mut(&mut metadata, "E:/Projects/Selected").engine_id =
            Some("local".to_string());

        let scope = HubScope::resolve(
            Some(Path::new("E:/Projects/Selected")),
            &projects,
            &metadata,
            &engines,
            None,
        );

        assert_eq!(scope.selected_project().unwrap().display_name, "Selected");
        assert_eq!(scope.source_engine.engine_id(), Some("local"));
        assert!(scope.selected_project().unwrap().can_build());
    }

    #[test]
    fn stale_selected_project_does_not_fallback_to_latest_recent() {
        let projects = vec![RecentProject::new("Latest", "E:/Projects/Latest", 20)];
        let scope = HubScope::resolve(
            Some(Path::new("E:/Projects/Missing")),
            &projects,
            &ProjectMetadataMap::new(),
            &[],
            None,
        );

        assert!(scope.has_stale_selected_project());
        assert!(scope.selected_or_latest_project().is_none());
    }

    #[test]
    fn no_selection_uses_latest_recent_and_active_engine_scope() {
        let projects = vec![
            RecentProject::new("Old", "E:/Projects/Old", 1),
            RecentProject::new("Latest", "E:/Projects/Latest", 20),
        ];
        let engines = vec![engine("first"), engine("active")];

        let scope = HubScope::resolve(
            None,
            &projects,
            &ProjectMetadataMap::new(),
            &engines,
            Some("active"),
        );

        assert_eq!(
            scope.selected_or_latest_project().unwrap().display_name,
            "Latest"
        );
        assert_eq!(scope.source_engine.engine_id(), Some("active"));
    }

    #[test]
    fn selected_project_without_engine_binding_reports_project_unbound() {
        let projects = vec![RecentProject::new("Game", "E:/Projects/Game", 20)];

        let scope = HubScope::resolve(
            Some(Path::new("E:/Projects/Game")),
            &projects,
            &ProjectMetadataMap::new(),
            &[engine("active")],
            Some("active"),
        );

        assert_eq!(
            scope.source_engine,
            SourceEngineScope::ProjectUnbound {
                project_name: "Game".to_string()
            }
        );
    }

    #[test]
    fn selected_project_with_missing_engine_reports_unavailable_binding() {
        let projects = vec![RecentProject::new("Game", "E:/Projects/Game", 20)];
        let mut metadata = ProjectMetadataMap::new();
        metadata_for_path_mut(&mut metadata, "E:/Projects/Game").engine_id =
            Some("missing".to_string());

        let scope = HubScope::resolve(
            Some(Path::new("E:/Projects/Game")),
            &projects,
            &metadata,
            &[engine("active")],
            Some("active"),
        );

        assert_eq!(
            scope.source_engine,
            SourceEngineScope::ProjectEngineUnavailable {
                project_name: "Game".to_string(),
                engine_id: "missing".to_string()
            }
        );
        assert!(!scope.selected_project().unwrap().can_build());
    }

    #[test]
    fn active_engine_scope_falls_back_to_first_engine_then_none() {
        let projects = vec![RecentProject::new("Latest", "E:/Projects/Latest", 20)];

        let first_fallback = HubScope::resolve(
            None,
            &projects,
            &ProjectMetadataMap::new(),
            &[engine("first"), engine("second")],
            Some("missing"),
        );
        let no_engine = HubScope::resolve(
            None,
            &projects,
            &ProjectMetadataMap::new(),
            &[],
            Some("missing"),
        );

        assert_eq!(first_fallback.source_engine.engine_id(), Some("first"));
        assert_eq!(no_engine.source_engine, SourceEngineScope::None);
    }
}
