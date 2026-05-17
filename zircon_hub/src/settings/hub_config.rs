use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engines::SourceEngineInstall;
use crate::error::HubError;
use crate::projects::RecentProject;

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
    pub engines: Vec<SourceEngineInstall>,
    #[serde(default)]
    pub active_engine_id: Option<String>,
    #[serde(default)]
    pub window: HubWindowState,
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
}

impl Default for HubConfig {
    fn default() -> Self {
        Self {
            settings: HubSettings::default(),
            recent_projects: Vec::new(),
            engines: Vec::new(),
            active_engine_id: None,
            window: HubWindowState::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hub_config_round_trips_settings_and_engines() {
        let mut config = HubConfig::default();
        config.settings.jobs = 4;
        config.settings.default_device_install_dir = PathBuf::from("E:/zircon-device");
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

        let encoded = toml::to_string(&config).unwrap();
        let decoded: HubConfig = toml::from_str(&encoded).unwrap();

        assert_eq!(decoded.settings.jobs, 4);
        assert_eq!(
            decoded.settings.default_device_install_dir,
            PathBuf::from("E:/zircon-device")
        );
        assert_eq!(decoded.engines[0].id, "local");
        assert_eq!(decoded.active_engine_id.as_deref(), Some("local"));
        assert_eq!(decoded.window.width, Some(1320));
        assert!(decoded.window.maximized);
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
}
