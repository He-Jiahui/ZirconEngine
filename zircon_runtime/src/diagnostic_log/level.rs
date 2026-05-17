use std::error::Error;
use std::fmt;

pub const DIAGNOSTIC_LOG_LEVEL_ENV: &str = "ZIRCON_LOG_LEVEL";
pub const DIAGNOSTIC_LOG_FILTER_ENV: &str = "ZIRCON_LOG_FILTER";
pub const DIAGNOSTIC_LOG_ENV: &str = "ZIRCON_LOG";
pub const RUST_LOG_ENV: &str = "RUST_LOG";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLogLevel {
    Verbose,
    Debug,
    Log,
    Warn,
    Error,
}

impl DiagnosticLogLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verbose => "verbose",
            Self::Debug => "debug",
            Self::Log => "log",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

impl fmt::Display for DiagnosticLogLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticLogFilter {
    Off,
    Minimum(DiagnosticLogLevel),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticLogFilterConfig {
    pub minimum: DiagnosticLogFilter,
    pub module_filters: Vec<DiagnosticLogModuleFilter>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticLogModuleFilter {
    pub scope_prefix: String,
    pub filter: DiagnosticLogFilter,
}

impl DiagnosticLogFilter {
    pub const fn default_for_debug_assertions(debug_assertions: bool) -> Self {
        if debug_assertions {
            Self::Minimum(DiagnosticLogLevel::Verbose)
        } else {
            Self::Minimum(DiagnosticLogLevel::Log)
        }
    }

    pub fn default_for_build_profile() -> Self {
        Self::default_for_debug_assertions(cfg!(debug_assertions))
    }

    pub fn from_env_or_default() -> Self {
        let default_filter = Self::default_for_build_profile();
        let Some(raw_value) =
            std::env::var_os(DIAGNOSTIC_LOG_LEVEL_ENV).filter(|value| !value.is_empty())
        else {
            return default_filter;
        };
        let value = raw_value.to_string_lossy();
        match Self::parse(value.as_ref()) {
            Ok(filter) => filter,
            Err(error) => {
                eprintln!(
                    "invalid {DIAGNOSTIC_LOG_LEVEL_ENV} override: {error}; using {default_filter}"
                );
                default_filter
            }
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, DiagnosticLogLevelParseError> {
        let value = value.as_ref().trim();
        let normalized = value.to_ascii_lowercase();
        match normalized.as_str() {
            "verbose" | "trace" => Ok(Self::Minimum(DiagnosticLogLevel::Verbose)),
            "debug" => Ok(Self::Minimum(DiagnosticLogLevel::Debug)),
            "log" | "info" => Ok(Self::Minimum(DiagnosticLogLevel::Log)),
            "warn" | "warning" => Ok(Self::Minimum(DiagnosticLogLevel::Warn)),
            "error" | "err" => Ok(Self::Minimum(DiagnosticLogLevel::Error)),
            "off" | "none" | "quiet" => Ok(Self::Off),
            _ => Err(DiagnosticLogLevelParseError::new(value)),
        }
    }

    pub const fn allows(self, level: DiagnosticLogLevel) -> bool {
        match self {
            Self::Off => false,
            Self::Minimum(minimum) => level as u8 >= minimum as u8,
        }
    }
}

impl DiagnosticLogFilterConfig {
    pub fn new(minimum: DiagnosticLogFilter) -> Self {
        Self {
            minimum,
            module_filters: Vec::new(),
        }
    }

    pub fn from_env_or_default() -> Self {
        let mut config = Self::new(DiagnosticLogFilter::from_env_or_default());
        if let Some((env_name, value)) = selected_filter_env_override(
            non_empty_env_value(DIAGNOSTIC_LOG_FILTER_ENV),
            non_empty_env_value(DIAGNOSTIC_LOG_ENV),
            non_empty_env_value(RUST_LOG_ENV),
        ) {
            match Self::parse(value.as_str(), config.minimum) {
                Ok(parsed) => config = parsed,
                Err(error) => eprintln!(
                    "invalid {env_name} override: {error}; using {}",
                    config.minimum
                ),
            }
        }
        config
    }

    pub fn parse(
        value: impl AsRef<str>,
        fallback_minimum: DiagnosticLogFilter,
    ) -> Result<Self, DiagnosticLogLevelParseError> {
        let mut config = Self::new(fallback_minimum);
        for raw_rule in value
            .as_ref()
            .split(',')
            .map(str::trim)
            .filter(|rule| !rule.is_empty())
        {
            let Some((scope, level)) = raw_rule.split_once('=') else {
                config.minimum = DiagnosticLogFilter::parse(raw_rule)?;
                continue;
            };
            let scope = scope.trim();
            if scope.is_empty() {
                return Err(DiagnosticLogLevelParseError::new(raw_rule));
            }
            config.module_filters.push(DiagnosticLogModuleFilter {
                scope_prefix: scope.to_string(),
                filter: DiagnosticLogFilter::parse(level)?,
            });
        }
        Ok(config)
    }

    pub fn allows(&self, level: DiagnosticLogLevel, scope: &str) -> bool {
        self.filter_for_scope(scope).allows(level)
    }

    pub fn filter_for_scope(&self, scope: &str) -> DiagnosticLogFilter {
        self.module_filters
            .iter()
            .filter(|rule| scope.starts_with(&rule.scope_prefix))
            .max_by_key(|rule| rule.scope_prefix.len())
            .map(|rule| rule.filter)
            .unwrap_or(self.minimum)
    }
}

fn non_empty_env_value(name: &'static str) -> Option<String> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string_lossy().into_owned())
}

fn selected_filter_env_override(
    zircon_log_filter: Option<String>,
    zircon_log: Option<String>,
    rust_log: Option<String>,
) -> Option<(&'static str, String)> {
    zircon_log_filter
        .map(|value| (DIAGNOSTIC_LOG_FILTER_ENV, value))
        .or_else(|| zircon_log.map(|value| (DIAGNOSTIC_LOG_ENV, value)))
        .or_else(|| rust_log.map(|value| (RUST_LOG_ENV, value)))
}

impl From<DiagnosticLogFilter> for DiagnosticLogFilterConfig {
    fn from(value: DiagnosticLogFilter) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for DiagnosticLogFilterConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.module_filters.is_empty() {
            return self.minimum.fmt(formatter);
        }
        write!(formatter, "{}", self.minimum)?;
        for rule in &self.module_filters {
            write!(formatter, ",{}={}", rule.scope_prefix, rule.filter)?;
        }
        Ok(())
    }
}

impl fmt::Display for DiagnosticLogFilter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => formatter.write_str("off"),
            Self::Minimum(level) => level.fmt(formatter),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticLogLevelParseError {
    value: String,
}

impl DiagnosticLogLevelParseError {
    fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for DiagnosticLogLevelParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unknown diagnostic log level `{}`; expected verbose, debug, log, warn, error, off, or scope=level rules",
            self.value
        )
    }
}

impl Error for DiagnosticLogLevelParseError {}

#[cfg(test)]
mod tests {
    use super::{
        selected_filter_env_override, DiagnosticLogFilter, DiagnosticLogFilterConfig,
        DiagnosticLogLevel, DIAGNOSTIC_LOG_ENV, DIAGNOSTIC_LOG_FILTER_ENV, RUST_LOG_ENV,
    };

