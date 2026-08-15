use std::{
    env,
    error::Error,
    ffi::OsString,
    fmt::{self, Display, Formatter},
    io::Write,
    num::NonZeroU64,
    path::{Path, PathBuf},
};

use winit::event_loop::EventLoop;
use zircon_runtime::asset::project::{ProjectPaths, ResolvedProjectPath, PROJECT_MANIFEST_FILE};
use zircon_runtime::core::framework::window::{
    WindowDescriptor, WindowExitCondition, WindowLifecyclePolicy,
};
use zircon_runtime::platform::EventLoopPolicy;

use super::super::runtime_entry_app::{
    RuntimeEntryApp, RuntimeEntryAppConfig, RuntimeEntryAppFailureState,
};
use super::super::runtime_library::{LoadedRuntime, RuntimeSession, RuntimeWakeRegistration};
use super::runtime_session_args::{
    invalid_runtime_project_root_error, missing_runtime_project_manifest_error,
    parse_runtime_session_startup_args, play_startup_requires_project_error,
    unknown_runtime_argument_error, RuntimeSessionProfile, RUNTIME_SESSION_STARTUP_HELP,
};
use super::EntryRunner;
use crate::entry::cli::parse_diagnostic_log_startup_args;

const RUNTIME_EXIT_AFTER_FIRST_FRAME_ENV: &str = "ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME";
const RUNTIME_EXIT_AFTER_PRESENTED_FRAMES_ENV: &str = "ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES";
const RUNTIME_FRAME_CAPTURE_PNG_ENV: &str = "ZIRCON_RUNTIME_CAPTURE_FRAME_PNG";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayStartupReportPhase {
    Starting,
    Ready,
    StartFailed,
    Terminal,
}

impl PlayStartupReportPhase {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::StartFailed => "start-failed",
            Self::Terminal => "terminal",
        }
    }
}

/// A typed Play report outlet carried by the existing bounded child-output pump.
///
/// The outlet name remains logical so it can be promoted to a native transport without changing
/// the runtime startup ABI. Each report is emitted as one newline-delimited, machine-readable
/// record on stdout; the editor owns process output transport and lifecycle cancellation.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimePlayStartupReporter {
    outlet: String,
}

impl RuntimePlayStartupReporter {
    fn new(outlet: impl Into<String>) -> Self {
        Self {
            outlet: outlet.into(),
        }
    }

    fn emit(
        &self,
        phase: PlayStartupReportPhase,
        detail: impl AsRef<str>,
    ) -> Result<(), RuntimeStartupExecutionError> {
        let record = play_startup_report_record(&self.outlet, phase, detail.as_ref());
        std::io::stdout()
            .lock()
            .write_all(record.as_bytes())
            .map_err(|error| {
                runtime_startup_execution_error(
                    "runtime_play_report",
                    self.outlet.as_str(),
                    format!("failed to write Play startup report: {error}"),
                    "ensure the editor-owned runtime output channel is writable before starting Play",
                )
            })
    }
}

fn play_startup_report_record(outlet: &str, phase: PlayStartupReportPhase, detail: &str) -> String {
    format!(
        "zircon_play_report outlet={outlet} phase={} detail={}\n",
        phase.as_str(),
        sanitize_play_report_detail(detail),
    )
}

fn sanitize_play_report_detail(detail: &str) -> String {
    detail.replace('\r', " ").replace('\n', " ")
}

fn report_play_startup(
    reporter: Option<&RuntimePlayStartupReporter>,
    phase: PlayStartupReportPhase,
    detail: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    reporter
        .map(|reporter| reporter.emit(phase, detail))
        .transpose()
        .map(|_| ())
        .map_err(|error| Box::new(error) as Box<dyn Error>)
}

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
    project_root: Option<&ResolvedProjectPath>,
) -> String {
    let project_root = project_root
        .map(ResolvedProjectPath::display_path)
        .map(|project_root| project_root.display().to_string())
        .unwrap_or_else(|| "<none>".to_owned());
    format!("profile={} project={project_root}", profile.as_str())
}

