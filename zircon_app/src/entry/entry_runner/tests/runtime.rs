use std::num::NonZeroU64;

use super::*;

#[test]
fn f0_runtime_project_fixture_roots_follow_the_resolved_test_binary_directory() {
    let root = runtime_mvp_fixture_root("physical-root");
    let executable = std::env::current_exe().expect("locate the F0 runtime test executable");
    let binary_directory = executable
        .parent()
        .expect("F0 runtime test executable must have a parent directory");
    let resolved_binary_directory =
        ProjectPaths::resolve_existing(binary_directory).expect("resolve F0 test binary directory");

    assert!(
        root.starts_with(resolved_binary_directory.operation_path()),
        "F0 runtime fixture output must retain the test binary's physical output root"
    );
}

fn runtime_mvp_fixture_root(label: impl AsRef<str>) -> PathBuf {
    let executable = std::env::current_exe().expect("locate the F0 runtime test executable");
    let binary_directory = executable
        .parent()
        .expect("F0 runtime test executable must have a parent directory");
    let binary_directory = ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve the F0 runtime test binary directory");

    binary_directory
        .operation_path()
        .join("zircon-mvp-fixtures")
        .join(label.as_ref())
}

#[test]
fn play_startup_report_records_typed_ordered_phases_without_newlines() {
    assert_eq!(
        play_startup_report_record(
            "zircon-play-report-42",
            PlayStartupReportPhase::Starting,
            "profile=runtime\nscene=.zircon/play/42/play-scene.zrscene.json",
        ),
        "zircon_play_report outlet=zircon-play-report-42 phase=starting detail=profile=runtime scene=.zircon/play/42/play-scene.zrscene.json\n"
    );
    assert_eq!(PlayStartupReportPhase::Ready.as_str(), "ready");
    assert_eq!(PlayStartupReportPhase::StartFailed.as_str(), "start-failed");
    assert_eq!(PlayStartupReportPhase::Terminal.as_str(), "terminal");
}

#[test]
fn play_startup_reporting_without_an_outlet_is_a_successful_no_op() {
    assert!(report_play_startup(
        None,
        PlayStartupReportPhase::Starting,
        "profile=runtime scene=<default>",
    )
    .is_ok());
}

#[test]
fn runtime_session_profile_selects_default_game_host_config() {
    let config = runtime_entry_app_config_for_session_profile(RuntimeSessionProfile::Runtime);

    assert!(config.window_descriptor().primary_window.is_some());
    assert_eq!(config.event_loop_policy(), EventLoopPolicy::Game);
    assert!(config
        .window_lifecycle_policy()
        .should_exit_after_primary_close());
}

#[test]
fn pipelined_runtime_profile_keeps_the_game_host_policy() {
    let config =
        runtime_entry_app_config_for_session_profile(RuntimeSessionProfile::RuntimePipelined);

    assert!(config.window_descriptor().primary_window.is_some());
    assert_eq!(config.event_loop_policy(), EventLoopPolicy::Game);
}

#[test]
fn editor_and_dev_profiles_select_desktop_app_event_loop_policy() {
    for profile in [RuntimeSessionProfile::Editor, RuntimeSessionProfile::Dev] {
        let config = runtime_entry_app_config_for_session_profile(profile);

        assert!(config.window_descriptor().primary_window.is_some());
        assert_eq!(config.event_loop_policy(), EventLoopPolicy::DesktopApp);
    }
}

#[test]
fn minimal_and_headless_profiles_disable_primary_window_creation() {
    for profile in [
        RuntimeSessionProfile::Minimal,
        RuntimeSessionProfile::Headless,
    ] {
        let config = runtime_entry_app_config_for_session_profile(profile);

        assert_eq!(config.window_descriptor().primary_window, None);
        assert_eq!(config.event_loop_policy(), EventLoopPolicy::Headless);
        assert_eq!(
            config.window_lifecycle_policy().exit_condition,
            WindowExitCondition::DontExit
        );
    }
}

