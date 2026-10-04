use std::error::Error;
use std::fmt;
use std::num::NonZeroU64;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use zircon_runtime::asset::project::{ProjectPaths, ResolvedProjectPath, PROJECT_MANIFEST_FILE};
use zircon_runtime::core::framework::time::ProductTimeProfile;
use zircon_runtime::core::runtime::ProductTimePolicies;
use zircon_runtime_interface::ZrRuntimeHostRequestV1;

use super::super::runtime_session_args::RuntimeSessionStartupArgs;
use super::controller::HeadlessWait;
use super::managed::{
    self, OwnerWatch, RUNTIME_CALL_TIMEOUT, SHUTDOWN_CALL_TIMEOUT, STARTUP_CALL_TIMEOUT,
};
use super::schedule::HeadlessSchedule;
use super::HeadlessController;
use crate::entry::product_shutdown::{
    ProductFailureLedger, ProductFailureSeverity, ProductHostPhase,
};
use crate::entry::runtime_library::{LoadedRuntime, RuntimeSession, RuntimeWakeRegistration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadlessStopReason {
    Help,
    TickLimit,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeadlessRunReport {
    pub completed_ticks: u64,
    pub overruns: u64,
    pub ready: bool,
    pub reason: HeadlessStopReason,
}

impl HeadlessRunReport {
    pub(super) fn help() -> Self {
        Self {
            completed_ticks: 0,
            overruns: 0,
            ready: false,
            reason: HeadlessStopReason::Help,
        }
    }
}

#[derive(Debug)]
pub enum HeadlessHostError {
    Startup(String),
    Clock(&'static str),
    UnsupportedHostRequest(&'static str),
    Runtime(String),
    Lifecycle(String),
    OperationDeadline(&'static str),
    OperationCancelled(&'static str),
    RetainedOwner {
        cause: Box<HeadlessHostError>,
        controller: HeadlessController,
    },
}

impl fmt::Display for HeadlessHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Startup(detail) => write!(formatter, "headless startup failed: {detail}"),
            Self::Clock(detail) => formatter.write_str(detail),
            Self::UnsupportedHostRequest(kind) => {
                write!(formatter, "headless host capability unsupported: {kind}")
            }
            Self::Runtime(detail) => write!(formatter, "headless runtime failed: {detail}"),
            Self::Lifecycle(detail) => write!(formatter, "headless product failures: {detail}"),
            Self::OperationDeadline(operation) => write!(
                formatter,
                "headless {operation} deadline expired; runtime owner retained"
            ),
            Self::OperationCancelled(operation) => write!(
                formatter,
                "headless {operation} cancellation grace expired; runtime owner retained"
            ),
            Self::RetainedOwner { cause, .. } => fmt::Display::fmt(cause, formatter),
        }
    }
}

impl HeadlessHostError {
    /// A deadline error retains an accessible owner even when the caller moved its controller.
    /// Keep this controller until finish_runtime_until succeeds; final unreaped drop is fatal.
    pub fn retained_controller(&self) -> Option<&HeadlessController> {
        match self {
            Self::RetainedOwner { controller, .. } => Some(controller),
            _ => None,
        }
    }
}

impl Error for HeadlessHostError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RetainedOwner { cause, .. } => Some(cause.as_ref()),
            _ => None,
        }
    }
}

pub(super) trait HeadlessSession {
    fn verify_composition(&self) -> Result<(), HeadlessHostError>;
    fn tick(&mut self) -> Result<(), HeadlessHostError>;
    fn drain_host(&mut self) -> Result<(), HeadlessHostError>;
    fn stop(&mut self) -> Result<(), HeadlessHostError>;
}

impl HeadlessSession for RuntimeSession {
    fn verify_composition(&self) -> Result<(), HeadlessHostError> {
        use zircon_runtime_interface::runtime_build_set::ZrRuntimeSessionProfileV1;
        if self.module_composition_receipt().session_profile != ZrRuntimeSessionProfileV1::Headless
        {
            return Err(HeadlessHostError::Startup(
                "composition receipt is not headless".to_owned(),
            ));
        }
        Ok(())
    }
    fn tick(&mut self) -> Result<(), HeadlessHostError> {
        self.tick_frame()
            .map(|_| ())
            .map_err(|error| HeadlessHostError::Runtime(error.to_string()))
    }

