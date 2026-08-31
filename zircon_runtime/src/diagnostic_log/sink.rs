use arc_swap::ArcSwapOption;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;

mod metrics;
mod worker;

pub use metrics::DiagnosticLogSinkSnapshot;
use worker::{DurableOutput, SinkRuntime, SINK_THREAD_NAME};

use super::level::{
    CompiledDiagnosticLogFilter, DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel,
};
pub use super::platform::DiagnosticLogLocation;
use super::platform::{log_directory_candidates, LogDirectoryCandidate};
use super::settings::{
    DiagnosticLogSettings, DiagnosticLogSinkSettings, DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT,
};
use super::timestamp::current_log_timestamp;

static LOG_CONTROLLER: OnceLock<ProcessLogController> = OnceLock::new();
static PANIC_FLUSH_HOOK: OnceLock<()> = OnceLock::new();

/// Keeps the current sink generation alive until every dynamic runtime session has stopped.
///
/// The state is unpublished before its worker is joined, so a later dynamic session receives a
/// fresh generation instead of writing to a closed sink.
struct ProcessLogController {
    active_state: ArcSwapOption<DiagnosticLogState>,
    lifecycle: Mutex<ProcessLogLifecycle>,
}

#[derive(Default)]
struct ProcessLogLifecycle {
    dynamic_session_count: usize,
}

impl Default for ProcessLogController {
    fn default() -> Self {
        Self {
            active_state: ArcSwapOption::empty(),
            lifecycle: Mutex::new(ProcessLogLifecycle::default()),
        }
    }
}

impl ProcessLogController {
    fn initialize(&self, settings: DiagnosticLogSettings) -> Arc<DiagnosticLogState> {
        let _lifecycle = self.lock_lifecycle();
        self.active_state
            .load_full()
            .unwrap_or_else(|| self.publish_state(settings))
    }

    fn acquire_dynamic_session(&self, settings: DiagnosticLogSettings) -> Arc<DiagnosticLogState> {
        let mut lifecycle = self.lock_lifecycle();
        let state = self
            .active_state
            .load_full()
            .unwrap_or_else(|| self.publish_state(settings));
        lifecycle.dynamic_session_count = lifecycle.dynamic_session_count.saturating_add(1);
        state
    }

    fn release_dynamic_session(&self) -> bool {
        self.release_dynamic_session_with_timeout(DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT)
    }

    fn release_dynamic_session_with_timeout(&self, timeout: Duration) -> bool {
        let mut lifecycle = self.lock_lifecycle();
        if lifecycle.dynamic_session_count == 0 {
            return false;
        }
        if lifecycle.dynamic_session_count > 1 {
            lifecycle.dynamic_session_count -= 1;
            return true;
        }

        if self
            .shutdown_active_state_for_library_unload(timeout)
            .is_none()
        {
            return false;
        }
        lifecycle.dynamic_session_count = 0;
        true
    }

    fn shutdown_when_idle(&self, timeout: Duration) -> bool {
        let lifecycle = self.lock_lifecycle();
        if lifecycle.dynamic_session_count != 0 {
            return false;
        }
        if self.active_state.load().is_none() {
            return true;
        }
        self.shutdown_active_state_for_library_unload(timeout)
            .is_some_and(|state| state.outputs_succeeded())
    }

    fn shutdown_active_state_for_library_unload(
        &self,
        timeout: Duration,
    ) -> Option<Arc<DiagnosticLogState>> {
        let Some(state) = self.active_state.swap(None) else {
            return None;
        };
        if state.shutdown_for_library_unload(timeout) {
            return Some(state);
        }
        self.active_state.store(Some(state));
        None
    }

    fn active_state(&self) -> Option<Arc<DiagnosticLogState>> {
        self.active_state.load_full()
    }

    fn with_active_state<R>(&self, read: impl FnOnce(Option<&DiagnosticLogState>) -> R) -> R {
        let state = self.active_state.load();
        read(state.as_deref())
    }

    fn publish_state(&self, settings: DiagnosticLogSettings) -> Arc<DiagnosticLogState> {
        let state = Arc::new(DiagnosticLogState::from_settings(settings));
        self.active_state.store(Some(Arc::clone(&state)));
        state
    }