#[test]
fn first_frame_exit_flag_projects_into_runtime_host_config() {
    let config = runtime_entry_app_config_for_session_profile_with_presented_frame_exit_limit(
        RuntimeSessionProfile::Runtime,
        Some(NonZeroU64::MIN),
    );

    assert!(config.exit_after_first_presented_frame());
}

#[test]
fn first_frame_exit_flag_requires_an_explicit_enabled_value() {
    assert!(!runtime_exit_after_first_frame_enabled_value(None));
    assert!(!runtime_exit_after_first_frame_enabled_value(Some("")));
    assert!(!runtime_exit_after_first_frame_enabled_value(Some("0")));
    assert!(!runtime_exit_after_first_frame_enabled_value(Some("false")));
    assert!(runtime_exit_after_first_frame_enabled_value(Some("1")));
    assert!(runtime_exit_after_first_frame_enabled_value(Some("TRUE")));
    assert!(runtime_exit_after_first_frame_enabled_value(Some("yes")));
}

#[test]
fn presented_frame_exit_limit_accepts_only_a_positive_decimal_count() {
    assert_eq!(
        runtime_presented_frame_exit_limit_from_value(Some(OsString::from("120"))).unwrap(),
        Some(NonZeroU64::new(120).unwrap())
    );
    assert_eq!(
        runtime_presented_frame_exit_limit_from_value(None).unwrap(),
        None
    );
    for value in ["", " ", "0", "-1", "one"] {
        assert!(
            runtime_presented_frame_exit_limit_from_value(Some(OsString::from(value))).is_err()
        );
    }
}

#[test]
fn first_frame_exit_setting_takes_precedence_over_the_multi_frame_value() {
    assert_eq!(
        runtime_presented_frame_exit_limit_from_values(
            Some("true"),
            Some(OsString::from("not-a-number")),
        )
        .unwrap(),
        Some(NonZeroU64::MIN)
    );
}

#[test]
fn presented_frame_exit_limit_projects_into_runtime_host_config() {
    let limit = NonZeroU64::new(120).unwrap();
    let config = runtime_entry_app_config_for_session_profile_with_presented_frame_exit_limit(
        RuntimeSessionProfile::Runtime,
        Some(limit),
    );

    assert_eq!(config.exit_after_presented_frames(), Some(limit));
    assert!(!config.exit_after_first_presented_frame());
}

#[test]
fn runtime_execution_failure_uses_actionable_startup_diagnostic_fields() {
    let error = runtime_startup_execution_error(
        "runtime_session",
        "profile=runtime project=C:/projects/basic",
        "runtime session creation failed: ABI mismatch",
        "verify the selected profile, project, and runtime library ABI before retrying zircon_runtime",
    );

    assert_eq!(
        error.to_string(),
        "runtime startup diagnostic: component=runtime_session requested=profile=runtime project=C:/projects/basic cause=runtime session creation failed: ABI mismatch recovery=verify the selected profile, project, and runtime library ABI before retrying zircon_runtime"
    );
}

#[test]
fn runtime_library_failure_uses_the_selected_session_request() {
    let project_root = ProjectPaths::resolve_path(Path::new("C:/projects/basic"))
        .expect("diagnostic project path should resolve");
    let error = runtime_library_startup_error(
        RuntimeSessionProfile::Dev,
        Some(&project_root),
        "runtime ABI version mismatch",
    );

    assert_eq!(
        error.to_string(),
        format!(
            "runtime startup diagnostic: component=runtime_library requested=profile=dev project={} cause=runtime library loading failed: runtime ABI version mismatch recovery=stage a compatible runtime library beside zircon_runtime or configure ZIRCON_RUNTIME_LIBRARY with a path relative to the product executable or an absolute path",
            project_root.display_path().display()
        )
    );
}