/// Runtime ABI calls require the operation path, but their type-erased diagnostics must not
/// expose a Windows verbatim path outside the resolver boundary.
fn runtime_project_diagnostic_cause(
    project_root: Option<&ResolvedProjectPath>,
    source: impl Display,
) -> String {
    match project_root {
        Some(root) => root.display_diagnostic(source),
        None => source.to_string(),
    }
}

/// Resolves the command-line project root to the one physical identity used by the runtime.
///
/// This keeps project aliases, junctions, SUBST drives, and symbolic links at the process
/// boundary instead of allowing downstream runtime services to resolve them independently.
fn resolve_runtime_project_root(
    project_root: Option<&Path>,
) -> Result<Option<ResolvedProjectPath>, Box<dyn Error>> {
    let Some(requested_root) = project_root else {
        return Ok(None);
    };
    let project_root = ProjectPaths::resolve_existing(requested_root)
        .map_err(|_| invalid_runtime_project_root_error(requested_root))?;
    let project_root = if ProjectPaths::is_project_manifest_file(project_root.operation_path()) {
        project_root
            .parent()
            .ok_or_else(|| invalid_runtime_project_root_error(requested_root))?
    } else {
        project_root
    };
    if !project_root.operation_path().is_dir() {
        return Err(invalid_runtime_project_root_error(requested_root).into());
    }
    if !project_root
        .operation_path()
        .join(PROJECT_MANIFEST_FILE)
        .is_file()
    {
        return Err(missing_runtime_project_manifest_error(requested_root).into());
    }
    Ok(Some(project_root))
}

fn runtime_library_startup_error(
    profile: RuntimeSessionProfile,
    project_root: Option<&ResolvedProjectPath>,
    source: impl Display,
) -> RuntimeStartupExecutionError {
    runtime_startup_execution_error(
        "runtime_library",
        runtime_session_startup_request(profile, project_root),
        format!(
            "runtime library loading failed: {}",
            runtime_project_diagnostic_cause(project_root, source)
        ),
        "stage a compatible runtime library beside zircon_runtime or configure ZIRCON_RUNTIME_LIBRARY with a path relative to the product executable or an absolute path",
    )
}

fn runtime_frame_capture_path_from_env(
    project_root: Option<&ResolvedProjectPath>,
) -> Result<Option<ResolvedProjectPath>, RuntimeStartupExecutionError> {
    runtime_frame_capture_path_from_value(env::var_os(RUNTIME_FRAME_CAPTURE_PNG_ENV), project_root)
}

fn runtime_frame_capture_path_from_value(
    value: Option<OsString>,
    project_root: Option<&ResolvedProjectPath>,
) -> Result<Option<ResolvedProjectPath>, RuntimeStartupExecutionError> {
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
    let path = PathBuf::from(value);
    let resolved = match project_root {
        Some(project_root) if !path.is_absolute() => {
            ProjectPaths::resolve_path_from(project_root, &path)
        }
        _ => ProjectPaths::resolve_path(&path),
    };
    resolved.map(Some).map_err(|error| {
        runtime_startup_execution_error(
            "runtime_app",
            format!(
                "{RUNTIME_FRAME_CAPTURE_PNG_ENV}={}",
                ProjectPaths::display_path(&path).display()
            ),
            format!("could not resolve first-frame PNG capture path: {error}"),
            "set ZIRCON_RUNTIME_CAPTURE_FRAME_PNG to a writable PNG path or unset it",
        )
    })
}

fn runtime_presented_frame_exit_limit_from_env(
) -> Result<Option<NonZeroU64>, RuntimeStartupExecutionError> {
    runtime_presented_frame_exit_limit_from_values(
        env::var_os(RUNTIME_EXIT_AFTER_FIRST_FRAME_ENV)
            .as_deref()
            .and_then(|value| value.to_str()),
        env::var_os(RUNTIME_EXIT_AFTER_PRESENTED_FRAMES_ENV),
    )
}

fn runtime_presented_frame_exit_limit_from_values(
    first_frame_exit: Option<&str>,
    value: Option<OsString>,
) -> Result<Option<NonZeroU64>, RuntimeStartupExecutionError> {
    if runtime_exit_after_first_frame_enabled_value(first_frame_exit) {
        return Ok(Some(NonZeroU64::MIN));
    }
    runtime_presented_frame_exit_limit_from_value(value)
}

