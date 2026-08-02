use std::{
    env,
    error::Error,
    ffi::OsString,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use winit::event_loop::EventLoop;
use zircon_runtime::core::framework::window::{
    WindowDescriptor, WindowExitCondition, WindowLifecyclePolicy,
};
use zircon_runtime::platform::EventLoopPolicy;

use super::super::runtime_entry_app::{
    RuntimeEntryApp, RuntimeEntryAppConfig, RuntimeEntryAppFailureState,
};
use super::super::runtime_library::{LoadedRuntime, RuntimeSession, RuntimeWakeRegistration};
use super::EntryRunner;
use super::diagnostic_log_args::parse_diagnostic_log_startup_args;
use super::runtime_session_args::{
    RUNTIME_SESSION_STARTUP_HELP, RuntimeSessionProfile, invalid_runtime_project_root_error,
    missing_runtime_project_manifest_error, parse_runtime_session_startup_args,
    unknown_runtime_argument_error,
};

const RUNTIME_EXIT_AFTER_FIRST_FRAME_ENV: &str = "ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME";
const RUNTIME_FRAME_CAPTURE_PNG_ENV: &str = "ZIRCON_RUNTIME_CAPTURE_FRAME_PNG";

#[derive(Debug)]
struct RuntimeStartupExecutionError {
    component: &'static str,
    requested: String,
    cause: String,
    recovery: &'static str,
}

impl Display for RuntimeStartupExecutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "runtime startup diagnostic: component={} requested={} cause={} recovery={}",
            self.component, self.requested, self.cause, self.recovery
        )
    }
}

impl Error for RuntimeStartupExecutionError {}

fn runtime_startup_execution_error(
    component: &'static str,
    requested: impl Into<String>,
    cause: impl Into<String>,
    recovery: &'static str,
) -> RuntimeStartupExecutionError {
    RuntimeStartupExecutionError {
        component,
        requested: requested.into(),
        cause: cause.into(),
        recovery,
    }
}

fn runtime_session_startup_request(
    profile: RuntimeSessionProfile,
    project_root: Option<&Path>,
) -> String {
    let project_root = project_root
        .map(|project_root| project_root.display().to_string())
        .unwrap_or_else(|| "<none>".to_owned());
    format!("profile={} project={project_root}", profile.as_str())
}

fn runtime_library_startup_error(
    profile: RuntimeSessionProfile,
    project_root: Option<&Path>,
    source: impl Display,
) -> RuntimeStartupExecutionError {
    runtime_startup_execution_error(
        "runtime_library",
        runtime_session_startup_request(profile, project_root),
        format!("runtime library loading failed: {source}"),
        "stage a compatible runtime library beside zircon_runtime or configure ZIRCON_RUNTIME_LIBRARY with an absolute path",
    )
}

fn runtime_frame_capture_path_from_env() -> Result<Option<PathBuf>, RuntimeStartupExecutionError> {
    runtime_frame_capture_path_from_value(env::var_os(RUNTIME_FRAME_CAPTURE_PNG_ENV))
}

fn runtime_frame_capture_path_from_value(
    value: Option<OsString>,
) -> Result<Option<PathBuf>, RuntimeStartupExecutionError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.as_os_str().is_empty() || value.to_str().is_some_and(|value| value.trim().is_empty()) {
        return Err(runtime_startup_execution_error(
            "runtime_app",
            RUNTIME_FRAME_CAPTURE_PNG_ENV,
            "first-frame PNG capture path is empty or blank",
            "set ZIRCON_RUNTIME_CAPTURE_FRAME_PNG to a writable absolute PNG path or unset it",
        ));
    }
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(runtime_startup_execution_error(
            "runtime_app",
            format!("{RUNTIME_FRAME_CAPTURE_PNG_ENV}={}", path.display()),
            "first-frame PNG capture path must be absolute",
            "set ZIRCON_RUNTIME_CAPTURE_FRAME_PNG to a writable absolute PNG path or unset it",
        ));
    }
    Ok(Some(path))
}

fn runtime_process_teardown_complete_diagnostic() -> &'static str {
    "runtime_process_teardown_complete"
}