#[cfg(windows)]
#[test]
fn runtime_session_diagnostic_uses_the_resolved_project_display_view() {
    let project_root = ProjectPaths::resolve_path(r"\\?\C:\ZirconBuilds\stage\project")
        .expect("Windows project root should resolve");
    let operation_path = project_root.operation_path().display();

    let diagnostic = project_root.display_diagnostic(format!(
        "runtime project open failed at {operation_path}\\zircon-project.toml"
    ));

    assert_eq!(
        diagnostic,
        r"runtime project open failed at C:\ZirconBuilds\stage\project\zircon-project.toml"
    );
}

#[test]
fn runtime_project_root_resolution_uses_the_physical_template_identity() {
    let template_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("templates")
        .join("projects")
        .join("renderable-empty");
    let requested_root = template_root.join("assets").join("..");

    let resolved = resolve_runtime_project_root(Some(&requested_root))
        .unwrap()
        .expect("a requested project root must resolve");

    assert_eq!(
        resolved,
        ProjectPaths::resolve_existing(&template_root).unwrap()
    );
}

#[test]
fn runtime_project_root_resolution_accepts_the_project_manifest_input() {
    let template_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("templates")
        .join("projects")
        .join("renderable-empty");
    let manifest = template_root.join(PROJECT_MANIFEST_FILE);

    let resolved = resolve_runtime_project_root(Some(&manifest))
        .unwrap()
        .expect("a project manifest input must resolve to its project root");

    assert_eq!(
        resolved,
        ProjectPaths::resolve_existing(&template_root).unwrap()
    );
}

#[test]
fn runtime_project_root_resolution_keeps_a_manifest_named_directory_as_the_root() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos();
    let location =
        runtime_mvp_fixture_root(format!("project-root-{unique}-{}", std::process::id()));
    let project_root = location.join(PROJECT_MANIFEST_FILE);
    std::fs::create_dir_all(&project_root).unwrap();
    std::fs::write(project_root.join(PROJECT_MANIFEST_FILE), "[project]\n").unwrap();

    let resolved = resolve_runtime_project_root(Some(&project_root))
        .unwrap()
        .expect("a directory input must remain the project root regardless of its name");

    assert_eq!(
        resolved,
        ProjectPaths::resolve_existing(&project_root).unwrap()
    );
    std::fs::remove_dir_all(location).unwrap();
}

#[cfg(windows)]
#[test]
fn runtime_project_root_resolution_rejects_drive_relative_paths() {
    let requested_root = Path::new(r"C:zircon-project");
    let error = resolve_runtime_project_root(Some(requested_root)).unwrap_err();

    assert_eq!(
        error.to_string(),
        "runtime startup diagnostic: component=runtime_app argument=--project requested=C:zircon-project cause=project input is not an existing directory or zircon-project.toml recovery=provide an existing project-root directory or zircon-project.toml after --project"
    );
}

#[test]
fn runtime_process_finish_preserves_a_single_terminal_failure() {
    let failure = runtime_startup_execution_error(
        "runtime_event_loop",
        "runtime_event_loop",
        "event loop execution failed: host closed unexpectedly",
        "restart zircon_runtime and inspect the preceding runtime diagnostics",
    );
    let failures = crate::entry::product_shutdown::ProductFailureLedger::default();
    failures.record(
        ProductHostPhase::Running,
        ProductFailureSeverity::Terminal,
        "runtime_event_loop",
        failure,
    );
    let error =
        finish_runtime_process("profile=runtime project=<none>", failures.snapshot()).unwrap_err();

    let diagnostic = error.to_string();
    assert!(diagnostic.contains("component=runtime_process"));
    assert!(diagnostic.contains("recorded=1 suppressed=0"));
    assert!(diagnostic.contains("owner=runtime_event_loop"));
    assert!(diagnostic.contains("event loop execution failed: host closed unexpectedly"));
}

