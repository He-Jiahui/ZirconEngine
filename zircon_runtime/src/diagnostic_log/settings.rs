use super::level::{DiagnosticLogFilter, DiagnosticLogFilterConfig};
use super::platform::DiagnosticLogLocation;

/// Runtime-facing log configuration, mirroring Bevy's configurable log plugin surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticLogSettings {
    pub channel: String,
    pub filter: DiagnosticLogFilterConfig,
    pub location: DiagnosticLogLocation,
    pub console_enabled: bool,
    pub file_enabled: bool,
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
    }
}