fn runtime_presented_frame_exit_limit_from_value(
    value: Option<OsString>,
) -> Result<Option<NonZeroU64>, RuntimeStartupExecutionError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(value) = value.to_str() else {
        return Err(runtime_startup_execution_error(
            "runtime_app",
            RUNTIME_EXIT_AFTER_PRESENTED_FRAMES_ENV,
            "presented-frame exit limit is not valid UTF-8",
            "set ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES to a positive decimal frame count or unset it",
        ));
    };
    let limit = value.trim().parse::<u64>().ok().and_then(NonZeroU64::new);
    limit.ok_or_else(|| {
        runtime_startup_execution_error(
            "runtime_app",
            format!("{RUNTIME_EXIT_AFTER_PRESENTED_FRAMES_ENV}={value}"),
            "presented-frame exit limit must be a positive decimal frame count",
            "set ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES to a positive decimal frame count or unset it",
        )
    })
    .map(Some)
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
        if runtime_session_args.project_root.is_none() {
            if runtime_session_args.play_scene.is_some() {
                return Err(play_startup_requires_project_error("--play-scene").into());
            }
            if runtime_session_args.play_report_pipe.is_some() {
                return Err(play_startup_requires_project_error("--play-report-pipe").into());
            }
        }
        let play_reporter = runtime_session_args
            .play_report_pipe
            .as_deref()
            .map(RuntimePlayStartupReporter::new);
        let project_root =
            match resolve_runtime_project_root(runtime_session_args.project_root.as_deref()) {
                Ok(project_root) => project_root,
                Err(error) => {
                    report_play_startup(
                        play_reporter.as_ref(),
                        PlayStartupReportPhase::StartFailed,
                        "stage=project-root-resolve",
                    )?;
                    return Err(error);
                }
            };
        let first_frame_capture_path =
            match runtime_frame_capture_path_from_env(project_root.as_ref()) {
                Ok(path) => path,
                Err(error) => {
                    report_play_startup(
                        play_reporter.as_ref(),
                        PlayStartupReportPhase::StartFailed,
                        "stage=frame-capture-path-resolve",
                    )?;
                    return Err(error.into());
                }
            };
        let presented_frame_exit_limit = match runtime_presented_frame_exit_limit_from_env() {
            Ok(limit) => limit,
            Err(error) => {
                report_play_startup(
                    play_reporter.as_ref(),
                    PlayStartupReportPhase::StartFailed,
                    "stage=presented-frame-exit-limit-resolve",
                )?;
                return Err(error.into());
            }
        };
        report_play_startup(
            play_reporter.as_ref(),
            PlayStartupReportPhase::Starting,
            format!(
                "profile={} scene={}",
                runtime_session_args.profile.as_str(),
                runtime_session_args.play_scene.as_ref().map_or(
                    "<default>",
                    zircon_runtime_interface::project::RelPath::as_str
                ),
            ),
        )?;
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
        let runtime = match LoadedRuntime::load_default().map_err(|error| {
            runtime_library_startup_error(
                runtime_session_args.profile,
                project_root.as_ref(),
                error,
            )
        }) {
            Ok(runtime) => runtime,
            Err(error) => {
                report_play_startup(
                    play_reporter.as_ref(),
                    PlayStartupReportPhase::StartFailed,
                    "stage=runtime-library-load",
                )?;
                return Err(error.into());
            }
        };
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_library_load_done");
        let event_loop = match EventLoop::new().map_err(|error| {
            runtime_startup_execution_error(
                "runtime_event_loop",
                "desktop_event_loop",
                format!("event loop creation failed: {error}"),
                "verify the desktop session can create an event loop and retry zircon_runtime",
            )
        }) {
            Ok(event_loop) => event_loop,
            Err(error) => {
                report_play_startup(
                    play_reporter.as_ref(),
                    PlayStartupReportPhase::StartFailed,
                    "stage=event-loop-create",
                )?;
                return Err(error.into());
            }
        };
        let wake_registration = RuntimeWakeRegistration::register(event_loop.create_proxy());
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_session_create_start");
        let session = match RuntimeSession::create_with_profile_and_project(
            runtime,
            runtime_session_args.profile.as_bytes(),
            project_root.as_ref().map(ResolvedProjectPath::operation_path),
            runtime_session_args.play_scene.as_ref(),
            runtime_session_args.play_report_pipe.as_deref(),
            Some(wake_registration),
        )
        .map_err(|error| {
            runtime_startup_execution_error(
                "runtime_session",
                runtime_session_startup_request(
                    runtime_session_args.profile,
                    project_root.as_ref(),
                ),
                format!(
                    "runtime session creation failed: {}",
                    runtime_project_diagnostic_cause(project_root.as_ref(), error)
                ),
                "verify the selected profile, project, and runtime library ABI before retrying zircon_runtime",
            )
        }) {
            Ok(session) => session,
            Err(error) => {
                report_play_startup(
                    play_reporter.as_ref(),
                    PlayStartupReportPhase::StartFailed,
                    "stage=runtime-session-create",
                )?;
                return Err(error.into());
            }
        };
        let session_teardown_failure = session.teardown_failure_state();
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_session_create_done");
        report_play_startup(
            play_reporter.as_ref(),
            PlayStartupReportPhase::Ready,
            "stage=runtime-session-create",
        )?;
        let host_config =
            runtime_entry_app_config_for_session_profile_with_presented_frame_exit_limit(
                runtime_session_args.profile,
                presented_frame_exit_limit,
            )
            .with_persisted_scene_diagnostics(project_root.is_some())
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
                    project_root.as_ref(),
                ),
                format!(
                    "runtime session teardown failed: {}",
                    runtime_project_diagnostic_cause(project_root.as_ref(), error)
                ),
                "verify the runtime surface and session lifecycle, then restart zircon_runtime",
            )) as Box<dyn Error>
        });
        let terminal_result = finish_runtime_process(
            runtime_session_startup_request(runtime_session_args.profile, project_root.as_ref()),
            event_loop_failure,
            runtime_app_failure,
            runtime_session_failure,
        );
        report_play_startup(
            play_reporter.as_ref(),
            PlayStartupReportPhase::Terminal,
            if terminal_result.is_ok() {
                "status=ok"
            } else {
                "status=failed"
            },
        )?;
        terminal_result?;
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
    runtime_entry_app_config_for_session_profile_with_presented_frame_exit_limit(
        profile,
        runtime_exit_after_first_frame_enabled().then_some(NonZeroU64::MIN),
    )
}