    #[test]
    fn default_filter_matches_build_profile_policy() {
        assert_eq!(
            DiagnosticLogFilter::default_for_debug_assertions(true),
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Verbose)
        );
        assert_eq!(
            DiagnosticLogFilter::default_for_debug_assertions(false),
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log)
        );
    }

    #[test]
    fn filter_parse_accepts_level_names_and_common_aliases() {
        assert_eq!(
            DiagnosticLogFilter::parse("verbose").unwrap(),
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Verbose)
        );
        assert_eq!(
            DiagnosticLogFilter::parse("trace").unwrap(),
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Verbose)
        );
        assert_eq!(
            DiagnosticLogFilter::parse("debug").unwrap(),
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug)
        );
        assert_eq!(
            DiagnosticLogFilter::parse("info").unwrap(),
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log)
        );
        assert_eq!(
            DiagnosticLogFilter::parse("warning").unwrap(),
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Warn)
        );
        assert_eq!(
            DiagnosticLogFilter::parse("err").unwrap(),
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Error)
        );
        assert_eq!(
            DiagnosticLogFilter::parse("off").unwrap(),
            DiagnosticLogFilter::Off
        );
    }

    #[test]
    fn filter_allows_only_level_at_or_above_minimum() {
        let release_default = DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log);

        assert!(!release_default.allows(DiagnosticLogLevel::Verbose));
        assert!(!release_default.allows(DiagnosticLogLevel::Debug));
        assert!(release_default.allows(DiagnosticLogLevel::Log));
        assert!(release_default.allows(DiagnosticLogLevel::Warn));
        assert!(release_default.allows(DiagnosticLogLevel::Error));
        assert!(!DiagnosticLogFilter::Off.allows(DiagnosticLogLevel::Error));
    }

    #[test]
    fn filter_parse_rejects_unknown_values() {
        let error = DiagnosticLogFilter::parse("chatty").unwrap_err();

        assert_eq!(error.value(), "chatty");
        assert!(error.to_string().contains("unknown diagnostic log level"));
    }

    #[test]
    fn filter_config_parse_supports_global_and_scoped_rules() {
        let config = DiagnosticLogFilterConfig::parse(
            "warn,zircon_runtime::asset=debug,zircon_runtime::asset::import=verbose",
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log),
        )
        .unwrap();

        assert!(config.allows(
            DiagnosticLogLevel::Verbose,
            "zircon_runtime::asset::import::native"
        ));
        assert!(config.allows(DiagnosticLogLevel::Debug, "zircon_runtime::asset::path"));
        assert!(!config.allows(DiagnosticLogLevel::Log, "zircon_runtime::ui"));
        assert!(config.allows(DiagnosticLogLevel::Warn, "zircon_runtime::ui"));
    }

    #[test]
    fn filter_config_env_precedence_prefers_zircon_filter_alias_before_rust_log() {
        assert_eq!(
            selected_filter_env_override(
                Some("warn".to_string()),
                Some("debug".to_string()),
                Some("error".to_string())
            ),
            Some((DIAGNOSTIC_LOG_FILTER_ENV, "warn".to_string()))
        );
        assert_eq!(
            selected_filter_env_override(
                None,
                Some("debug".to_string()),
                Some("error".to_string())
            ),
            Some((DIAGNOSTIC_LOG_ENV, "debug".to_string()))
        );
        assert_eq!(
            selected_filter_env_override(None, None, Some("trace".to_string())),
            Some((RUST_LOG_ENV, "trace".to_string()))
        );
        assert_eq!(selected_filter_env_override(None, None, None), None);
    }
}
