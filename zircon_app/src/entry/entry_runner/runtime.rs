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
use super::diagnostic_log_args::parse_diagnostic_log_startup_args;
use super::runtime_session_args::{
    invalid_runtime_project_root_error, missing_runtime_project_manifest_error,
    parse_runtime_session_startup_args, unknown_runtime_argument_error, RuntimeSessionProfile,
    RUNTIME_SESSION_STARTUP_HELP,
};
use super::EntryRunner;

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
            "set ZIRCON_RUNTIME_CAPTURE_FRAME_PNG to a writable PNG path or unset it",
        ));
    }
    Ok(Some(PathBuf::from(value)))
}

fn runtime_process_teardown_complete_diagnostic() -> &'static str {
    "runtime_process_teardown_complete"
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
        let runtime = LoadedRuntime::load_default()?;
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
        result.map_err(|error| {
            runtime_startup_execution_error(
                "runtime_event_loop",
                "runtime_event_loop",
                format!("event loop execution failed: {error}"),
                "restart zircon_runtime and inspect the preceding runtime diagnostics",
            )
        })?;
        if let Some(failure) = failure_state.take() {
            return Err(failure.into());
        }
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
    fn runtime_first_frame_capture_path_accepts_a_nonempty_environment_value() {
        assert_eq!(
            runtime_frame_capture_path_from_value(Some(OsString::from(
                "E:/evidence/runtime-first-frame.png"
            )))
            .unwrap(),
            Some(PathBuf::from("E:/evidence/runtime-first-frame.png"))
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
