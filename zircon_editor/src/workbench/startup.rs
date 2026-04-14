use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use zircon_asset::ProjectPaths;

use super::project::EditorProjectDocument;

pub(crate) const STARTUP_SESSION_KEY: &str = "editor.startup.session";
pub(crate) const WELCOME_DESCRIPTOR_ID: &str = "editor.welcome";
pub(crate) const WELCOME_INSTANCE_ID: &str = "editor.welcome#1";
pub(crate) const WELCOME_PAGE_ID: &str = "page:welcome";
pub(crate) const RECENT_PROJECT_LIMIT: usize = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorSessionMode {
    #[default]
    Welcome,
    Project,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecentProjectValidation {
    #[default]
    Valid,
    Missing,
    InvalidManifest,
    InvalidProject,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProjectEntry {
    pub display_name: String,
    pub path: String,
    pub last_opened_unix_ms: u64,
    #[serde(default)]
    pub validation: RecentProjectValidation,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewProjectTemplate {
    #[default]
    RenderableEmpty,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewProjectDraft {
    pub project_name: String,
    pub location: String,
    pub template: NewProjectTemplate,
}

impl NewProjectDraft {
    pub fn renderable_empty_default() -> Self {
        Self {
            project_name: "ZirconProject".to_string(),
            location: default_project_location()
                .to_string_lossy()
                .into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        }
    }

    pub fn project_root(&self) -> Result<PathBuf, String> {
        let project_name = self.project_name.trim();
        if project_name.is_empty() {
            return Err("Project name is required".to_string());
        }

        let location = self.location.trim();
        if location.is_empty() {
            return Err("Location is required".to_string());
        }

        let location = PathBuf::from(location);
        let root = if location.is_absolute() {
            location.join(project_name)
        } else {
            std::env::current_dir()
                .map_err(|error| error.to_string())?
                .join(location)
                .join(project_name)
        };
        Ok(root)
    }

    pub fn validate_for_creation(&self) -> Result<PathBuf, String> {
        let root = self.project_root()?;
        if root.exists() {
            if root.is_file() {
                return Err("Target path already exists as a file".to_string());
            }
            let mut entries = root.read_dir().map_err(|error| error.to_string())?;
            if entries.next().transpose().map_err(|error| error.to_string())?.is_some() {
                return Err("Target directory must be empty".to_string());
            }
        }
        Ok(root)
    }

    pub fn validate_for_open_existing(&self) -> Result<PathBuf, String> {
        let root = self.project_root()?;
        if !root.exists() {
            return Err("Project directory does not exist".to_string());
        }
        let paths = ProjectPaths::from_root(&root).map_err(|error| error.to_string())?;
        if !paths.manifest_path().exists() {
            return Err("zircon-project.toml not found in target directory".to_string());
        }
        Ok(root)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecentProjectItemSnapshot {
    pub display_name: String,
    pub path: String,
    pub validation: RecentProjectValidation,
    pub last_opened_label: String,
    pub selected: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewProjectFormSnapshot {
    pub project_name: String,
    pub location: String,
    pub project_path_preview: String,
    pub template_label: String,
    pub can_create: bool,
    pub can_open_existing: bool,
    pub validation_message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WelcomePaneSnapshot {
    pub title: String,
    pub subtitle: String,
    pub status_message: String,
    pub browse_supported: bool,
    pub recent_projects: Vec<RecentProjectItemSnapshot>,
    pub form: NewProjectFormSnapshot,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EditorStartupSessionDocument {
    pub mode: EditorSessionMode,
    pub project: Option<EditorProjectDocument>,
    pub recent_projects: Vec<RecentProjectEntry>,
    pub draft: NewProjectDraft,
    pub status_message: String,
}

impl Default for EditorStartupSessionDocument {
    fn default() -> Self {
        Self {
            mode: EditorSessionMode::Welcome,
            project: None,
            recent_projects: Vec::new(),
            draft: NewProjectDraft::renderable_empty_default(),
            status_message: "Open an existing project or create a renderable empty project."
                .to_string(),
        }
    }
}

impl EditorStartupSessionDocument {
    pub fn welcome_pane_snapshot(&self, browse_supported: bool) -> WelcomePaneSnapshot {
        let project_path_preview = self
            .draft
            .project_root()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default();
        let creation_validation = self
            .draft
            .validate_for_creation()
            .map(|_| String::new())
            .unwrap_or_else(|error| error);
        let can_open_existing = self.draft.validate_for_open_existing().is_ok();

        WelcomePaneSnapshot {
            title: "Open or Create".to_string(),
            subtitle: "Continue from a recent project or scaffold a renderable empty project."
                .to_string(),
            status_message: self.status_message.clone(),
            browse_supported,
            recent_projects: self
                .recent_projects
                .iter()
                .enumerate()
                .map(|(index, entry)| RecentProjectItemSnapshot {
                    display_name: entry.display_name.clone(),
                    path: entry.path.clone(),
                    validation: entry.validation,
                    last_opened_label: format_recent_project_time(entry.last_opened_unix_ms),
                    selected: index == 0,
                })
                .collect(),
            form: NewProjectFormSnapshot {
                project_name: self.draft.project_name.clone(),
                location: self.draft.location.clone(),
                project_path_preview,
                template_label: "Renderable Empty".to_string(),
                can_create: creation_validation.is_empty(),
                can_open_existing,
                validation_message: creation_validation,
            },
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StoredStartupSession {
    pub last_project_path: Option<String>,
    #[serde(default)]
    pub recent_projects: Vec<StoredRecentProjectEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StoredRecentProjectEntry {
    pub display_name: String,
    pub path: String,
    pub last_opened_unix_ms: u64,
}

impl StoredStartupSession {
    pub fn recent_projects_with_validation<F>(&self, mut validate: F) -> Vec<RecentProjectEntry>
    where
        F: FnMut(&str) -> RecentProjectValidation,
    {
        self.recent_projects
            .iter()
            .map(|entry| RecentProjectEntry {
                display_name: entry.display_name.clone(),
                path: entry.path.clone(),
                last_opened_unix_ms: entry.last_opened_unix_ms,
                validation: validate(&entry.path),
            })
            .collect()
    }

    pub fn update_recent_project(&mut self, path: &str, display_name: &str, now_unix_ms: u64) {
        self.last_project_path = Some(path.to_string());
        self.recent_projects.retain(|entry| entry.path != path);
        self.recent_projects.insert(
            0,
            StoredRecentProjectEntry {
                display_name: display_name.to_string(),
                path: path.to_string(),
                last_opened_unix_ms: now_unix_ms,
            },
        );
        self.recent_projects.sort_by(|left, right| {
            right
                .last_opened_unix_ms
                .cmp(&left.last_opened_unix_ms)
                .then_with(|| left.path.cmp(&right.path))
        });
        self.recent_projects.truncate(RECENT_PROJECT_LIMIT);
    }

    pub fn forget_recent_project(&mut self, path: &str) {
        self.recent_projects.retain(|entry| entry.path != path);
        if self.last_project_path.as_deref() == Some(path) {
            self.last_project_path = self.recent_projects.first().map(|entry| entry.path.clone());
        }
    }
}

pub(crate) fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

pub(crate) fn format_recent_project_time(last_opened_unix_ms: u64) -> String {
    if last_opened_unix_ms == 0 {
        return "Unknown".to_string();
    }
    let now = now_unix_ms();
    let delta_ms = now.saturating_sub(last_opened_unix_ms);
    let delta = Duration::from_millis(delta_ms);
    if delta < Duration::from_secs(60) {
        "Just now".to_string()
    } else if delta < Duration::from_secs(60 * 60) {
        format!("{}m ago", delta.as_secs() / 60)
    } else if delta < Duration::from_secs(60 * 60 * 24) {
        format!("{}h ago", delta.as_secs() / (60 * 60))
    } else {
        format!("{}d ago", delta.as_secs() / (60 * 60 * 24))
    }
}

pub(crate) fn default_project_location() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(home) = std::env::var_os("USERPROFILE") {
            return PathBuf::from(home).join("Documents").join("ZirconProjects");
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("ZirconProjects");
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