    fn lock_lifecycle(&self) -> MutexGuard<'_, ProcessLogLifecycle> {
        self.lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    fn dynamic_session_count(&self) -> usize {
        self.lock_lifecycle().dynamic_session_count
    }

    #[cfg(test)]
    fn acquire_dynamic_session_for_test(
        &self,
        state: impl FnOnce() -> DiagnosticLogState,
    ) -> Arc<DiagnosticLogState> {
        let mut lifecycle = self.lock_lifecycle();
        let state = self.active_state.load_full().unwrap_or_else(|| {
            let state = Arc::new(state());
            self.active_state.store(Some(Arc::clone(&state)));
            state
        });
        lifecycle.dynamic_session_count = lifecycle.dynamic_session_count.saturating_add(1);
        state
    }

    #[cfg(test)]
    fn release_dynamic_session_for_test(&self) -> bool {
        self.release_dynamic_session()
    }

    #[cfg(test)]
    fn release_dynamic_session_with_timeout_for_test(&self, timeout: Duration) -> bool {
        self.release_dynamic_session_with_timeout(timeout)
    }

    #[cfg(test)]
    fn active_state_for_test(&self) -> Option<Arc<DiagnosticLogState>> {
        self.active_state()
    }

    #[cfg(test)]
    fn dynamic_session_count_for_test(&self) -> usize {
        self.dynamic_session_count()
    }
}

pub(crate) struct DynamicProcessLogLease {
    released: bool,
}

impl DynamicProcessLogLease {
    pub(crate) fn shutdown(&mut self) -> bool {
        if self.released {
            return true;
        }
        if !process_log_controller().release_dynamic_session() {
            return false;
        }
        self.released = true;
        true
    }
}

impl Drop for DynamicProcessLogLease {
    fn drop(&mut self) {
        if !self.released {
            let _ = process_log_controller().release_dynamic_session();
        }
    }
}

fn process_log_controller() -> &'static ProcessLogController {
    LOG_CONTROLLER.get_or_init(ProcessLogController::default)
}

pub fn initialize_process_log(channel: impl Into<String>) -> Option<PathBuf> {
    initialize_process_log_with_location(channel, DiagnosticLogLocation::LocalFirst)
}

pub fn initialize_process_log_with_filter(
    channel: impl Into<String>,
    filter: DiagnosticLogFilter,
) -> Option<PathBuf> {
    initialize_process_log_with_config(channel, filter.into())
}

pub fn initialize_process_log_with_config(
    channel: impl Into<String>,
    filter: DiagnosticLogFilterConfig,
) -> Option<PathBuf> {
    initialize_process_log_with_location_and_filter(
        channel,
        DiagnosticLogLocation::LocalFirst,
        filter,
    )
}

pub fn initialize_unity_process_log(channel: impl Into<String>) -> Option<PathBuf> {
    initialize_process_log_with_location(channel, DiagnosticLogLocation::UnityCompatibleFirst)
}

pub fn initialize_unity_process_log_with_filter(
    channel: impl Into<String>,
    filter: DiagnosticLogFilter,
) -> Option<PathBuf> {
    initialize_unity_process_log_with_config(channel, filter.into())
}

pub fn initialize_unity_process_log_with_config(
    channel: impl Into<String>,
    filter: DiagnosticLogFilterConfig,
) -> Option<PathBuf> {
    initialize_process_log_with_location_and_filter(
        channel,
        DiagnosticLogLocation::UnityCompatibleFirst,
        filter,
    )
}

pub fn initialize_process_log_with_location(
    channel: impl Into<String>,
    location: DiagnosticLogLocation,
) -> Option<PathBuf> {
    initialize_process_log_with_location_and_filter(
        channel,
        location,
        DiagnosticLogFilterConfig::from_env_or_default(),
    )
}

pub fn initialize_process_log_with_location_and_filter(
    channel: impl Into<String>,
    location: DiagnosticLogLocation,
    filter: impl Into<DiagnosticLogFilterConfig>,
) -> Option<PathBuf> {
    initialize_process_log_with_settings(
        DiagnosticLogSettings::new(channel)
            .with_location(location)
            .with_filter(filter),
    )
}

