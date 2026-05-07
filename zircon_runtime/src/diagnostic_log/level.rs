use std::error::Error;
use std::fmt;

pub const DIAGNOSTIC_LOG_LEVEL_ENV: &str = "ZIRCON_LOG_LEVEL";

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

impl DiagnosticLogFilter {
    pub const fn default_for_debug_assertions(_debug_assertions: bool) -> Self {
        Self::Minimum(DiagnosticLogLevel::Log)
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
            "unknown diagnostic log level `{}`; expected verbose, debug, log, warn, error, or off",
            self.value
        )
    }
}

impl Error for DiagnosticLogLevelParseError {}

#[cfg(test)]
mod tests {
    use super::{DiagnosticLogFilter, DiagnosticLogLevel};

    #[test]
    fn default_filter_matches_build_profile_policy() {
        assert_eq!(
            DiagnosticLogFilter::default_for_debug_assertions(true),
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log)
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
}