#[test]
fn runtime_process_finish_preserves_all_terminal_failures() {
    let failures = crate::entry::product_shutdown::ProductFailureLedger::default();
    for (phase, owner, message) in [
        (ProductHostPhase::Running, "event_loop", "event loop failed"),
        (
            ProductHostPhase::Running,
            "runtime_app",
            "frame callback failed",
        ),
        (
            ProductHostPhase::DestroyingRuntime,
            "runtime_session",
            "session destroy failed",
        ),
    ] {
        failures.record(phase, ProductFailureSeverity::Terminal, owner, message);
    }
    let error = finish_runtime_process(
        "profile=runtime project=C:/projects/basic",
        failures.snapshot(),
    )
    .unwrap_err();

    let diagnostic = error.to_string();
    assert!(diagnostic.contains("recorded=3 suppressed=0"));
    assert!(diagnostic.contains("sequence=0 phase=running"));
    assert!(diagnostic.contains("owner=event_loop message=event loop failed"));
    assert!(diagnostic.contains("owner=runtime_app message=frame callback failed"));
    assert!(diagnostic.contains(
        "phase=destroying_runtime severity=terminal owner=runtime_session message=session destroy failed"
    ));
}

#[test]
fn terminal_report_failure_is_secondary_to_an_existing_runtime_failure() {
    let failures = crate::entry::product_shutdown::ProductFailureLedger::default();
    failures.record(
        ProductHostPhase::Running,
        ProductFailureSeverity::Terminal,
        "runtime_event_loop",
        "event loop failed",
    );

    record_runtime_terminal_report_failure(
        &failures,
        Err(std::io::Error::other("terminal report write failed").into()),
    );

    let report = failures.snapshot();
    assert_eq!(report.primary().unwrap().owner(), "runtime_event_loop");
    assert_eq!(report.secondary()[0].owner(), "runtime_play_report");
    assert_eq!(
        report.secondary()[0].phase(),
        ProductHostPhase::FlushingDiagnostics
    );
}

#[test]
fn terminal_report_failure_becomes_primary_when_runtime_teardown_succeeded() {
    let failures = crate::entry::product_shutdown::ProductFailureLedger::default();

    record_runtime_terminal_report_failure(
        &failures,
        Err(std::io::Error::other("terminal report write failed").into()),
    );

    assert_eq!(
        failures.snapshot().primary().unwrap().owner(),
        "runtime_play_report"
    );
}

#[test]
fn startup_report_failure_preserves_the_primary_startup_error() {
    let report_failures = crate::entry::product_shutdown::ProductFailureLedger::default();
    record_runtime_report_failure(
        &report_failures,
        ProductHostPhase::Composing,
        Err(std::io::Error::other("startup report write failed").into()),
    );

    let primary = runtime_startup_execution_error(
        "runtime_library",
        "profile=runtime project=<none>",
        "runtime library loading failed: ABI mismatch",
        "stage a compatible runtime library",
    );
    let error = finish_runtime_startup_failure(
        "profile=runtime project=<none>",
        primary.into(),
        report_failures.snapshot(),
    );

    let diagnostic = error.to_string();
    assert!(diagnostic.contains("owner=runtime_startup"));
    assert!(diagnostic.contains("runtime library loading failed: ABI mismatch"));
    assert!(diagnostic.contains("owner=runtime_play_report"));
    assert!(
        diagnostic.find("owner=runtime_startup").unwrap()
            < diagnostic.find("owner=runtime_play_report").unwrap(),
        "the startup error must remain primary when report delivery also fails"
    );
}

#[test]
fn runtime_first_frame_capture_path_resolves_a_relative_environment_value() {
    let path = PathBuf::from("captures/runtime-first-frame.png");

    assert_eq!(
        runtime_frame_capture_path_from_value(Some(path.clone().into_os_string()), None).unwrap(),
        Some(ProjectPaths::resolve_path(&path).unwrap())
    );
}

#[test]
fn runtime_first_frame_capture_path_resolves_relative_to_the_open_project_root() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("templates")
        .join("projects")
        .join("renderable-empty");
    let project_root = ProjectPaths::resolve_existing(project_root).unwrap();
    let path = PathBuf::from("captures/runtime-first-frame.png");

    assert_eq!(
        runtime_frame_capture_path_from_value(
            Some(path.clone().into_os_string()),
            Some(&project_root),
        )
        .unwrap(),
        Some(ProjectPaths::resolve_path_from(&project_root, &path).unwrap())
    );
}

