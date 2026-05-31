use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engines::SourceEngineInstall;
use crate::error::HubError;
use crate::projects::{
    project_metadata_key, project_paths_match, ProjectMetadataMap, ProjectTemplate, RecentProject,
    RECENT_PROJECT_LIMIT,
};
use crate::state::{
    HubActionRecord, HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode,
    ACTION_HISTORY_LIMIT,
};

use super::{
    default_build_output_dir, default_device_install_dir, default_project_dir, default_source_dir,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubConfig {
    #[serde(default)]
    pub settings: HubSettings,
    #[serde(default)]
    pub recent_projects: Vec<RecentProject>,
    #[serde(default)]
    pub project_metadata: ProjectMetadataMap,
    #[serde(default)]
    pub engines: Vec<SourceEngineInstall>,
    #[serde(default)]
    pub active_engine_id: Option<String>,
    #[serde(default)]
    pub window: HubWindowState,
    #[serde(default)]
    pub runtime: HubRuntimeState,
    #[serde(default)]
    pub action_history: Vec<HubActionRecord>,
}

impl HubConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, HubError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), HubError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Repairs persisted Hub-owned registries after loading or importing data.
    /// This keeps callers from projecting stale selections, duplicate projects,
    /// orphan action records, or metadata that no longer has a recent project owner.
    pub fn repair_registries(&mut self) -> HubConfigRepairReport {
        let mut report = HubConfigRepairReport::default();
        report.removed_recent_projects = deduplicate_recent_projects(&mut self.recent_projects);
        report.removed_project_metadata =
            prune_unowned_project_metadata(&mut self.project_metadata, &self.recent_projects);
        report.removed_action_history = truncate_action_history(&mut self.action_history);
        report.repaired_active_engine =
            repair_active_engine(&self.engines, &mut self.active_engine_id);
        self.runtime.normalize();
        report
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HubConfigRepairReport {
    pub removed_recent_projects: usize,
    pub removed_project_metadata: usize,
    pub removed_action_history: usize,
    pub repaired_active_engine: bool,
}

impl HubConfigRepairReport {
    pub fn repaired_anything(self) -> bool {
        self.removed_recent_projects > 0
            || self.removed_project_metadata > 0
            || self.removed_action_history > 0
            || self.repaired_active_engine
    }
}

impl Default for HubConfig {
    fn default() -> Self {
        Self {
            settings: HubSettings::default(),
            recent_projects: Vec::new(),
            project_metadata: ProjectMetadataMap::new(),
            engines: Vec::new(),
            active_engine_id: None,
            window: HubWindowState::default(),
            runtime: HubRuntimeState::default(),
            action_history: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubWindowState {
    #[serde(default)]
    pub position_x: Option<i32>,
    #[serde(default)]
    pub position_y: Option<i32>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub maximized: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubRuntimeState {
    #[serde(default)]
    pub selected_page: HubPage,
    #[serde(default)]
    pub project_subpage: ProjectSubpage,
    #[serde(default)]
    pub project_filter: ProjectFilterMode,
    #[serde(default)]
    pub project_sort: ProjectSortMode,
    #[serde(default)]
    pub project_view_mode: ProjectViewMode,
    #[serde(default)]
    pub search_query: String,
    #[serde(default)]
    pub selected_project_path: Option<PathBuf>,
    #[serde(default = "default_selected_template_id")]
    pub selected_template_id: String,
    #[serde(default = "default_project_dir")]
    pub new_project_location: PathBuf,
    #[serde(default)]
    pub new_project_engine_id: Option<String>,
}

impl HubRuntimeState {
    pub fn normalize(&mut self) {
        if self
            .selected_project_path
            .as_ref()
            .is_some_and(|path| path.as_os_str().is_empty())
        {
            self.selected_project_path = None;
        }
        if self.selected_template_id.trim().is_empty() {
            self.selected_template_id = default_selected_template_id();
        }
        if self.new_project_location.as_os_str().is_empty() {
            self.new_project_location = default_project_dir();
        }
        if self
            .new_project_engine_id
            .as_ref()
            .is_some_and(|id| id.trim().is_empty())
        {
            self.new_project_engine_id = None;
        }
    }
}

impl Default for HubRuntimeState {
    fn default() -> Self {
        Self {
            selected_page: HubPage::default(),
            project_subpage: ProjectSubpage::default(),
            project_filter: ProjectFilterMode::default(),
            project_sort: ProjectSortMode::default(),
            project_view_mode: ProjectViewMode::default(),
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: default_selected_template_id(),
            new_project_location: default_project_dir(),
            new_project_engine_id: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubSettings {
    #[serde(default = "default_python_executable")]
    pub python_path: String,
    #[serde(default = "default_cargo_executable")]
    pub cargo_path: String,
    #[serde(default = "default_rustup_executable")]
    pub rustup_path: String,
    #[serde(default = "default_project_dir")]
    pub default_project_dir: PathBuf,
    #[serde(default = "default_source_dir")]
    pub default_source_dir: PathBuf,
    #[serde(default = "default_build_output_dir")]
    pub default_build_output_dir: PathBuf,
    #[serde(default = "default_device_install_dir")]
    pub default_device_install_dir: PathBuf,
    #[serde(default)]
    pub language: HubLanguage,
    #[serde(default)]
    pub build_profile: BuildProfile,
    #[serde(default = "default_jobs")]
    pub jobs: u16,
}

impl Default for HubSettings {
    fn default() -> Self {
        Self {
            python_path: default_python_executable(),
            cargo_path: default_cargo_executable(),
            rustup_path: default_rustup_executable(),
            default_project_dir: default_project_dir(),
            default_source_dir: default_source_dir(),
            default_build_output_dir: default_build_output_dir(),
            default_device_install_dir: default_device_install_dir(),
            language: HubLanguage::default(),
            build_profile: BuildProfile::default(),
            jobs: default_jobs(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HubLanguage {
    #[default]
    English,
    Chinese,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildProfile {
    #[default]
    Debug,
    Release,
}

impl BuildProfile {
    pub fn as_mode(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }

    pub fn from_ui_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "debug" => Some(Self::Debug),
            "release" => Some(Self::Release),
            _ => None,
        }
    }
}

impl HubLanguage {
    pub fn as_ui_value(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Chinese => "Chinese",
        }
    }

    pub fn from_ui_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "english" | "en" => Some(Self::English),
            "chinese" | "zh" | "cn" => Some(Self::Chinese),
            _ => None,
        }
    }
}

fn default_python_executable() -> String {
    "python".to_string()
}

fn default_cargo_executable() -> String {
    "cargo".to_string()
}

fn default_rustup_executable() -> String {
    "rustup".to_string()
}

fn default_jobs() -> u16 {
    1
}

fn default_selected_template_id() -> String {
    ProjectTemplate::RenderableEmpty.id().to_string()
}

fn deduplicate_recent_projects(recent_projects: &mut Vec<RecentProject>) -> usize {
    let original_len = recent_projects.len();
    recent_projects.sort_by(|left, right| {
        right
            .last_opened_unix_ms
            .cmp(&left.last_opened_unix_ms)
            .then_with(|| left.path.cmp(&right.path))
    });
    let mut seen = std::collections::BTreeSet::new();
    recent_projects.retain(|project| seen.insert(project_metadata_key(&project.path)));
    recent_projects.truncate(RECENT_PROJECT_LIMIT);
    original_len.saturating_sub(recent_projects.len())
}

fn prune_unowned_project_metadata(
    metadata: &mut ProjectMetadataMap,
    recent_projects: &[RecentProject],
) -> usize {
    let original_len = metadata.len();
    metadata.retain(|key, _| {
        recent_projects
            .iter()
            .any(|project| project_paths_match(&project.path, key))
    });
    original_len.saturating_sub(metadata.len())
}

fn truncate_action_history(action_history: &mut Vec<HubActionRecord>) -> usize {
    let original_len = action_history.len();
    action_history.sort_by(|left, right| right.finished_unix_ms.cmp(&left.finished_unix_ms));
    action_history.truncate(ACTION_HISTORY_LIMIT);
    original_len.saturating_sub(action_history.len())
}

fn repair_active_engine(
    engines: &[SourceEngineInstall],
    active_engine_id: &mut Option<String>,
) -> bool {
    let before = active_engine_id.clone();
    let active_exists = active_engine_id
        .as_deref()
        .is_some_and(|id| engines.iter().any(|engine| engine.id == id));
    if !active_exists {
        *active_engine_id = engines.first().map(|engine| engine.id.clone());
    }
    *active_engine_id != before
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hub_config_round_trips_settings_and_engines() {
        let mut config = HubConfig::default();
        config.settings.jobs = 4;
        config.settings.default_device_install_dir = PathBuf::from("E:/zircon-device");
        config.project_metadata.insert(
            "e:/projects/game".to_string(),
            crate::projects::ProjectMetadata {
                pinned: true,
                engine_id: Some("local".to_string()),
                last_selected_template: Some("renderable-empty".to_string()),
            },
        );
        config.engines.push(SourceEngineInstall {
            id: "local".to_string(),
            display_name: "Local Source".to_string(),
            source_dir: PathBuf::from("E:/Git/ZirconEngine"),
            output_dir: PathBuf::from("E:/zircon-build"),
            last_build_unix_ms: Some(7),
            build_history: Vec::new(),
        });
        config.active_engine_id = Some("local".to_string());
        config.window.width = Some(1320);
        config.window.height = Some(820);
        config.window.maximized = true;
        config.runtime.selected_page = HubPage::Builds;
        config.runtime.project_subpage = ProjectSubpage::ProjectDetail;
        config.runtime.project_filter = ProjectFilterMode::Existing;
        config.runtime.project_sort = ProjectSortMode::Name;
        config.runtime.project_view_mode = ProjectViewMode::List;
        config.runtime.search_query = "elysium".to_string();
        config.runtime.selected_project_path = Some(PathBuf::from("E:/Projects/Game"));
        config.runtime.selected_template_id = "renderable-empty".to_string();
        config.runtime.new_project_location = PathBuf::from("E:/Drafts");
        config.runtime.new_project_engine_id = Some("local".to_string());
        config.action_history.push(HubActionRecord {
            finished_unix_ms: 9,
            action: crate::state::HubActionKind::OpenEditor,
            status: crate::state::HubActionStatus::Success,
            target: "Game".to_string(),
            detail: "pid 42".to_string(),
            log_excerpt: String::new(),
            recovery: None,
            process_id: Some(42),
            command_line: vec!["zircon_editor".to_string(), "--project".to_string()],
            output_dir: None,
        });

        let encoded = toml::to_string(&config).unwrap();
        let decoded: HubConfig = toml::from_str(&encoded).unwrap();

        assert_eq!(decoded.settings.jobs, 4);
        assert_eq!(
            decoded.settings.default_device_install_dir,
            PathBuf::from("E:/zircon-device")
        );
        assert_eq!(decoded.engines[0].id, "local");
        assert!(decoded.project_metadata["e:/projects/game"].pinned);
        assert_eq!(
            decoded.project_metadata["e:/projects/game"]
                .engine_id
                .as_deref(),
            Some("local")
        );
        assert_eq!(decoded.active_engine_id.as_deref(), Some("local"));
        assert_eq!(decoded.window.width, Some(1320));
        assert!(decoded.window.maximized);
        assert_eq!(decoded.runtime.selected_page, HubPage::Builds);
        assert_eq!(
            decoded.runtime.project_subpage,
            ProjectSubpage::ProjectDetail
        );
        assert_eq!(decoded.runtime.project_filter, ProjectFilterMode::Existing);
        assert_eq!(decoded.runtime.project_sort, ProjectSortMode::Name);
        assert_eq!(decoded.runtime.project_view_mode, ProjectViewMode::List);
        assert_eq!(decoded.runtime.search_query, "elysium");
        assert_eq!(
            decoded.runtime.selected_project_path,
            Some(PathBuf::from("E:/Projects/Game"))
        );
        assert_eq!(
            decoded.runtime.new_project_engine_id.as_deref(),
            Some("local")
        );
        assert_eq!(decoded.action_history[0].process_id, Some(42));
    }

    #[test]
    fn runtime_state_normalizes_empty_persisted_inputs() {
        let mut state = HubRuntimeState {
            selected_project_path: Some(PathBuf::new()),
            selected_template_id: String::new(),
            new_project_location: PathBuf::new(),
            new_project_engine_id: Some(String::new()),
            ..HubRuntimeState::default()
        };

        state.normalize();

        assert!(state.selected_project_path.is_none());
        assert_eq!(
            state.selected_template_id,
            ProjectTemplate::RenderableEmpty.id()
        );
        assert_eq!(state.new_project_location, default_project_dir());
        assert!(state.new_project_engine_id.is_none());
    }

    #[test]
    fn settings_parse_profile_and_language_from_ui_values() {
        assert_eq!(
            BuildProfile::from_ui_value("release"),
            Some(BuildProfile::Release)
        );
        assert_eq!(
            BuildProfile::from_ui_value(" DEBUG "),
            Some(BuildProfile::Debug)
        );
        assert_eq!(BuildProfile::from_ui_value("fast"), None);
        assert_eq!(HubLanguage::from_ui_value("zh"), Some(HubLanguage::Chinese));
        assert_eq!(HubLanguage::English.as_ui_value(), "English");
    }

    #[test]
    fn config_repair_deduplicates_and_prunes_foundation_registries() {
        let mut config = HubConfig::default();
        config.recent_projects = vec![
            RecentProject::new("Old", "E:/Projects/Game", 1),
            RecentProject::new("New", "E:/Projects/Game", 9),
            RecentProject::new("Other", "E:/Projects/Other", 2),
        ];
        config.project_metadata.insert(
            project_metadata_key("E:/Projects/Game"),
            crate::projects::ProjectMetadata {
                pinned: true,
                ..crate::projects::ProjectMetadata::default()
            },
        );
        config.project_metadata.insert(
            project_metadata_key("E:/Projects/Removed"),
            crate::projects::ProjectMetadata {
                pinned: true,
                ..crate::projects::ProjectMetadata::default()
            },
        );
        config.engines.push(SourceEngineInstall {
            id: "local".to_string(),
            display_name: "Local".to_string(),
            source_dir: PathBuf::from("E:/src"),
            output_dir: PathBuf::from("E:/out"),
            last_build_unix_ms: None,
            build_history: Vec::new(),
        });
        config.active_engine_id = Some("missing".to_string());

        let report = config.repair_registries();

        assert!(report.repaired_anything());
        assert_eq!(config.recent_projects.len(), 2);
        assert_eq!(config.recent_projects[0].display_name, "New");
        assert!(config
            .project_metadata
            .contains_key(&project_metadata_key("E:/Projects/Game")));
        assert!(!config
            .project_metadata
            .contains_key(&project_metadata_key("E:/Projects/Removed")));
        assert_eq!(config.active_engine_id.as_deref(), Some("local"));
    }

    #[test]
    fn config_repair_truncates_recent_projects_and_action_history() {
        let mut config = HubConfig::default();
        config.recent_projects = (0..(RECENT_PROJECT_LIMIT + 3))
            .map(|index| {
                RecentProject::new(
                    format!("Project {index}"),
                    format!("E:/Projects/{index}"),
                    index as u64,
                )
            })
            .collect();
        config.action_history = (0..(ACTION_HISTORY_LIMIT + 2))
            .map(|index| HubActionRecord {
                finished_unix_ms: index as u64,
                action: crate::state::HubActionKind::OpenEditor,
                status: crate::state::HubActionStatus::Success,
                target: format!("Project {index}"),
                detail: String::new(),
                log_excerpt: String::new(),
                recovery: None,
                process_id: None,
                command_line: Vec::new(),
                output_dir: None,
            })
            .collect();

        let report = config.repair_registries();

        assert_eq!(config.recent_projects.len(), RECENT_PROJECT_LIMIT);
        assert_eq!(config.recent_projects[0].display_name, "Project 10");
        assert_eq!(config.action_history.len(), ACTION_HISTORY_LIMIT);
        assert_eq!(config.action_history[0].finished_unix_ms, 17);
        assert_eq!(report.removed_recent_projects, 3);
        assert_eq!(report.removed_action_history, 2);
    }

    #[test]
    fn config_repair_preserves_valid_metadata_and_active_engine() {
        let mut config = HubConfig::default();
        config.recent_projects = vec![RecentProject::new("Game", "E:/Projects/Game", 1)];
        config.project_metadata.insert(
            project_metadata_key("E:/Projects/Game"),
            crate::projects::ProjectMetadata {
                pinned: true,
                engine_id: Some("local".to_string()),
                last_selected_template: Some("renderable-empty".to_string()),
            },
        );
        config.engines.push(SourceEngineInstall {
            id: "local".to_string(),
            display_name: "Local".to_string(),
            source_dir: PathBuf::from("E:/src"),
            output_dir: PathBuf::from("E:/out"),
            last_build_unix_ms: None,
            build_history: Vec::new(),
        });
        config.active_engine_id = Some("local".to_string());

        let report = config.repair_registries();

        assert!(!report.repaired_anything());
        let metadata = &config.project_metadata[&project_metadata_key("E:/Projects/Game")];
        assert!(metadata.pinned);
        assert_eq!(metadata.engine_id.as_deref(), Some("local"));
        assert_eq!(
            metadata.last_selected_template.as_deref(),
            Some("renderable-empty")
        );
        assert_eq!(config.active_engine_id.as_deref(), Some("local"));
    }
}