fn runtime_entry_app_config_for_session_profile_with_presented_frame_exit_limit(
    profile: RuntimeSessionProfile,
    exit_after_presented_frames: Option<NonZeroU64>,
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
    if let Some(limit) = exit_after_presented_frames {
        config.with_exit_after_presented_frames(limit)
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
    use std::num::NonZeroU64;

    use super::*;

    #[test]
    fn f0_runtime_project_fixture_roots_follow_the_resolved_test_binary_directory() {
        let root = runtime_mvp_fixture_root("physical-root");
        let executable = std::env::current_exe().expect("locate the F0 runtime test executable");
        let binary_directory = executable
            .parent()
            .expect("F0 runtime test executable must have a parent directory");
        let resolved_binary_directory = ProjectPaths::resolve_existing(binary_directory)
            .expect("resolve F0 test binary directory");

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
    fn runtime_first_frame_capture_path_resolves_a_relative_environment_value() {
        let path = PathBuf::from("captures/runtime-first-frame.png");

        assert_eq!(
            runtime_frame_capture_path_from_value(Some(path.clone().into_os_string()), None)
                .unwrap(),
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
                runtime_frame_capture_path_from_value(
                    Some(absolute.clone().into_os_string(),),
                    None,
                )
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
        let error =
            EntryRunner::run_runtime_with_args(["--unsupported-runtime-input"]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_app argument=<unknown> requested=--unsupported-runtime-input cause=unsupported runtime argument recovery=run zircon_runtime --help to inspect supported startup arguments"
        );
    }

    #[test]
    fn missing_runtime_project_root_emits_actionable_startup_diagnostic() {
        let missing_root =
            runtime_mvp_fixture_root(format!("missing-project-{}", std::process::id()));
        let requested = missing_root.display().to_string();
        let error =
            EntryRunner::run_runtime_with_args(["--project".to_string(), requested.clone()])
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
