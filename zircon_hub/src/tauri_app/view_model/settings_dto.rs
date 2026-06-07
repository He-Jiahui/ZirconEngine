use std::{
    env,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::HubError;
use crate::settings::{BuildProfile, HubLanguage, HubSettings};

use super::display::path_text;
use super::localized::HubTextBundle;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubSettingsSummary {
    pub python_path: String,
    pub cargo_path: String,
    pub rustup_path: String,
    pub default_project_dir: String,
    pub default_source_dir: String,
    pub default_build_output_dir: String,
    pub default_device_install_dir: String,
    pub build_profile: String,
    pub build_profile_label: String,
    pub language_label: String,
    pub jobs_label: String,
    pub build_profile_detail: String,
    pub build_workflow_detail: String,
    pub jobs: u16,
    pub language: String,
    pub health: HubSettingsHealthSummary,
    pub text: HubSettingsText,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubSettingsHealthSummary {
    pub label: String,
    pub detail: String,
    pub tone: String,
    pub completion: u8,
    pub rows: Vec<HubSettingsHealthRow>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubSettingsHealthRow {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub meta: String,
    pub state: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubSettingsText {
    pub heading: String,
    pub projects_button: String,
    pub save_button: String,
    pub build_defaults_panel: String,
    pub configuration_paths_panel: String,
    pub source_engines_panel: String,
    pub path_defaults_panel: String,
    pub advanced_configuration_panel: String,
    pub configuration_health_panel: String,
    pub active_source_engine_panel: String,
    pub completeness_label: String,
    pub job_count_singular_template: String,
    pub job_count_plural_template: String,
    pub tabs: Vec<HubSettingsTabText>,
    pub build_profile_options: Vec<HubSettingsOptionText>,
    pub language_options: Vec<HubSettingsOptionText>,
    pub labels: HubSettingsFieldLabels,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubSettingsTabText {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubSettingsOptionText {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubSettingsFieldLabels {
    pub python_path: String,
    pub cargo_path: String,
    pub rustup_path: String,
    pub default_project_dir: String,
    pub default_source_dir: String,
    pub default_build_output_dir: String,
    pub default_device_install_dir: String,
    pub build_profile: String,
    pub jobs: String,
    pub language: String,
    pub release_build: String,
    pub localized_ui: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HubSettingsActionPayload {
    settings: HubSettingsPayload,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubSettingsPayload {
    pub python_path: Option<String>,
    pub cargo_path: Option<String>,
    pub rustup_path: Option<String>,
    pub default_project_dir: Option<String>,
    pub default_source_dir: Option<String>,
    pub default_build_output_dir: Option<String>,
    pub default_device_install_dir: Option<String>,
    pub build_profile: Option<String>,
    pub jobs: Option<u16>,
    pub language: Option<String>,
}

impl HubSettingsPayload {
    pub(crate) fn apply_to(self, settings: &mut HubSettings) -> Result<(), HubError> {
        if let Some(value) = self.python_path {
            settings.python_path = trimmed_required(value, "Python executable")?;
        }
        if let Some(value) = self.cargo_path {
            settings.cargo_path = trimmed_required(value, "Cargo executable")?;
        }
        if let Some(value) = self.rustup_path {
            settings.rustup_path = trimmed_required(value, "Rustup executable")?;
        }
        if let Some(value) = self.default_project_dir {
            settings.default_project_dir = path_from_required(value, "Default project directory")?;
        }
        if let Some(value) = self.default_source_dir {
            settings.default_source_dir = path_from_required(value, "Default source directory")?;
        }
        if let Some(value) = self.default_build_output_dir {
            settings.default_build_output_dir =
                path_from_required(value, "Default build output directory")?;
        }
        if let Some(value) = self.default_device_install_dir {
            settings.default_device_install_dir =
                path_from_required(value, "Default device install directory")?;
        }
        if let Some(value) = self.build_profile {
            settings.build_profile = BuildProfile::from_ui_value(&value)
                .ok_or_else(|| HubError::message(format!("Unknown build profile: {value}")))?;
        }
        if let Some(value) = self.jobs {
            settings.jobs = value.max(1);
        }
        if let Some(value) = self.language {
            settings.language = HubLanguage::from_ui_value(&value)
                .ok_or_else(|| HubError::message(format!("Unknown Hub language: {value}")))?;
        }
        Ok(())
    }
}

pub(crate) fn settings_payload_from_value(
    payload: Option<&Value>,
) -> Result<Option<HubSettingsPayload>, HubError> {
    let Some(payload) = payload else {
        return Ok(None);
    };

    if payload.get("settings").is_some() {
        let payload: HubSettingsActionPayload = serde_json::from_value(payload.clone())?;
        return Ok(Some(payload.settings));
    }

    Ok(Some(serde_json::from_value(payload.clone())?))
}

pub(crate) fn settings_summary(settings: &HubSettings) -> HubSettingsSummary {
    let text = HubSettingsText::for_language(settings.language);
    let build_profile_display =
        build_profile_display_label(settings.build_profile, settings.language);
    let language_label = language_display_label(settings.language);
    let jobs_label = job_count_label(settings.jobs, settings.language);
    HubSettingsSummary {
        python_path: settings.python_path.clone(),
        cargo_path: settings.cargo_path.clone(),
        rustup_path: settings.rustup_path.clone(),
        default_project_dir: path_text(&settings.default_project_dir, settings.language),
        default_source_dir: path_text(&settings.default_source_dir, settings.language),
        default_build_output_dir: path_text(&settings.default_build_output_dir, settings.language),
        default_device_install_dir: path_text(
            &settings.default_device_install_dir,
            settings.language,
        ),
        build_profile: build_profile_label(settings.build_profile).to_string(),
        build_profile_label: build_profile_display.clone(),
        language_label,
        jobs_label: jobs_label.clone(),
        build_profile_detail: build_profile_detail(
            &build_profile_display,
            &jobs_label,
            settings.language,
        ),
        build_workflow_detail: build_workflow_detail(&build_profile_display, settings.language),
        jobs: settings.jobs,
        language: settings.language.as_ui_value().to_string(),
        health: settings_health(settings),
        text,
    }
}

impl HubSettingsText {
    fn for_language(language: HubLanguage) -> Self {
        let text = HubTextBundle::new(language);
        Self {
            heading: text
                .pair(
                    "Toolchain, Build Defaults & Paths",
                    "工具链、构建默认值与路径",
                )
                .to_string(),
            projects_button: text.pair("Projects", "项目").to_string(),
            save_button: text.pair("Save Changes", "保存更改").to_string(),
            build_defaults_panel: text.pair("Build Defaults", "构建默认值").to_string(),
            configuration_paths_panel: text.pair("Configuration Paths", "配置路径").to_string(),
            source_engines_panel: text.pair("Source Engines", "源码引擎").to_string(),
            path_defaults_panel: text.pair("Path Defaults", "路径默认值").to_string(),
            advanced_configuration_panel: text
                .pair("Advanced Configuration", "高级配置")
                .to_string(),
            configuration_health_panel: text
                .pair("Configuration Health", "配置健康状态")
                .to_string(),
            active_source_engine_panel: text
                .pair("Active Source Engine", "当前源码引擎")
                .to_string(),
            completeness_label: text.pair("Completeness", "完整度").to_string(),
            job_count_singular_template: text.pair("{jobs} job", "{jobs} 任务").to_string(),
            job_count_plural_template: text.pair("{jobs} jobs", "{jobs} 任务").to_string(),
            tabs: vec![
                tab("overview", text.pair("Overview", "概览")),
                tab("toolchain", text.pair("Toolchain", "工具链")),
                tab("paths", text.pair("Paths", "路径")),
                tab("advanced", text.pair("Advanced", "高级")),
            ],
            build_profile_options: vec![
                option("debug", text.pair("Debug", "Debug")),
                option("release", text.pair("Release", "Release")),
            ],
            language_options: vec![
                option("Chinese", text.pair("Chinese", "中文")),
                option("English", text.pair("English", "English")),
            ],
            labels: HubSettingsFieldLabels {
                python_path: text
                    .pair("Python Executable", "Python 可执行文件")
                    .to_string(),
                cargo_path: text
                    .pair("Cargo Executable", "Cargo 可执行文件")
                    .to_string(),
                rustup_path: text
                    .pair("Rustup Executable", "Rustup 可执行文件")
                    .to_string(),
                default_project_dir: text
                    .pair("Default Project Directory", "默认项目目录")
                    .to_string(),
                default_source_dir: text
                    .pair("Default Source Directory", "默认源码目录")
                    .to_string(),
                default_build_output_dir: text
                    .pair("Default Build Output Directory", "默认构建输出目录")
                    .to_string(),
                default_device_install_dir: text
                    .pair("Default Device Install Directory", "默认设备安装目录")
                    .to_string(),
                build_profile: text.pair("Build Profile", "构建配置").to_string(),
                jobs: text.pair("Parallel Jobs", "并行任务数").to_string(),
                language: text.pair("Language", "语言").to_string(),
                release_build: text.pair("Release Build", "Release 构建").to_string(),
                localized_ui: text.pair("Localized UI", "本地化界面").to_string(),
            },
        }
    }
}

fn settings_health(settings: &HubSettings) -> HubSettingsHealthSummary {
    let text = HubTextBundle::new(settings.language);
    let rows = vec![
        executable_row(
            "python-path",
            text.pair("Python", "Python"),
            &settings.python_path,
            settings.language,
            true,
        ),
        executable_row(
            "cargo-path",
            text.pair("Cargo", "Cargo"),
            &settings.cargo_path,
            settings.language,
            false,
        ),
        executable_row(
            "rustup-path",
            text.pair("Rustup", "Rustup"),
            &settings.rustup_path,
            settings.language,
            false,
        ),
        directory_row(
            "project-dir",
            text.pair("Project Directory", "项目目录"),
            &settings.default_project_dir,
            settings.language,
            false,
        ),
        directory_row(
            "source-dir",
            text.pair("Source Checkout", "源码检出目录"),
            &settings.default_source_dir,
            settings.language,
            false,
        ),
        directory_row(
            "build-output",
            text.pair("Build Output", "构建输出"),
            &settings.default_build_output_dir,
            settings.language,
            false,
        ),
        directory_row(
            "device-install",
            text.pair("Device Install", "设备安装"),
            &settings.default_device_install_dir,
            settings.language,
            false,
        ),
    ];
    let ready_count = rows
        .iter()
        .filter(|row| row.state == "ok" || row.state == "warn")
        .count();
    let completion = ((ready_count * 100) / rows.len()).min(100) as u8;
    let has_error = rows.iter().any(|row| row.state == "error");
    HubSettingsHealthSummary {
        label: if has_error {
            text.pair("Needs Attention", "需要处理")
        } else {
            text.pair("Ready", "就绪")
        }
        .to_string(),
        detail: if has_error {
            text.pair(
                "Repair required settings before running workflows",
                "运行工作流前需要修复配置",
            )
        } else {
            text.pair(
                "Configuration is available for local workflows",
                "配置可用于本地工作流",
            )
        }
        .to_string(),
        tone: if has_error { "warning" } else { "success" }.to_string(),
        completion,
        rows,
    }
}

fn executable_row(
    id: &str,
    title: &str,
    value: &str,
    language: HubLanguage,
    selected: bool,
) -> HubSettingsHealthRow {
    let text = HubTextBundle::new(language);
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return health_row(
            id,
            title,
            text.pair("Not configured", "未配置"),
            text.pair("Required", "必需"),
            "error",
            selected,
        );
    }
    let looks_like_path = trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains(':');
    if looks_like_path && !Path::new(trimmed).exists() {
        return health_row(
            id,
            title,
            trimmed,
            text.pair("Missing", "缺失"),
            "error",
            selected,
        );
    }
    if !looks_like_path && !path_command_exists(trimmed) {
        return health_row(
            id,
            title,
            trimmed,
            text.pair("Missing", "缺失"),
            "error",
            selected,
        );
    }
    health_row(
        id,
        title,
        trimmed,
        if looks_like_path {
            text.pair("Available", "可用")
        } else {
            text.pair("PATH command", "PATH 命令")
        },
        "ok",
        selected,
    )
}

fn path_command_exists(command: &str) -> bool {
    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };
    let has_extension = Path::new(command).extension().is_some();
    let extensions = path_command_extensions(has_extension);

    env::split_paths(&path_var).any(|dir| {
        extensions
            .iter()
            .map(|extension| dir.join(format!("{command}{extension}")))
            .any(|candidate| candidate.is_file())
    })
}

fn path_command_extensions(has_extension: bool) -> Vec<String> {
    if has_extension {
        return vec![String::new()];
    }

    #[cfg(windows)]
    {
        env::var_os("PATHEXT")
            .and_then(|value| value.into_string().ok())
            .map(|value| {
                value
                    .split(';')
                    .map(str::trim)
                    .filter(|extension| !extension.is_empty())
                    .map(|extension| {
                        if extension.starts_with('.') {
                            extension.to_string()
                        } else {
                            format!(".{extension}")
                        }
                    })
                    .chain(std::iter::once(String::new()))
                    .collect()
            })
            .unwrap_or_else(|| {
                [".COM", ".EXE", ".BAT", ".CMD", ""]
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            })
    }

    #[cfg(not(windows))]
    {
        vec![String::new()]
    }
}

fn directory_row(
    id: &str,
    title: &str,
    path: &Path,
    language: HubLanguage,
    selected: bool,
) -> HubSettingsHealthRow {
    let text = HubTextBundle::new(language);
    if path.as_os_str().is_empty() {
        return health_row(
            id,
            title,
            text.pair("Not configured", "未配置"),
            text.pair("Required", "必需"),
            "error",
            selected,
        );
    }
    let detail = path_text(path, language);
    if path.is_dir() {
        health_row(
            id,
            title,
            &detail,
            text.pair("Available", "可用"),
            "ok",
            selected,
        )
    } else {
        health_row(
            id,
            title,
            &detail,
            text.pair("Created on use", "使用时创建"),
            "warn",
            selected,
        )
    }
}

fn health_row(
    id: &str,
    title: &str,
    detail: &str,
    meta: &str,
    state: &str,
    selected: bool,
) -> HubSettingsHealthRow {
    HubSettingsHealthRow {
        id: id.to_string(),
        title: title.to_string(),
        detail: detail.to_string(),
        meta: meta.to_string(),
        state: state.to_string(),
        selected,
    }
}

fn tab(value: &str, label: &str) -> HubSettingsTabText {
    HubSettingsTabText {
        value: value.to_string(),
        label: label.to_string(),
    }
}

fn option(value: &str, label: &str) -> HubSettingsOptionText {
    HubSettingsOptionText {
        value: value.to_string(),
        label: label.to_string(),
    }
}

fn trimmed_required(value: String, label: &str) -> Result<String, HubError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(HubError::message(format!("{label} is required")));
    }
    Ok(trimmed.to_string())
}

fn path_from_required(value: String, label: &str) -> Result<PathBuf, HubError> {
    Ok(PathBuf::from(trimmed_required(value, label)?))
}

fn build_profile_label(profile: BuildProfile) -> &'static str {
    match profile {
        BuildProfile::Debug => "debug",
        BuildProfile::Release => "release",
    }
}

fn build_profile_display_label(profile: BuildProfile, language: HubLanguage) -> String {
    let text = HubTextBundle::new(language);
    match profile {
        BuildProfile::Debug => text.pair("Debug", "Debug"),
        BuildProfile::Release => text.pair("Release", "Release"),
    }
    .to_string()
}

fn language_display_label(language: HubLanguage) -> String {
    let text = HubTextBundle::new(language);
    match language {
        HubLanguage::Chinese => text.pair("Chinese", "中文"),
        HubLanguage::English => text.pair("English", "English"),
    }
    .to_string()
}

fn job_count_label(jobs: u16, language: HubLanguage) -> String {
    match language {
        HubLanguage::English if jobs == 1 => "1 job".to_string(),
        HubLanguage::English => format!("{jobs} jobs"),
        HubLanguage::Chinese => format!("{jobs} 任务"),
    }
}

fn build_profile_detail(
    build_profile_label: &str,
    jobs_label: &str,
    language: HubLanguage,
) -> String {
    match language {
        HubLanguage::English | HubLanguage::Chinese => {
            format!("{build_profile_label} / {jobs_label}")
        }
    }
}

fn build_workflow_detail(build_profile_label: &str, language: HubLanguage) -> String {
    let text = HubTextBundle::new(language);
    match language {
        HubLanguage::English => format!(
            "{}: {build_profile_label}",
            text.pair(
                "Compile editor/runtime targets with configured build defaults",
                "使用当前构建默认值编译编辑器/运行时目标",
            )
        ),
        HubLanguage::Chinese => format!(
            "{}：{build_profile_label}",
            text.pair(
                "Compile editor/runtime targets with configured build defaults",
                "使用当前构建默认值编译编辑器/运行时目标",
            )
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_payload_accepts_wrapped_payload_and_updates_config() {
        let value = serde_json::json!({
            "settings": {
                "pythonPath": "py",
                "cargoPath": "cargo",
                "rustupPath": "rustup",
                "defaultProjectDir": "E:/Projects",
                "defaultSourceDir": "E:/Source",
                "defaultBuildOutputDir": "E:/Builds",
                "defaultDeviceInstallDir": "E:/Device",
                "buildProfile": "release",
                "jobs": 0,
                "language": "zh"
            }
        });
        let payload = settings_payload_from_value(Some(&value))
            .unwrap()
            .expect("settings payload should parse");
        let mut settings = HubSettings::default();

        payload.apply_to(&mut settings).unwrap();

        assert_eq!(settings.python_path, "py");
        assert_eq!(settings.build_profile, BuildProfile::Release);
        assert_eq!(settings.jobs, 1);
        assert_eq!(settings.language, HubLanguage::Chinese);
        assert_eq!(settings.default_project_dir, PathBuf::from("E:/Projects"));
    }

    #[test]
    fn settings_summary_defaults_to_chinese_text() {
        let settings = HubSettings::default();

        let summary = settings_summary(&settings);

        assert_eq!(summary.language, "Chinese");
        assert_eq!(summary.text.heading, "工具链、构建默认值与路径");
        assert_eq!(summary.text.language_options[0].label, "中文");
        assert_eq!(summary.text.job_count_plural_template, "{jobs} 任务");
    }

    #[test]
    fn settings_summary_projects_saved_option_labels_for_react_consumers() {
        let mut settings = HubSettings {
            jobs: 3,
            ..HubSettings::default()
        };
        settings.language = HubLanguage::Chinese;
        settings.build_profile = BuildProfile::Release;

        let summary = settings_summary(&settings);

        assert_eq!(summary.build_profile, "release");
        assert_eq!(summary.language, "Chinese");
        assert_eq!(summary.build_profile_label, "Release");
        assert_eq!(summary.language_label, "中文");
        assert_eq!(summary.jobs_label, "3 任务");
        assert_eq!(summary.build_profile_detail, "Release / 3 任务");
        assert_eq!(
            summary.build_workflow_detail,
            "使用当前构建默认值编译编辑器/运行时目标：Release"
        );
    }

    #[test]
    fn settings_health_includes_rustup_path_status() {
        let mut settings = HubSettings::default();
        let missing_rustup = std::env::temp_dir().join(format!(
            "zircon-hub-missing-rustup-{}-{}",
            std::process::id(),
            crate::projects::now_unix_ms()
        ));
        settings.rustup_path = missing_rustup.to_string_lossy().into_owned();

        let summary = settings_summary(&settings);
        let rustup_row = summary
            .health
            .rows
            .iter()
            .find(|row| row.id == "rustup-path")
            .expect("Rustup should participate in Settings health");

        assert_eq!(rustup_row.title, "Rustup");
        assert_eq!(rustup_row.state, "error");
        assert_eq!(rustup_row.meta, "缺失");
        assert_eq!(summary.health.label, "需要处理");
    }

    #[test]
    fn settings_health_checks_path_command_availability() {
        let mut settings = HubSettings::default();
        settings.python_path = format!(
            "zircon-hub-missing-python-command-{}-{}",
            std::process::id(),
            crate::projects::now_unix_ms()
        );

        let summary = settings_summary(&settings);
        let python_row = summary
            .health
            .rows
            .iter()
            .find(|row| row.id == "python-path")
            .expect("Python should participate in Settings health");

        assert_eq!(python_row.title, "Python");
        assert_eq!(python_row.state, "error");
        assert_eq!(python_row.meta, "缺失");
        assert_eq!(summary.health.label, "需要处理");
    }
}
