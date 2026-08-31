use std::time::Duration;

use super::level::{DiagnosticLogFilter, DiagnosticLogFilterConfig};
use super::platform::DiagnosticLogLocation;

pub const DEFAULT_DIAGNOSTIC_LOG_QUEUE_CAPACITY: usize = 4_096;
pub const DEFAULT_DIAGNOSTIC_LOG_BATCH_RECORDS: usize = 256;
pub const DEFAULT_DIAGNOSTIC_LOG_BATCH_BYTES: usize = 256 * 1_024;
pub const DEFAULT_DIAGNOSTIC_LOG_FLUSH_INTERVAL: Duration = Duration::from_millis(50);
pub const DEFAULT_DIAGNOSTIC_LOG_CRITICAL_ENQUEUE_TIMEOUT: Duration = Duration::from_millis(2);
pub const DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT: Duration = Duration::from_millis(250);
pub const DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticLogSinkSettings {
    pub queue_capacity: usize,
    pub max_batch_records: usize,
    pub max_batch_bytes: usize,
    pub flush_interval: Duration,
    pub critical_enqueue_timeout: Duration,
}

impl DiagnosticLogSinkSettings {
    pub fn with_queue_capacity(mut self, capacity: usize) -> Self {
        self.queue_capacity = capacity.max(1);
        self
    }

    pub fn with_max_batch_records(mut self, records: usize) -> Self {
        self.max_batch_records = records.max(1);
        self
    }

    pub fn with_max_batch_bytes(mut self, bytes: usize) -> Self {
        self.max_batch_bytes = bytes.max(1);
        self
    }

    pub fn with_flush_interval(mut self, interval: Duration) -> Self {
        self.flush_interval = interval;
        self
    }

    pub fn with_critical_enqueue_timeout(mut self, timeout: Duration) -> Self {
        self.critical_enqueue_timeout = timeout;
        self
    }

    pub(crate) fn normalized(mut self) -> Self {
        self.queue_capacity = self.queue_capacity.max(1);
        self.max_batch_records = self.max_batch_records.max(1);
        self.max_batch_bytes = self.max_batch_bytes.max(1);
        self
    }
}

impl Default for DiagnosticLogSinkSettings {
    fn default() -> Self {
        Self {
            queue_capacity: DEFAULT_DIAGNOSTIC_LOG_QUEUE_CAPACITY,
            max_batch_records: DEFAULT_DIAGNOSTIC_LOG_BATCH_RECORDS,
            max_batch_bytes: DEFAULT_DIAGNOSTIC_LOG_BATCH_BYTES,
            flush_interval: DEFAULT_DIAGNOSTIC_LOG_FLUSH_INTERVAL,
            critical_enqueue_timeout: DEFAULT_DIAGNOSTIC_LOG_CRITICAL_ENQUEUE_TIMEOUT,
        }
    }
}

/// Runtime-facing log configuration, mirroring Bevy's configurable log plugin surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticLogSettings {
    pub channel: String,
    pub filter: DiagnosticLogFilterConfig,
    pub location: DiagnosticLogLocation,
    pub console_enabled: bool,
    pub file_enabled: bool,
    pub sink: DiagnosticLogSinkSettings,
}

pub type LogSettings = DiagnosticLogSettings;