pub fn initialize_process_log_with_settings(settings: DiagnosticLogSettings) -> Option<PathBuf> {
    let requested_channel = sanitize_channel_name(settings.channel.clone());
    let requested_filter = settings.filter.clone();
    let requested_console_enabled = settings.console_enabled;
    let requested_file_enabled = settings.file_enabled;
    let requested_sink_settings = settings.sink.clone();
    let state = process_log_controller().initialize(settings);
    state.write_lazy(DiagnosticLogLevel::Log, "diagnostic_log", || {
        format!(
            "active channel={} requested_channel={} active_filter={} requested_filter={} active_console_enabled={} requested_console_enabled={} active_file_enabled={} requested_file_enabled={} queue_capacity={} requested_queue_capacity={} file={}",
            state.channel,
            requested_channel,
            state.filter,
            requested_filter,
            state.console_enabled,
            requested_console_enabled,
            state.file_enabled,
            requested_file_enabled,
            state.sink_settings.queue_capacity,
            requested_sink_settings.queue_capacity,
            state
                .file_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<file-unavailable>".to_string())
        )
    });
    state.file_path.clone()
}

pub(crate) fn acquire_dynamic_unity_process_log(
    channel: impl Into<String>,
) -> DynamicProcessLogLease {
    let settings = DiagnosticLogSettings::unity_compatible(channel);
    let _state = process_log_controller().acquire_dynamic_session(settings);
    DynamicProcessLogLease { released: false }
}

pub fn write_diagnostic_log(scope: &str, message: impl AsRef<str>) {
    write_diagnostic_log_at(DiagnosticLogLevel::Verbose, scope, message);
}

pub fn diagnostic_log_allows(level: DiagnosticLogLevel) -> bool {
    diagnostic_log_allows_for_scope(level, "")
}

pub fn diagnostic_log_allows_for_scope(level: DiagnosticLogLevel, scope: &str) -> bool {
    process_log_controller()
        .with_active_state(|state| state.is_some_and(|state| state.allows(level, scope)))
}

pub fn write_debug_log(scope: &str, message: impl AsRef<str>) {
    write_diagnostic_log_at(DiagnosticLogLevel::Debug, scope, message);
}

pub fn write_log(scope: &str, message: impl AsRef<str>) {
    write_diagnostic_log_at(DiagnosticLogLevel::Log, scope, message);
}

pub fn write_warn(scope: &str, message: impl AsRef<str>) {
    write_diagnostic_log_at(DiagnosticLogLevel::Warn, scope, message);
}

pub fn write_error(scope: &str, message: impl AsRef<str>) {
    write_diagnostic_log_at(DiagnosticLogLevel::Error, scope, message);
}

pub fn write_diagnostic_log_at(level: DiagnosticLogLevel, scope: &str, message: impl AsRef<str>) {
    process_log_controller().with_active_state(|state| {
        if let Some(state) = state {
            state.write(level, scope, message.as_ref());
        }
    });
}

pub fn write_diagnostic_log_lazy<F, M>(scope: &str, message: F)
where
    F: FnOnce() -> M,
    M: AsRef<str>,
{
    write_diagnostic_log_lazy_at(DiagnosticLogLevel::Verbose, scope, message);
}

pub fn write_debug_log_lazy<F, M>(scope: &str, message: F)
where
    F: FnOnce() -> M,
    M: AsRef<str>,
{
    write_diagnostic_log_lazy_at(DiagnosticLogLevel::Debug, scope, message);
}

pub fn write_log_lazy<F, M>(scope: &str, message: F)
where
    F: FnOnce() -> M,
    M: AsRef<str>,
{
    write_diagnostic_log_lazy_at(DiagnosticLogLevel::Log, scope, message);
}

pub fn write_warn_lazy<F, M>(scope: &str, message: F)
where
    F: FnOnce() -> M,
    M: AsRef<str>,
{
    write_diagnostic_log_lazy_at(DiagnosticLogLevel::Warn, scope, message);
}

pub fn write_error_lazy<F, M>(scope: &str, message: F)
where
    F: FnOnce() -> M,
    M: AsRef<str>,
{
    write_diagnostic_log_lazy_at(DiagnosticLogLevel::Error, scope, message);
}

