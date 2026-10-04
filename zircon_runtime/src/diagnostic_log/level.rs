//! 解析日志环境配置与级别别名；作用域规则按最长字节前缀选择过滤级别，供 sink 初始化及判定复用。
use std::error::Error;
use std::fmt;

mod compiled;

pub(crate) use compiled::CompiledDiagnosticLogFilter;

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

    // 先取构建配置默认值，再读取 ZIRCON_LOG_LEVEL；非法单级值输出错误并回退到默认过滤级别。
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

    // 借用并裁剪单级输入，以 ASCII 不区分大小写比较级别别名；作用域规则由配置解析器处理。
    pub fn parse(value: impl AsRef<str>) -> Result<Self, DiagnosticLogLevelParseError> {
        let value = value.as_ref().trim();
        if value.eq_ignore_ascii_case("verbose") || value.eq_ignore_ascii_case("trace") {
            Ok(Self::Minimum(DiagnosticLogLevel::Verbose))
        } else if value.eq_ignore_ascii_case("debug") {
            Ok(Self::Minimum(DiagnosticLogLevel::Debug))
        } else if value.eq_ignore_ascii_case("log") || value.eq_ignore_ascii_case("info") {
            Ok(Self::Minimum(DiagnosticLogLevel::Log))
        } else if value.eq_ignore_ascii_case("warn") || value.eq_ignore_ascii_case("warning") {
            Ok(Self::Minimum(DiagnosticLogLevel::Warn))
        } else if value.eq_ignore_ascii_case("error") || value.eq_ignore_ascii_case("err") {
            Ok(Self::Minimum(DiagnosticLogLevel::Error))
        } else if value.eq_ignore_ascii_case("off")
            || value.eq_ignore_ascii_case("none")
            || value.eq_ignore_ascii_case("quiet")
        {
            Ok(Self::Off)
        } else {
            Err(DiagnosticLogLevelParseError::new(value))
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
#[path = "tests/level.rs"]
mod tests;

#[cfg(test)]
#[path = "level/tests/borrowed_parse_tests.rs"]
mod borrowed_parse_tests;