impl DiagnosticLogSettings {
    pub fn new(channel: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
            filter: DiagnosticLogFilterConfig::from_env_or_default(),
            location: DiagnosticLogLocation::LocalFirst,
            console_enabled: true,
            file_enabled: true,
            sink: DiagnosticLogSinkSettings::default(),
        }
    }

    pub fn unity_compatible(channel: impl Into<String>) -> Self {
        Self::new(channel).with_location(DiagnosticLogLocation::UnityCompatibleFirst)
    }

    pub fn with_filter(mut self, filter: impl Into<DiagnosticLogFilterConfig>) -> Self {
        self.filter = filter.into();
        self
    }

    pub fn with_location(mut self, location: DiagnosticLogLocation) -> Self {
        self.location = location;
        self
    }

    pub fn with_console_enabled(mut self, enabled: bool) -> Self {
        self.console_enabled = enabled;
        self
    }

    pub fn with_file_enabled(mut self, enabled: bool) -> Self {
        self.file_enabled = enabled;
        self
    }

    pub fn with_sink_settings(mut self, settings: DiagnosticLogSinkSettings) -> Self {
        self.sink = settings.normalized();
        self
    }

    pub fn diagnostic_lines(&self) -> Vec<String> {
        let module_filters = if self.filter.module_filters.is_empty() {
            "none".to_string()
        } else {
            self.filter
                .module_filters
                .iter()
                .map(|rule| format!("{}={}", rule.scope_prefix, rule.filter))
                .collect::<Vec<_>>()
                .join(",")
        };

        vec![
            format!("diagnostic_log.channel={}", self.channel),
            format!("diagnostic_log.minimum={}", self.filter.minimum),
            format!("diagnostic_log.filter={}", self.filter),
            format!("diagnostic_log.module_filters={module_filters}"),
            format!("diagnostic_log.location={:?}", self.location),
            format!("diagnostic_log.console_enabled={}", self.console_enabled),
            format!("diagnostic_log.file_enabled={}", self.file_enabled),
            format!("diagnostic_log.queue_capacity={}", self.sink.queue_capacity),
            format!(
                "diagnostic_log.max_batch_records={}",
                self.sink.max_batch_records
            ),
            format!(
                "diagnostic_log.max_batch_bytes={}",
                self.sink.max_batch_bytes
            ),
            format!(
                "diagnostic_log.flush_interval_ms={}",
                self.sink.flush_interval.as_millis()
            ),
            format!(
                "diagnostic_log.critical_enqueue_timeout_ms={}",
                self.sink.critical_enqueue_timeout.as_millis()
            ),
        ]
    }

    pub fn format_diagnostics(&self) -> String {
        self.diagnostic_lines().join("\n")
    }
}

impl Default for DiagnosticLogSettings {
    fn default() -> Self {
        Self::new("runtime")
    }
}

impl From<DiagnosticLogFilter> for DiagnosticLogSettings {
    fn from(filter: DiagnosticLogFilter) -> Self {
        Self::default().with_filter(filter)
    }
}

impl From<DiagnosticLogFilterConfig> for DiagnosticLogSettings {
    fn from(filter: DiagnosticLogFilterConfig) -> Self {
        Self::default().with_filter(filter)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::DiagnosticLogSettings;
    use crate::diagnostic_log::{
        DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel, DiagnosticLogLocation,
    };

    #[test]
    fn settings_format_stable_diagnostics_for_level_filter_and_sinks() {
        let filter = DiagnosticLogFilterConfig::parse(
            "warn,zircon_runtime::asset=debug",
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log),
        )
        .unwrap();
        let settings = DiagnosticLogSettings::new("runtime/player")
            .with_filter(filter)
            .with_location(DiagnosticLogLocation::UnityCompatibleFirst)
            .with_console_enabled(false)
            .with_file_enabled(true);

        let diagnostics = settings.format_diagnostics();

        assert!(diagnostics.contains("diagnostic_log.channel=runtime/player"));
        assert!(diagnostics.contains("diagnostic_log.minimum=warn"));
        assert!(diagnostics.contains("diagnostic_log.filter=warn,zircon_runtime::asset=debug"));
        assert!(diagnostics.contains("diagnostic_log.module_filters=zircon_runtime::asset=debug"));
        assert!(diagnostics.contains("diagnostic_log.location=UnityCompatibleFirst"));
        assert!(diagnostics.contains("diagnostic_log.console_enabled=false"));
        assert!(diagnostics.contains("diagnostic_log.file_enabled=true"));
        assert!(diagnostics.contains("diagnostic_log.queue_capacity=4096"));
        assert!(diagnostics.contains("diagnostic_log.max_batch_records=256"));
        assert!(diagnostics.contains("diagnostic_log.max_batch_bytes=262144"));
        assert!(diagnostics.contains("diagnostic_log.flush_interval_ms=50"));
        assert!(diagnostics.contains("diagnostic_log.critical_enqueue_timeout_ms=2"));
    }

    #[test]
    fn critical_enqueue_timeout_is_configurable_and_visible() {
        let settings = DiagnosticLogSettings::new("runtime").with_sink_settings(
            super::DiagnosticLogSinkSettings::default()
                .with_critical_enqueue_timeout(Duration::from_millis(7)),
        );

        assert_eq!(
            settings.sink.critical_enqueue_timeout,
            Duration::from_millis(7)
        );
        assert!(settings
            .format_diagnostics()
            .contains("diagnostic_log.critical_enqueue_timeout_ms=7"));
    }
}