pub fn write_diagnostic_log_lazy_at<F, M>(level: DiagnosticLogLevel, scope: &str, message: F)
where
    F: FnOnce() -> M,
    M: AsRef<str>,
{
    process_log_controller().with_active_state(|state| {
        if let Some(state) = state {
            state.write_lazy(level, scope, message);
        }
    });
}

pub fn diagnostic_log_sink_snapshot() -> Option<DiagnosticLogSinkSnapshot> {
    process_log_controller().with_active_state(|state| state.and_then(DiagnosticLogState::snapshot))
}

/// Installs one process-wide panic hook that attempts the bounded crash flush before delegating.
pub fn install_process_log_panic_flush(timeout: Duration) {
    PANIC_FLUSH_HOOK.get_or_init(|| {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            if std::thread::current().name() != Some(SINK_THREAD_NAME) {
                let _ = flush_process_log(timeout);
            }
            previous_hook(panic_info);
        }));
    });
}

/// Flushes records queued before this call and `sync_data`s file output.
///
/// Hosts use this bounded handoff at panic/crash boundaries. It returns `false` when the timeout
/// expires or any configured output has failed.
pub fn flush_process_log(timeout: Duration) -> bool {
    process_log_controller()
        .active_state()
        .as_deref()
        .and_then(|state| state.sink.as_ref())
        .is_some_and(|sink| sink.flush(timeout))
}

/// Stops the process sink after flushing console output and `sync_data`-ing file output.
///
/// The return value is `false` on timeout or output failure; callers must not report durability
/// from process exit alone.
pub fn shutdown_process_log(timeout: Duration) -> bool {
    process_log_controller().shutdown_when_idle(timeout)
}

struct DiagnosticLogState {
    channel: String,
    filter: DiagnosticLogFilterConfig,
    compiled_filter: CompiledDiagnosticLogFilter,
    console_enabled: bool,
    file_enabled: bool,
    file_path: Option<PathBuf>,
    sink_settings: DiagnosticLogSinkSettings,
    sink: Option<SinkRuntime>,
}

impl DiagnosticLogState {
    fn from_settings(settings: DiagnosticLogSettings) -> Self {
        Self::new(
            sanitize_channel_name(settings.channel),
            settings.location,
            settings.filter,
            settings.console_enabled,
            settings.file_enabled,
            settings.sink,
        )
    }

    fn new(
        channel: String,
        location: DiagnosticLogLocation,
        filter: DiagnosticLogFilterConfig,
        console_enabled: bool,
        file_enabled: bool,
        sink_settings: DiagnosticLogSinkSettings,
    ) -> Self {
        let timestamp = current_log_timestamp();
        let candidates = log_directory_candidates(&timestamp, location);
        let mut notes = Vec::new();
        let (file_path, file) =
            open_first_available_log_file(&channel, candidates, file_enabled, &mut notes);
        let compiled_filter = CompiledDiagnosticLogFilter::new(&filter);
        let normalized_sink_settings = sink_settings.normalized();
        let sink = if file.is_none() && !console_enabled {
            None
        } else {
            SinkRuntime::start(
                file.map(|file| Box::new(file) as Box<dyn DurableOutput>),
                console_enabled,
                normalized_sink_settings.clone(),
            )
            .map_err(|error| {
                eprintln!("failed to start diagnostic log sink owner: {error}");
            })
            .ok()
        };
        let state = Self {
            channel,
            filter,
            compiled_filter,
            console_enabled,
            file_enabled,
            file_path: sink.as_ref().and(file_path),
            sink_settings: normalized_sink_settings,
            sink,
        };
        for note in notes {
            state.write(note.level, "diagnostic_log", &note.message);
        }
        state
    }

    fn write(&self, level: DiagnosticLogLevel, scope: &str, message: &str) {
        if !self.allows(level, scope) {
            return;
        }
        if let Some(sink) = &self.sink {
            sink.enqueue(level, scope, message);
        }
    }

