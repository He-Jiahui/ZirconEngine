use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    default_build_output_dir, default_device_install_dir, default_project_dir, default_source_dir,
};

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

pub(super) fn migrate_legacy_build_output_default(
    settings: &mut HubSettings,
    home: Option<PathBuf>,
) {
    if settings.default_build_output_provenance == OutputRootProvenance::NativeFolderPicker {
        return;
    }
    let Some(home) = home else {
        return;
    };
    let legacy_default = home.join("ZirconBuilds");
    if normalize_windows_path(&settings.default_build_output_dir)
        != normalize_windows_path(&legacy_default)
    {
        return;
    }

    settings.default_build_output_dir = default_build_output_dir();
    settings.default_build_output_provenance = OutputRootProvenance::HubManaged;
}

fn normalize_windows_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
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

#[cfg(test)]
#[path = "tests/core.rs"]
mod tests;