fn finish_runtime_process(
    requested: impl Into<String>,
    event_loop_failure: Option<Box<dyn Error>>,
    runtime_app_failure: Option<Box<dyn Error>>,
    runtime_session_failure: Option<Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let mut failures = Vec::with_capacity(3);
    if let Some(failure) = event_loop_failure {
        failures.push(("event_loop", failure));
    }
    if let Some(failure) = runtime_app_failure {
        failures.push(("runtime_app", failure));
    }
    if let Some(failure) = runtime_session_failure {
        failures.push(("runtime_session", failure));
    }

    if failures.is_empty() {
        return Ok(());
    }
    if failures.len() == 1 {
        if let Some((_, failure)) = failures.pop() {
            return Err(failure);
        }
    }

    let causes = failures
        .iter()
        .map(|(component, failure)| format!("{component}: {failure}"))
        .collect::<Vec<_>>()
        .join(" | ");
    Err(runtime_startup_execution_error(
        "runtime_process",
        requested,
        format!("multiple terminal failures: {causes}"),
        "inspect every reported terminal failure, repair the lowest runtime owner, and restart zircon_runtime",
    )
    .into())
}

impl EntryRunner {
    pub fn run_runtime() -> Result<(), Box<dyn Error>> {
        Self::run_runtime_with_args(std::iter::empty::<String>())
    }

    pub fn run_runtime_with_args<I, S>(args: I) -> Result<(), Box<dyn Error>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let diagnostic_args = parse_diagnostic_log_startup_args(args)?;
        let runtime_session_args =
            parse_runtime_session_startup_args(diagnostic_args.remaining_args)?;
        if runtime_session_args.help_requested {
            println!("{RUNTIME_SESSION_STARTUP_HELP}");
            return Ok(());
        }
        if !runtime_session_args.remaining_args.is_empty() {
            return Err(unknown_runtime_argument_error(
                runtime_session_args.remaining_args[0].clone(),
            )
            .into());
        }
        if let Some(project_root) = runtime_session_args.project_root.as_deref() {
            if !project_root.is_dir() {
                return Err(invalid_runtime_project_root_error(project_root).into());
            }
            if !project_root.join("zircon-project.toml").is_file() {
                return Err(missing_runtime_project_manifest_error(project_root).into());
            }
        }
        let first_frame_capture_path = runtime_frame_capture_path_from_env()?;
        zircon_runtime::diagnostic_log::initialize_unity_process_log_with_config(
            "runtime",
            diagnostic_args.filter,
        );
        #[cfg(feature = "profiling-tracy")]
        let _ = zircon_runtime::core::diagnostics::profiling::initialize_tracy_sink();
        #[cfg(feature = "profiling")]
        let profile_capture =
            zircon_runtime::core::diagnostics::profiling::start_capture_from_env("runtime");
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_library_load_start");
        let runtime = LoadedRuntime::load_default().map_err(|error| {
            runtime_library_startup_error(
                runtime_session_args.profile,
                runtime_session_args.project_root.as_deref(),
                error,
            )
        })?;
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_library_load_done");
        let event_loop = EventLoop::new().map_err(|error| {
            runtime_startup_execution_error(
                "runtime_event_loop",
                "desktop_event_loop",
                format!("event loop creation failed: {error}"),
                "verify the desktop session can create an event loop and retry zircon_runtime",
            )
        })?;
        let wake_registration = RuntimeWakeRegistration::register(event_loop.create_proxy());
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_session_create_start");
        let session = RuntimeSession::create_with_profile_and_project(
            runtime,
            runtime_session_args.profile.as_bytes(),
            runtime_session_args.project_root.as_deref(),
            Some(wake_registration),
        )
        .map_err(|error| {
            runtime_startup_execution_error(
                "runtime_session",
                runtime_session_startup_request(
                    runtime_session_args.profile,
                    runtime_session_args.project_root.as_deref(),
                ),
                format!("runtime session creation failed: {error}"),
                "verify the selected profile, project, and runtime library ABI before retrying zircon_runtime",
            )
        })?;
        let session_teardown_failure = session.teardown_failure_state();
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_session_create_done");
        let host_config = runtime_entry_app_config_for_session_profile_with_first_frame_exit(
            runtime_session_args.profile,
            runtime_exit_after_first_frame_enabled(),
        )
        .with_persisted_scene_diagnostics(runtime_session_args.project_root.is_some())
        .with_first_frame_capture_path(first_frame_capture_path);
        let failure_state = RuntimeEntryAppFailureState::default();
        let app = RuntimeEntryApp::new(session, host_config, failure_state.clone());
        let result = event_loop.run_app(app);
        #[cfg(feature = "profiling")]
        if profile_capture.is_some() {
            match zircon_runtime::core::diagnostics::profiling::stop_and_export_capture_from_env() {
                Some(Ok(report)) => eprintln!("profile report exported: {}", report.export_dir),
                Some(Err(error)) => eprintln!("profile report export failed: {error}"),
                None => {}
            }
        }
        let event_loop_failure = result.err().map(|error| {
            Box::new(runtime_startup_execution_error(
                "runtime_event_loop",
                "runtime_event_loop",
                format!("event loop execution failed: {error}"),
                "restart zircon_runtime and inspect the preceding runtime diagnostics",
            )) as Box<dyn Error>
        });
        let runtime_app_failure = failure_state
            .take()
            .map(|failure| Box::new(failure) as Box<dyn Error>);
        let runtime_session_failure = session_teardown_failure.take().map(|error| {
            Box::new(runtime_startup_execution_error(
                "runtime_session",
                runtime_session_startup_request(
                    runtime_session_args.profile,
                    runtime_session_args.project_root.as_deref(),
                ),
                format!("runtime session teardown failed: {error}"),
                "verify the runtime surface and session lifecycle, then restart zircon_runtime",
            )) as Box<dyn Error>
        });
        finish_runtime_process(
            runtime_session_startup_request(
                runtime_session_args.profile,
                runtime_session_args.project_root.as_deref(),
            ),
            event_loop_failure,
            runtime_app_failure,
            runtime_session_failure,
        )?;
        zircon_runtime::diagnostic_log::write_log(
            "runtime_app",
            runtime_process_teardown_complete_diagnostic(),
        );
        Ok(())
    }
}

