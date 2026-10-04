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
use super::super::runtime_library::{
    LoadedRuntime, RuntimeSession, RuntimeSessionCreateFailure, RuntimeWakeRegistration,
};
use super::runtime_session_args::{
    invalid_runtime_project_root_error, missing_runtime_project_manifest_error,
    parse_runtime_session_startup_args, play_startup_requires_project_error,
    unknown_runtime_argument_error, RuntimeSessionProfile, RUNTIME_SESSION_STARTUP_HELP,
};
use super::EntryRunner;
use crate::entry::cli::parse_diagnostic_log_startup_args;
use crate::entry::product_shutdown::{
    ProductFailureLedger, ProductFailureReport, ProductFailureSeverity, ProductHostPhase,
};

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

fn runtime_session_create_diagnostic(
    profile: RuntimeSessionProfile,
    project_root: Option<&ResolvedProjectPath>,
    error: RuntimeSessionCreateFailure,
) -> RuntimeStartupExecutionError {
    let cause = runtime_project_diagnostic_cause(project_root, &error);
    let cause = match error.cleanup_recovery_context() {
        Some(recovery) => format!("runtime session creation failed: {cause}; {recovery}"),
        None => format!("runtime session creation failed: {cause}"),
    };
    runtime_startup_execution_error(
        "runtime_session",
        runtime_session_startup_request(profile, project_root),
        cause,
        "verify the selected profile, project, and runtime library ABI; if retained cleanup is reported, call zircon_app::retry_runtime_startup_cleanup on the creating thread before retrying zircon_runtime",
    )
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
    failure_report: ProductFailureReport,
) -> Result<(), Box<dyn Error>> {
    if failure_report.is_empty() {
        return Ok(());
    }
    Err(runtime_startup_execution_error(
        "runtime_process",
        requested,
        format!("terminal failure ledger: {failure_report}"),
        "inspect every reported terminal failure, repair the lowest runtime owner, and restart zircon_runtime",
    )
    .into())
}

fn record_runtime_terminal_report_failure(
    failures: &ProductFailureLedger,
    result: Result<(), Box<dyn Error>>,
) {
    record_runtime_report_failure(failures, ProductHostPhase::FlushingDiagnostics, result);
}

fn record_runtime_report_failure(
    failures: &ProductFailureLedger,
    phase: ProductHostPhase,
    result: Result<(), Box<dyn Error>>,
) {
    if let Err(error) = result {
        failures.record(
            phase,
            ProductFailureSeverity::Terminal,
            "runtime_play_report",
            error,
        );
    }
}

fn merge_runtime_failure_report(failures: &ProductFailureLedger, report: &ProductFailureReport) {
    for record in report.records() {
        failures.record(
            record.phase(),
            record.severity(),
            record.owner(),
            record.message(),
        );
    }
}