#[test]
fn runtime_first_frame_capture_path_resolves_an_absolute_environment_value() {
    for absolute in [
        PathBuf::from(r"C:\zircon\runtime-first-frame.png"),
        PathBuf::from(r"\\server\share\runtime-first-frame.png"),
    ] {
        assert_eq!(
            runtime_frame_capture_path_from_value(Some(absolute.clone().into_os_string(),), None,)
                .unwrap(),
            Some(ProjectPaths::resolve_path(absolute).unwrap())
        );
    }
}

#[cfg(windows)]
#[test]
fn runtime_first_frame_capture_path_rejects_windows_drive_relative_input() {
    assert!(runtime_frame_capture_path_from_value(
        Some(OsString::from(r"C:runtime-first-frame.png",)),
        None,
    )
    .is_err());
}

#[cfg(unix)]
#[test]
fn runtime_first_frame_capture_path_resolves_non_utf8_absolute_path() {
    use std::os::unix::ffi::OsStringExt;

    let value = OsString::from_vec(vec![b'/', b't', b'm', b'p', b'/', 0xFF]);

    assert_eq!(
        runtime_frame_capture_path_from_value(Some(value.clone()), None).unwrap(),
        Some(ProjectPaths::resolve_path(PathBuf::from(value)).unwrap())
    );
}

#[test]
fn runtime_first_frame_capture_path_rejects_an_empty_or_blank_environment_value() {
    for value in [
        OsString::new(),
        OsString::from(" "),
        OsString::from("\u{2003}"),
    ] {
        let error = runtime_frame_capture_path_from_value(Some(value), None).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_app requested=ZIRCON_RUNTIME_CAPTURE_FRAME_PNG cause=first-frame PNG capture path is empty or blank recovery=set ZIRCON_RUNTIME_CAPTURE_FRAME_PNG to a writable PNG path or unset it"
        );
    }
}

#[test]
fn successful_runtime_teardown_emits_the_staged_product_diagnostic() {
    assert_eq!(
        runtime_process_teardown_complete_diagnostic(),
        "runtime_process_teardown_complete"
    );
}

#[test]
fn unknown_runtime_argument_emits_actionable_startup_diagnostic() {
    let error = EntryRunner::run_runtime_with_args(["--unsupported-runtime-input"]).unwrap_err();

    assert_eq!(
        error.to_string(),
        "runtime startup diagnostic: component=runtime_app argument=<unknown> requested=--unsupported-runtime-input cause=unsupported runtime argument recovery=run zircon_runtime --help to inspect supported startup arguments"
    );
}

#[test]
fn missing_runtime_project_root_emits_actionable_startup_diagnostic() {
    let missing_root = runtime_mvp_fixture_root(format!("missing-project-{}", std::process::id()));
    let requested = missing_root.display().to_string();
    let error = EntryRunner::run_runtime_with_args(["--project".to_string(), requested.clone()])
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        format!(
            "runtime startup diagnostic: component=runtime_app argument=--project requested={requested} cause=project input is not an existing directory or zircon-project.toml recovery=provide an existing project-root directory or zircon-project.toml after --project"
        )
    );
}

#[test]
fn project_root_without_manifest_emits_actionable_startup_diagnostic() {
    let project_root =
        runtime_mvp_fixture_root(format!("project-without-manifest-{}", std::process::id()));
    std::fs::create_dir_all(&project_root).unwrap();
    let requested = project_root.display().to_string();
    let result = EntryRunner::run_runtime_with_args(["--project".to_string(), requested.clone()]);
    std::fs::remove_dir_all(&project_root).unwrap();
    let error = result.unwrap_err();

    assert_eq!(
        error.to_string(),
        format!(
            "runtime startup diagnostic: component=runtime_app argument=--project requested={requested} cause=project manifest zircon-project.toml is missing recovery=provide a Zircon project root containing zircon-project.toml"
        )
    );
}