#[cfg(test)]
fn runtime_entry_app_config_for_session_profile(
    profile: RuntimeSessionProfile,
) -> RuntimeEntryAppConfig {
    runtime_entry_app_config_for_session_profile_with_first_frame_exit(
        profile,
        runtime_exit_after_first_frame_enabled(),
    )
}

fn runtime_entry_app_config_for_session_profile_with_first_frame_exit(
    profile: RuntimeSessionProfile,
    exit_after_first_frame: bool,
) -> RuntimeEntryAppConfig {
    let config = match profile {
        RuntimeSessionProfile::Runtime | RuntimeSessionProfile::RuntimePipelined => {
            RuntimeEntryAppConfig::default()
        }
        RuntimeSessionProfile::Editor | RuntimeSessionProfile::Dev => {
            RuntimeEntryAppConfig::default().with_event_loop_policy(EventLoopPolicy::DesktopApp)
        }
        RuntimeSessionProfile::Minimal | RuntimeSessionProfile::Headless => {
            RuntimeEntryAppConfig::default()
                .with_window_descriptor(WindowDescriptor::default().without_primary_window())
                .with_event_loop_policy(EventLoopPolicy::Headless)
                .with_window_lifecycle_policy(
                    WindowLifecyclePolicy::default()
                        .with_exit_condition(WindowExitCondition::DontExit),
                )
        }
    };
    if exit_after_first_frame {
        config.with_exit_after_first_presented_frame(true)
    } else {
        config
    }
}

fn runtime_exit_after_first_frame_enabled() -> bool {
    runtime_exit_after_first_frame_enabled_value(
        env::var_os(RUNTIME_EXIT_AFTER_FIRST_FRAME_ENV)
            .as_deref()
            .and_then(|value| value.to_str()),
    )
}