fn finish_runtime_startup_failure(
    requested: impl Into<String>,
    primary: Box<dyn Error>,
    report: ProductFailureReport,
) -> Box<dyn Error> {
    if report.is_empty() {
        return primary;
    }

    let failures = ProductFailureLedger::default();
    failures.record(
        ProductHostPhase::Composing,
        ProductFailureSeverity::Terminal,
        "runtime_startup",
        primary,
    );
    merge_runtime_failure_report(&failures, &report);
    finish_runtime_process(requested, failures.snapshot())
        .expect_err("startup failure ledger must retain the primary startup failure")
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
        let startup_report_failures = ProductFailureLedger::default();
        let project_root =
            match resolve_runtime_project_root(runtime_session_args.project_root.as_deref()) {
                Ok(project_root) => project_root,
                Err(error) => {
                    record_runtime_report_failure(
                        &startup_report_failures,
                        ProductHostPhase::Composing,
                        report_play_startup(
                            play_reporter.as_ref(),
                            PlayStartupReportPhase::StartFailed,
                            "stage=project-root-resolve",
                        ),
                    );
                    return Err(finish_runtime_startup_failure(
                        runtime_session_startup_request(runtime_session_args.profile, None),
                        error,
                        startup_report_failures.snapshot(),
                    ));
                }
            };
        let first_frame_capture_path =
            match runtime_frame_capture_path_from_env(project_root.as_ref()) {
                Ok(path) => path,
                Err(error) => {
                    record_runtime_report_failure(
                        &startup_report_failures,
                        ProductHostPhase::Composing,
                        report_play_startup(
                            play_reporter.as_ref(),
                            PlayStartupReportPhase::StartFailed,
                            "stage=frame-capture-path-resolve",
                        ),
                    );
                    return Err(finish_runtime_startup_failure(
                        runtime_session_startup_request(
                            runtime_session_args.profile,
                            project_root.as_ref(),
                        ),
                        error.into(),
                        startup_report_failures.snapshot(),
                    ));
                }
            };
        let presented_frame_exit_limit = match runtime_presented_frame_exit_limit_from_env() {
            Ok(limit) => limit,
            Err(error) => {
                record_runtime_report_failure(
                    &startup_report_failures,
                    ProductHostPhase::Composing,
                    report_play_startup(
                        play_reporter.as_ref(),
                        PlayStartupReportPhase::StartFailed,
                        "stage=presented-frame-exit-limit-resolve",
                    ),
                );
                return Err(finish_runtime_startup_failure(
                    runtime_session_startup_request(
                        runtime_session_args.profile,
                        project_root.as_ref(),
                    ),
                    error.into(),
                    startup_report_failures.snapshot(),
                ));
            }
        };
        record_runtime_report_failure(
            &startup_report_failures,
            ProductHostPhase::Composing,
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
            ),
        );
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
                record_runtime_report_failure(
                    &startup_report_failures,
                    ProductHostPhase::Composing,
                    report_play_startup(
                        play_reporter.as_ref(),
                        PlayStartupReportPhase::StartFailed,
                        "stage=runtime-library-load",
                    ),
                );
                return Err(finish_runtime_startup_failure(
                    runtime_session_startup_request(
                        runtime_session_args.profile,
                        project_root.as_ref(),
                    ),
                    error.into(),
                    startup_report_failures.snapshot(),
                ));
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
                record_runtime_report_failure(
                    &startup_report_failures,
                    ProductHostPhase::Composing,
                    report_play_startup(
                        play_reporter.as_ref(),
                        PlayStartupReportPhase::StartFailed,
                        "stage=event-loop-create",
                    ),
                );
                return Err(finish_runtime_startup_failure(
                    runtime_session_startup_request(
                        runtime_session_args.profile,
                        project_root.as_ref(),
                    ),
                    error.into(),
                    startup_report_failures.snapshot(),
                ));
            }
        };
        let wake_registration = RuntimeWakeRegistration::register(event_loop.create_proxy());
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_session_create_start");
        let session = match RuntimeSession::create_with_profile_and_project(
            runtime,
            runtime_session_args.profile.as_bytes(),
            project_root
                .as_ref()
                .map(ResolvedProjectPath::operation_path),
            runtime_session_args.play_scene.as_ref(),
            runtime_session_args.play_report_pipe.as_deref(),
            Some(wake_registration),
        )
        .map_err(|error| {
            runtime_session_create_diagnostic(
                runtime_session_args.profile,
                project_root.as_ref(),
                error,
            )
        }) {
            Ok(session) => session,
            Err(error) => {
                record_runtime_report_failure(
                    &startup_report_failures,
                    ProductHostPhase::Composing,
                    report_play_startup(
                        play_reporter.as_ref(),
                        PlayStartupReportPhase::StartFailed,
                        "stage=runtime-session-create",
                    ),
                );
                return Err(finish_runtime_startup_failure(
                    runtime_session_startup_request(
                        runtime_session_args.profile,
                        project_root.as_ref(),
                    ),
                    error.into(),
                    startup_report_failures.snapshot(),
                ));
            }
        };
        let session_teardown_failure = session.teardown_failure_state();
        let product_failure_ledger = session_teardown_failure.failure_ledger();
        merge_runtime_failure_report(&product_failure_ledger, &startup_report_failures.snapshot());
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_session_create_done");
        record_runtime_report_failure(
            &product_failure_ledger,
            ProductHostPhase::Composing,
            report_play_startup(
                play_reporter.as_ref(),
                PlayStartupReportPhase::Ready,
                "stage=runtime-session-create",
            ),
        );
        let host_config =
            runtime_entry_app_config_for_session_profile_with_presented_frame_exit_limit(
                runtime_session_args.profile,
                presented_frame_exit_limit,
            )
            .with_persisted_scene_diagnostics(project_root.is_some())
            .with_reference_cpu_presenter(runtime_session_args.reference_cpu_presenter)
            .with_first_frame_capture_path(first_frame_capture_path);
        let failure_state =
            RuntimeEntryAppFailureState::with_failure_ledger(product_failure_ledger.clone());
        let app = RuntimeEntryApp::new(session, host_config, failure_state);
        let result = event_loop.run_app(app);
        #[cfg(feature = "profiling")]
        if profile_capture.is_some() {
            match zircon_runtime::core::diagnostics::profiling::stop_and_export_capture_from_env() {
                Some(Ok(report)) => eprintln!("profile report exported: {}", report.export_dir),
                Some(Err(error)) => eprintln!("profile report export failed: {error}"),
                None => {}
            }
        }
        if let Err(error) = result {
            product_failure_ledger.record(
                ProductHostPhase::Running,
                ProductFailureSeverity::Terminal,
                "runtime_event_loop",
                runtime_startup_execution_error(
                    "runtime_event_loop",
                    "runtime_event_loop",
                    format!("event loop execution failed: {error}"),
                    "restart zircon_runtime and inspect the preceding runtime diagnostics",
                ),
            );
        }
        let terminal_status = if product_failure_ledger.is_empty() {
            "status=ok"
        } else {
            "status=failed"
        };
        let terminal_report_result = report_play_startup(
            play_reporter.as_ref(),
            PlayStartupReportPhase::Terminal,
            terminal_status,
        );
        record_runtime_terminal_report_failure(&product_failure_ledger, terminal_report_result);
        let failure_report = product_failure_ledger.snapshot();
        let terminal_result = finish_runtime_process(
            runtime_session_startup_request(runtime_session_args.profile, project_root.as_ref()),
            failure_report,
        );
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
#[path = "tests/runtime.rs"]
mod tests;