    fn write_lazy<F, M>(&self, level: DiagnosticLogLevel, scope: &str, message: F)
    where
        F: FnOnce() -> M,
        M: AsRef<str>,
    {
        if !self.allows(level, scope) {
            return;
        }
        if let Some(sink) = &self.sink {
            sink.enqueue_lazy(level, scope, message);
        }
    }

    fn allows(&self, level: DiagnosticLogLevel, scope: &str) -> bool {
        self.sink.as_ref().is_some_and(SinkRuntime::is_open)
            && self.compiled_filter.allows(level, scope)
    }

    fn snapshot(&self) -> Option<DiagnosticLogSinkSnapshot> {
        self.sink.as_ref().map(SinkRuntime::snapshot)
    }

    fn shutdown_for_library_unload(&self, timeout: Duration) -> bool {
        self.sink
            .as_ref()
            .is_none_or(|sink| sink.shutdown_for_library_unload(timeout))
    }

    fn outputs_succeeded(&self) -> bool {
        self.sink
            .as_ref()
            .is_some_and(SinkRuntime::outputs_succeeded)
    }

    #[cfg(test)]
    fn for_test(filter: DiagnosticLogFilterConfig) -> Self {
        Self {
            channel: "test".to_string(),
            compiled_filter: CompiledDiagnosticLogFilter::new(&filter),
            filter,
            console_enabled: false,
            file_enabled: false,
            file_path: None,
            sink_settings: DiagnosticLogSinkSettings::default(),
            sink: None,
        }
    }

    #[cfg(test)]
    fn with_test_sink(sink: SinkRuntime) -> Self {
        let filter = DiagnosticLogFilterConfig::new(DiagnosticLogFilter::Minimum(
            DiagnosticLogLevel::Verbose,
        ));
        Self {
            channel: "test".to_string(),
            compiled_filter: CompiledDiagnosticLogFilter::new(&filter),
            filter,
            console_enabled: false,
            file_enabled: false,
            file_path: None,
            sink_settings: DiagnosticLogSinkSettings::default(),
            sink: Some(sink),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiagnosticLogNote {
    level: DiagnosticLogLevel,
    message: String,
}

impl DiagnosticLogNote {
    fn new(level: DiagnosticLogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
        }
    }
}

fn diagnostic_log_line(
    timestamp: &str,
    level: DiagnosticLogLevel,
    scope: &str,
    message: &str,
) -> String {
    format!(
        "[{timestamp}] [{level}] [{scope}] {}\n",
        message.replace('\n', "\\n")
    )
}

fn open_first_available_log_file(
    channel: &str,
    candidates: Vec<LogDirectoryCandidate>,
    file_enabled: bool,
    notes: &mut Vec<DiagnosticLogNote>,
) -> (Option<PathBuf>, Option<File>) {
    if !file_enabled {
        notes.push(DiagnosticLogNote::new(
            DiagnosticLogLevel::Log,
            "file-backed log sink disabled by diagnostic log settings",
        ));
        return (None, None);
    }

    for candidate in candidates {
        if let Err(error) = std::fs::create_dir_all(&candidate.path) {
            notes.push(DiagnosticLogNote::new(
                DiagnosticLogLevel::Warn,
                format!(
                    "log directory candidate failed source={} path={} error={error}",
                    candidate.source,
                    candidate.path.display()
                ),
            ));
            continue;
        }

        let file_path = candidate.path.join(format!("{channel}.log"));
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
        {
            Ok(file) => {
                notes.push(DiagnosticLogNote::new(
                    DiagnosticLogLevel::Log,
                    format!(
                        "log directory selected source={} path={}",
                        candidate.source,
                        candidate.path.display()
                    ),
                ));
                return (Some(file_path), Some(file));
            }
            Err(error) => notes.push(DiagnosticLogNote::new(
                DiagnosticLogLevel::Warn,
                format!(
                    "log file candidate failed source={} path={} error={error}",
                    candidate.source,
                    file_path.display()
                ),
            )),
        }
    }

    notes.push(DiagnosticLogNote::new(
        DiagnosticLogLevel::Warn,
        "no file-backed log sink available; console diagnostics remain active",
    ));
    (None, None)
}

fn sanitize_channel_name(channel: String) -> String {
    let sanitized = channel
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "runtime".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests;