    fn drain_host(&mut self) -> Result<(), HeadlessHostError> {
        let requests = self
            .drain_host_requests()
            .map_err(|error| HeadlessHostError::Runtime(error.to_string()))?;
        if let Some(request) = requests.first() {
            return Err(HeadlessHostError::UnsupportedHostRequest(match request {
                ZrRuntimeHostRequestV1::Ime(_) => "ime",
                ZrRuntimeHostRequestV1::GamepadRumble(_) => "gamepad-rumble",
                ZrRuntimeHostRequestV1::Cursor(_) => "cursor",
                ZrRuntimeHostRequestV1::Clipboard(_) => "clipboard",
                ZrRuntimeHostRequestV1::UiAction(_) => "ui-action",
                ZrRuntimeHostRequestV1::UiHost(_) => "ui-host",
            }));
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<(), HeadlessHostError> {
        self.try_destroy()
            .map_err(|error| HeadlessHostError::Runtime(error.to_string()))
    }
}

pub(super) fn run(
    args: RuntimeSessionStartupArgs,
    tick_limit: Option<NonZeroU64>,
    controller: HeadlessController,
) -> Result<HeadlessRunReport, HeadlessHostError> {
    if controller.is_cancelled() {
        return Ok(cancelled_report());
    }
    managed::run_owned(&controller, move |controller, watch| {
        run_on_owner(args, tick_limit, controller, watch)
    })
}

fn cancelled_report() -> HeadlessRunReport {
    HeadlessRunReport {
        completed_ticks: 0,
        overruns: 0,
        ready: false,
        reason: HeadlessStopReason::Cancelled,
    }
}

fn run_on_owner(
    args: RuntimeSessionStartupArgs,
    tick_limit: Option<NonZeroU64>,
    controller: HeadlessController,
    watch: Arc<OwnerWatch>,
) -> Result<HeadlessRunReport, HeadlessHostError> {
    let failures = watch.failures.clone();
    if controller.is_cancelled() {
        let report = HeadlessRunReport {
            completed_ticks: 0,
            overruns: 0,
            ready: false,
            reason: HeadlessStopReason::Cancelled,
        };
        println!("zircon_server phase=terminal ticks=0 overruns=0 reason=Cancelled");
        return Ok(report);
    }
    println!("zircon_server phase=starting");
    let startup = (|| {
        let _operation = watch.operation("startup", STARTUP_CALL_TIMEOUT);
        ensure_starting(&controller)?;
        let preflight = LoadedRuntime::preflight_default()
            .map_err(|error| HeadlessHostError::Startup(error.to_string()))?;
        ensure_starting(&controller)?;
        let project = resolve_project(args.project_root.as_deref())?;
        ensure_starting(&controller)?;
        let runtime = preflight
            .load_after_preflight()
            .map_err(|error| HeadlessHostError::Startup(error.to_string()))?;
        ensure_starting(&controller)?;
        let wake_controller = controller.clone();
        let wake = RuntimeWakeRegistration::register_callback(move || wake_controller.wake());
        let created = RuntimeSession::create_with_profile_and_project(
            runtime,
            args.profile.as_bytes(),
            project.as_ref().map(ResolvedProjectPath::operation_path),
            args.play_scene.as_ref(),
            None,
            Some(wake),
        );
        match created {
            Ok(session) => Ok(session),
            Err(mut failure) => {
                let error = HeadlessHostError::Startup(failure.to_string());
                failures.record(
                    ProductHostPhase::Composing,
                    ProductFailureSeverity::Terminal,
                    "headless",
                    &error,
                );
                if let Some(session) = failure.take_session() {
                    controller.begin_shutdown();
                    let mut session = ObservedSession::new(session, watch.clone());
                    if let Err(cleanup) = session.stop() {
                        failures.record(
                            ProductHostPhase::DestroyingRuntime,
                            ProductFailureSeverity::Terminal,
                            "headless",
                            cleanup,
                        );
                    }
                    session.finish_cleanup_on_owner();
                }
                Err(error)
            }
        }
    })();
    let policy = ProductTimePolicies::for_profile(ProductTimeProfile::Headless);
    finish_startup(
        startup,
        &controller,
        watch,
        tick_limit,
        policy.time_policy().fixed_timestep(),
    )
}

pub(super) fn finish_startup<S: HeadlessSession>(
    startup: Result<S, HeadlessHostError>,
    controller: &HeadlessController,
    watch: Arc<OwnerWatch>,
    tick_limit: Option<NonZeroU64>,
    timestep: Duration,
) -> Result<HeadlessRunReport, HeadlessHostError> {
    let failures = watch.failures.clone();
    let mut session = match startup {
        Ok(session) => ObservedSession::new(session, watch),
        Err(HeadlessHostError::OperationCancelled("startup")) => {
            println!("zircon_server phase=terminal ticks=0 overruns=0 reason=Cancelled");
            return Ok(cancelled_report());
        }
        Err(error) => {
            failures.record(
                ProductHostPhase::Composing,
                ProductFailureSeverity::Terminal,
                "headless",
                &error,
            );
            eprintln!("zircon_server phase=terminal {}", failures.snapshot());
            return Err(error);
        }
    };
    let result = drive(&mut session, controller, tick_limit, timestep, &failures);
    if let Err(error) = &result {
        eprintln!("zircon_server phase=terminal {error}");
    }
    session.finish_cleanup_on_owner();
    result
}

pub(super) fn ensure_starting(controller: &HeadlessController) -> Result<(), HeadlessHostError> {
    if controller.is_cancelled() {
        Err(HeadlessHostError::OperationCancelled("startup"))
    } else {
        Ok(())
    }
}

pub(super) struct ObservedSession<S> {
    session: S,
    watch: Arc<OwnerWatch>,
    cleanup_pending: bool,
}

impl<S: HeadlessSession> ObservedSession<S> {
    pub(super) fn new(session: S, watch: Arc<OwnerWatch>) -> Self {
        Self {
            session,
            watch,
            cleanup_pending: false,
        }
    }

    pub(super) fn finish_cleanup_on_owner(&mut self) {
        while self.cleanup_pending {
            self.watch.wait_cleanup_retry();
            if let Err(error) = self.stop() {
                self.watch.failures.record(
                    ProductHostPhase::DestroyingRuntime,
                    ProductFailureSeverity::Terminal,
                    "headless-cleanup-retry",
                    error,
                );
            }
        }
    }
}

impl<S: HeadlessSession> HeadlessSession for ObservedSession<S> {
    fn verify_composition(&self) -> Result<(), HeadlessHostError> {
        self.session.verify_composition()
    }

    fn tick(&mut self) -> Result<(), HeadlessHostError> {
        let _operation = self.watch.operation("tick", RUNTIME_CALL_TIMEOUT);
        self.session.tick()
    }

    fn drain_host(&mut self) -> Result<(), HeadlessHostError> {
        let _operation = self.watch.operation("host-drain", RUNTIME_CALL_TIMEOUT);
        self.session.drain_host()
    }

    fn stop(&mut self) -> Result<(), HeadlessHostError> {
        let _operation = self.watch.operation("destroy", SHUTDOWN_CALL_TIMEOUT);
        let result = self.session.stop();
        self.cleanup_pending = result.is_err();
        self.watch.record_destroy(&result);
        result
    }
}

pub(super) fn drive(
    session: &mut impl HeadlessSession,
    controller: &HeadlessController,
    tick_limit: Option<NonZeroU64>,
    timestep: Duration,
    failures: &ProductFailureLedger,
) -> Result<HeadlessRunReport, HeadlessHostError> {
    let result = session
        .verify_composition()
        .and_then(|()| run_loop(session, controller, tick_limit, timestep));
    if let Err(error) = &result {
        failures.record(
            ProductHostPhase::Running,
            ProductFailureSeverity::Terminal,
            "headless",
            error,
        );
    }
    controller.begin_shutdown();
    if let Err(error) = session.stop() {
        failures.record(
            ProductHostPhase::DestroyingRuntime,
            ProductFailureSeverity::Terminal,
            "headless",
            error,
        );
    }
    if !failures.is_empty() {
        return Err(HeadlessHostError::Lifecycle(
            failures.snapshot().to_string(),
        ));
    }
    let report = result?;
    println!(
        "zircon_server phase=terminal ticks={} overruns={} reason={:?}",
        report.completed_ticks, report.overruns, report.reason
    );
    Ok(report)
}

fn run_loop(
    session: &mut impl HeadlessSession,
    controller: &HeadlessController,
    tick_limit: Option<NonZeroU64>,
    timestep: Duration,
) -> Result<HeadlessRunReport, HeadlessHostError> {
    let start = Instant::now();
    let mut schedule = HeadlessSchedule::new(timestep)?;
    let mut completed_ticks = 0u64;
    let mut ready = false;
    let reason = loop {
        let deadline = start
            .checked_add(schedule.next_tick())
            .ok_or(HeadlessHostError::Clock("headless deadline overflow"))?;
        match controller.wait_event(deadline) {
            HeadlessWait::Cancelled => break HeadlessStopReason::Cancelled,
            HeadlessWait::Woken => {
                session.drain_host()?;
                continue;
            }
            HeadlessWait::Deadline => {}
        }
        session.tick()?;
        session.drain_host()?;
        completed_ticks = completed_ticks
            .checked_add(1)
            .ok_or(HeadlessHostError::Clock("headless tick counter overflow"))?;
        if completed_ticks == 1 {
            ready = true;
            println!("zircon_server phase=ready composition=headless first_tick=ok host_drain=ok ticks=1");
        }
        if tick_limit.is_some_and(|limit| completed_ticks >= limit.get()) {
            break HeadlessStopReason::TickLimit;
        }
        schedule.tick_completed(start.elapsed())?;
    };
    Ok(HeadlessRunReport {
        completed_ticks,
        overruns: schedule.overruns(),
        ready,
        reason,
    })
}

fn resolve_project(
    project: Option<&Path>,
) -> Result<Option<ResolvedProjectPath>, HeadlessHostError> {
    let Some(path) = project else { return Ok(None) };
    let resolved = ProjectPaths::resolve_existing(path)
        .map_err(|error| HeadlessHostError::Startup(error.to_string()))?;
    let root = if ProjectPaths::is_project_manifest_file(resolved.operation_path()) {
        resolved.parent().ok_or_else(|| {
            HeadlessHostError::Startup("project manifest has no parent".to_owned())
        })?
    } else {
        resolved
    };
    if !root.operation_path().is_dir()
        || !root.operation_path().join(PROJECT_MANIFEST_FILE).is_file()
    {
        return Err(HeadlessHostError::Startup(
            "project root must contain zircon-project.toml".to_owned(),
        ));
    }
    Ok(Some(root))
}
