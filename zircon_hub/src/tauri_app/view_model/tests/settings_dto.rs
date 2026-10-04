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
    let payload: HubSettingsActionPayload =
        serde_json::from_value(value).expect("settings payload should parse");
    let mut settings = HubSettings::default();

    payload.settings.apply_to(&mut settings).unwrap();

    assert_eq!(settings.python_path, "py");
    assert_eq!(settings.build_profile, BuildProfile::Release);
    assert_eq!(settings.jobs, 1);
    assert_eq!(settings.language, HubLanguage::Chinese);
    assert_eq!(settings.default_project_dir, PathBuf::from("E:/Projects"));
    assert!(
        !settings.default_build_output_grants_open_capability(),
        "a WebView-supplied output path must not retain a shell capability"
    );
    assert!(
        !settings.default_device_install_grants_open_capability(),
        "a WebView-supplied device path must not retain a shell capability"
    );
}

#[test]
fn settings_payload_rejects_output_provenance_injection() {
    let error = serde_json::from_value::<HubSettingsPayload>(serde_json::json!({
        "defaultBuildOutputProvenance": "native-folder-picker"
    }))
    .expect_err("WebView clients must not be able to set output provenance");

    assert!(error.to_string().contains("defaultBuildOutputProvenance"));
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
fn settings_language_options_keep_native_names_across_ui_languages() {
    let mut settings = HubSettings {
        language: HubLanguage::English,
        ..HubSettings::default()
    };

    let english_summary = settings_summary(&settings);

    assert_eq!(english_summary.text.language_options[0].value, "Chinese");
    assert_eq!(english_summary.text.language_options[0].label, "中文");
    assert_eq!(english_summary.text.language_options[1].value, "English");
    assert_eq!(english_summary.text.language_options[1].label, "English");

    settings.language = HubLanguage::Chinese;
    let chinese_summary = settings_summary(&settings);

    assert_eq!(chinese_summary.text.language_options[0].value, "Chinese");
    assert_eq!(chinese_summary.text.language_options[0].label, "中文");
    assert_eq!(chinese_summary.text.language_options[1].value, "English");
    assert_eq!(chinese_summary.text.language_options[1].label, "English");
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