fn runtime_exit_after_first_frame_enabled_value(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        value == "1" || value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("yes")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_session_profile_selects_default_game_host_config() {
        let config = runtime_entry_app_config_for_session_profile(RuntimeSessionProfile::Runtime);

        assert!(config.window_descriptor().primary_window.is_some());
        assert_eq!(config.event_loop_policy(), EventLoopPolicy::Game);
        assert!(
            config
                .window_lifecycle_policy()
                .should_exit_after_primary_close()
        );
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
        let config = runtime_entry_app_config_for_session_profile_with_first_frame_exit(
            RuntimeSessionProfile::Runtime,
            true,
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
        let error = runtime_library_startup_error(
            RuntimeSessionProfile::Dev,
            Some(Path::new("C:/projects/basic")),
            "runtime ABI version mismatch",
        );

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_library requested=profile=dev project=C:/projects/basic cause=runtime library loading failed: runtime ABI version mismatch recovery=stage a compatible runtime library beside zircon_runtime or configure ZIRCON_RUNTIME_LIBRARY with an absolute path"
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
        let error = finish_runtime_process(
            "profile=runtime project=<none>",
            Some(Box::new(failure)),
            None,
            None,
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_event_loop requested=runtime_event_loop cause=event loop execution failed: host closed unexpectedly recovery=restart zircon_runtime and inspect the preceding runtime diagnostics"
        );
    }

    #[test]
    fn runtime_process_finish_preserves_all_terminal_failures() {
        let error = finish_runtime_process(
            "profile=runtime project=C:/projects/basic",
            Some(Box::new(std::io::Error::other("event loop failed"))),
            Some(Box::new(std::io::Error::other("frame callback failed"))),
            Some(Box::new(std::io::Error::other("session destroy failed"))),
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_process requested=profile=runtime project=C:/projects/basic cause=multiple terminal failures: event_loop: event loop failed | runtime_app: frame callback failed | runtime_session: session destroy failed recovery=inspect every reported terminal failure, repair the lowest runtime owner, and restart zircon_runtime"
        );
    }

    #[test]
    fn runtime_first_frame_capture_path_accepts_an_absolute_environment_value() {
        let path = std::path::absolute("runtime-first-frame.png").unwrap();

        assert_eq!(
            runtime_frame_capture_path_from_value(Some(path.clone().into_os_string())).unwrap(),
            Some(path)
        );
    }

    #[test]
    fn runtime_first_frame_capture_path_rejects_a_relative_environment_value() {
        let error = runtime_frame_capture_path_from_value(Some(OsString::from(
            "captures/runtime-first-frame.png",
        )))
        .expect_err("relative capture path must not depend on the process working directory");

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_app requested=ZIRCON_RUNTIME_CAPTURE_FRAME_PNG=captures/runtime-first-frame.png cause=first-frame PNG capture path must be absolute recovery=set ZIRCON_RUNTIME_CAPTURE_FRAME_PNG to a writable absolute PNG path or unset it"
        );
    }

    #[cfg(windows)]
    #[test]
    fn runtime_first_frame_capture_path_preserves_windows_absolute_path_semantics() {
        for absolute in [
            PathBuf::from(r"C:\zircon\runtime-first-frame.png"),
            PathBuf::from(r"\\server\share\runtime-first-frame.png"),
        ] {
            assert_eq!(
                runtime_frame_capture_path_from_value(Some(absolute.clone().into_os_string(),))
                    .unwrap(),
                Some(absolute)
            );
        }

        for relative in [
            OsString::from(r"C:runtime-first-frame.png"),
            OsString::from(r"\runtime-first-frame.png"),
            OsString::from(r"/runtime-first-frame.png"),
        ] {
            assert!(runtime_frame_capture_path_from_value(Some(relative)).is_err());
        }
    }

    #[cfg(unix)]
    #[test]
    fn runtime_first_frame_capture_path_preserves_non_utf8_absolute_path() {
        use std::os::unix::ffi::OsStringExt;

        let value = OsString::from_vec(vec![b'/', b't', b'm', b'p', b'/', 0xFF]);

        assert_eq!(
            runtime_frame_capture_path_from_value(Some(value.clone())).unwrap(),
            Some(PathBuf::from(value))
        );
    }

    #[test]
    fn runtime_first_frame_capture_path_rejects_an_empty_or_blank_environment_value() {
        for value in [
            OsString::new(),
            OsString::from(" "),
            OsString::from("\u{2003}"),
        ] {
            let error = runtime_frame_capture_path_from_value(Some(value)).unwrap_err();

            assert_eq!(
                error.to_string(),
                "runtime startup diagnostic: component=runtime_app requested=ZIRCON_RUNTIME_CAPTURE_FRAME_PNG cause=first-frame PNG capture path is empty or blank recovery=set ZIRCON_RUNTIME_CAPTURE_FRAME_PNG to a writable absolute PNG path or unset it"
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
        let error =
            EntryRunner::run_runtime_with_args(["--unsupported-runtime-input"]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_app argument=<unknown> requested=--unsupported-runtime-input cause=unsupported runtime argument recovery=run zircon_runtime --help to inspect supported startup arguments"
        );
    }

    #[test]
    fn missing_runtime_project_root_emits_actionable_startup_diagnostic() {
        let missing_root = std::env::temp_dir().join(format!(
            "zircon-runtime-missing-project-{}",
            std::process::id()
        ));
        let requested = missing_root.display().to_string();
        let error =
            EntryRunner::run_runtime_with_args(["--project".to_string(), requested.clone()])
                .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "runtime startup diagnostic: component=runtime_app argument=--project requested={requested} cause=project root is not an existing directory recovery=provide an existing project-root directory after --project"
            )
        );
    }

    #[test]
    fn project_root_without_manifest_emits_actionable_startup_diagnostic() {
        let project_root = std::env::temp_dir().join(format!(
            "zircon-runtime-project-without-manifest-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&project_root).unwrap();
        let requested = project_root.display().to_string();
        let result =
            EntryRunner::run_runtime_with_args(["--project".to_string(), requested.clone()]);
        std::fs::remove_dir_all(&project_root).unwrap();
        let error = result.unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "runtime startup diagnostic: component=runtime_app argument=--project requested={requested} cause=project manifest zircon-project.toml is missing recovery=provide a Zircon project root containing zircon-project.toml"
            )
        );
    }
}
