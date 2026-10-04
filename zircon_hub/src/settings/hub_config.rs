use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engines::SourceEngineInstall;
use crate::error::HubError;
use crate::projects::{
    project_metadata_key, project_paths_match, CloudProjectBinding, ProjectMetadataMap,
    ProjectTemplateId, RecentProject, RECENT_PROJECT_LIMIT,
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
    pub cloud_bindings: Vec<CloudProjectBinding>,
    #[serde(default)]
    pub(crate) last_seen_shared_recent_revision: Option<u64>,
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
        let mut config: Self = toml::from_str(&text)?;
        for project in &mut config.recent_projects {
            if project.path.join("zircon-project.toml").is_file() {
                project.refresh_summary()?;
            }
        }
        Ok(config)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), HubError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        write_atomic(path, toml::to_string_pretty(self)?.as_bytes())
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

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), HubError> {
    let tmp_path = path.with_extension(format!(
        "{}tmp",
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| format!("{extension}."))
            .unwrap_or_default()
    ));
    fs::write(&tmp_path, bytes)?;
    replace_file(&tmp_path, path)
}

fn replace_file(tmp_path: &Path, target_path: &Path) -> Result<(), HubError> {
    match fs::rename(tmp_path, target_path) {
        Ok(()) => Ok(()),
        Err(first_error) if target_path.exists() => {
            match replace_existing_file(tmp_path, target_path) {
                Ok(()) => Ok(()),
                Err(replace_error) => {
                    let _ = fs::remove_file(tmp_path);
                    Err(HubError::message(format!(
                    "Failed to replace Hub config: {first_error}; atomic replace failed: {replace_error}"
                )))
                }
            }
        }
        Err(error) => {
            let _ = fs::remove_file(tmp_path);
            Err(error.into())
        }
    }
}

#[cfg(not(windows))]
fn replace_existing_file(tmp_path: &Path, target_path: &Path) -> std::io::Result<()> {
    fs::rename(tmp_path, target_path)
}

#[cfg(windows)]
fn replace_existing_file(tmp_path: &Path, target_path: &Path) -> std::io::Result<()> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn ReplaceFileW(
            replaced_file_name: *const u16,
            replacement_file_name: *const u16,
            backup_file_name: *const u16,
            replace_flags: u32,
            exclude: *const c_void,
            reserved: *const c_void,
        ) -> i32;
    }
    const REPLACEFILE_WRITE_THROUGH: u32 = 1;

    fn wide_path(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    let target = wide_path(target_path);
    let replacement = wide_path(tmp_path);
    // SAFETY: both paths are NUL-terminated for the duration of the synchronous call.
    let replaced = unsafe {
        ReplaceFileW(
            target.as_ptr(),
            replacement.as_ptr(),
            std::ptr::null(),
            REPLACEFILE_WRITE_THROUGH,
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    if replaced == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
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
            cloud_bindings: Vec::new(),
            last_seen_shared_recent_revision: None,
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
    #[serde(default)]
    pub new_project_name: String,
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
        self.new_project_name = self.new_project_name.trim().to_string();
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
            new_project_name: String::new(),
            selected_template_id: default_selected_template_id(),
            new_project_location: default_project_dir(),
            new_project_engine_id: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum OutputRootProvenance {
    #[default]
    Unverified,
    HubManaged,
    NativeFolderPicker,
}

impl OutputRootProvenance {
    pub(crate) fn grants_open_capability(self) -> bool {
        matches!(self, Self::HubManaged | Self::NativeFolderPicker)
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
    #[serde(default)]
    pub(crate) default_build_output_provenance: OutputRootProvenance,
    #[serde(default = "default_device_install_dir")]
    pub default_device_install_dir: PathBuf,
    #[serde(default)]
    pub(crate) default_device_install_provenance: OutputRootProvenance,
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
            default_build_output_provenance: OutputRootProvenance::HubManaged,
            default_device_install_dir: default_device_install_dir(),
            default_device_install_provenance: OutputRootProvenance::HubManaged,
            language: HubLanguage::default(),
            build_profile: BuildProfile::default(),
            jobs: default_jobs(),
        }
    }
}

impl HubSettings {
    pub(crate) fn default_build_output_grants_open_capability(&self) -> bool {
        self.default_build_output_provenance
            .grants_open_capability()
    }

    pub(crate) fn default_device_install_grants_open_capability(&self) -> bool {
        self.default_device_install_provenance
            .grants_open_capability()
    }

    pub(crate) fn set_default_build_output_from_webview(&mut self, path: PathBuf) {
        self.default_build_output_dir = path;
        self.default_build_output_provenance = OutputRootProvenance::Unverified;
    }

    pub(crate) fn set_default_device_install_from_webview(&mut self, path: PathBuf) {
        self.default_device_install_dir = path;
        self.default_device_install_provenance = OutputRootProvenance::Unverified;
    }

    pub(crate) fn set_default_build_output_from_native_folder_picker(&mut self, path: PathBuf) {
        self.default_build_output_dir = path;
        self.default_build_output_provenance = OutputRootProvenance::NativeFolderPicker;
    }

    pub(crate) fn set_default_device_install_from_native_folder_picker(&mut self, path: PathBuf) {
        self.default_device_install_dir = path;
        self.default_device_install_provenance = OutputRootProvenance::NativeFolderPicker;
    }

    pub(crate) fn set_default_build_output_from_registered_engine(&mut self, path: PathBuf) {
        if self.default_build_output_dir != path {
            self.set_default_build_output_from_webview(path);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HubLanguage {
    English,
    #[default]
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
        let value = value.trim();
        if value.eq_ignore_ascii_case("debug") {
            Some(Self::Debug)
        } else if value.eq_ignore_ascii_case("release") {
            Some(Self::Release)
        } else {
            None
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
        let value = value.trim();
        if value.eq_ignore_ascii_case("english") || value.eq_ignore_ascii_case("en") {
            Some(Self::English)
        } else if value.eq_ignore_ascii_case("chinese")
            || value.eq_ignore_ascii_case("zh")
            || value.eq_ignore_ascii_case("cn")
        {
            Some(Self::Chinese)
        } else {
            None
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
    ProjectTemplateId::RenderableEmpty.as_str().to_string()
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
#[path = "tests/hub_config.rs"]
mod tests;
