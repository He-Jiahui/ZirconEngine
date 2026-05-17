use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use super::level::{DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel};
pub use super::platform::DiagnosticLogLocation;
use super::platform::{log_directory_candidates, LogDirectoryCandidate};
use super::settings::DiagnosticLogSettings;
use super::timestamp::current_log_timestamp;

static LOG_STATE: OnceLock<DiagnosticLogState> = OnceLock::new();

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
    let state = LOG_STATE.get_or_init(|| {
        DiagnosticLogState::new(
            requested_channel.clone(),
            settings.location,
            settings.filter,
            settings.console_enabled,
            settings.file_enabled,
        )
    });
    write_log(
        "diagnostic_log",
        format!(
            "active channel={} requested_channel={} active_filter={} requested_filter={} active_console_enabled={} requested_console_enabled={} active_file_enabled={} requested_file_enabled={} file={}",
            state.channel,
            requested_channel,
            state.filter,
            requested_filter,
            state.console_enabled,
            requested_console_enabled,
            state.file_enabled,
            requested_file_enabled,
            state
                .file_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<file-unavailable>".to_string())
        ),
    );
    state.file_path.clone()
}

pub fn write_diagnostic_log(scope: &str, message: impl AsRef<str>) {
    write_diagnostic_log_at(DiagnosticLogLevel::Verbose, scope, message);
}

pub fn diagnostic_log_allows(level: DiagnosticLogLevel) -> bool {
    diagnostic_log_allows_for_scope(level, "")
}

pub fn diagnostic_log_allows_for_scope(level: DiagnosticLogLevel, scope: &str) -> bool {
    LOG_STATE
        .get()
        .is_some_and(|state| state.filter.allows(level, scope))
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
    let Some(state) = LOG_STATE.get() else {
        return;
    };
    state.write(level, scope, message.as_ref());
}

struct DiagnosticLogState {
    channel: String,
    filter: DiagnosticLogFilterConfig,
    console_enabled: bool,
    file_enabled: bool,
    file_path: Option<PathBuf>,
    file: Mutex<Option<File>>,
}

impl DiagnosticLogState {
    fn new(
        channel: String,
        location: DiagnosticLogLocation,
        filter: DiagnosticLogFilterConfig,
        console_enabled: bool,
        file_enabled: bool,
    ) -> Self {
        let timestamp = current_log_timestamp();
        let candidates = log_directory_candidates(&timestamp, location);
        let mut notes = Vec::new();
        let (file_path, mut file) =
            open_first_available_log_file(&channel, candidates, file_enabled, &mut notes);

        for note in &notes {
            write_initial_note(
                &mut file,
                &filter,
                console_enabled,
                note.level,
                &note.message,
            );
        }

        Self {
            channel,
            filter,
            console_enabled,
            file_enabled,
            file_path,
            file: Mutex::new(file),
        }
    }

    fn write(&self, level: DiagnosticLogLevel, scope: &str, message: &str) {
        if !self.filter.allows(level, scope) {
            return;
        }
        let line = diagnostic_log_line(&current_log_timestamp(), level, scope, message);
        if self.console_enabled {
            eprint!("{line}");
        }

        if let Ok(mut file) = self.file.lock() {
            if let Some(file) = file.as_mut() {
                let _ = file.write_all(line.as_bytes());
                let _ = file.flush();
            }
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

fn write_initial_note(
    file: &mut Option<File>,
    filter: &DiagnosticLogFilterConfig,
    console_enabled: bool,
    level: DiagnosticLogLevel,
    message: &str,
) {
    if !filter.allows(level, "diagnostic_log") {
        return;
    }

    let line = diagnostic_log_line(&current_log_timestamp(), level, "diagnostic_log", message);
    if console_enabled {
        eprint!("{line}");
    }
    if let Some(file) = file.as_mut() {
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
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
mod tests {
    use std::path::PathBuf;

    use super::{
        diagnostic_log_line, open_first_available_log_file, sanitize_channel_name,
        LogDirectoryCandidate,
    };
    use crate::diagnostic_log::DiagnosticLogLevel;

    #[test]
    fn channel_names_are_safe_file_stems() {
        assert_eq!(sanitize_channel_name("editor".to_string()), "editor");
        assert_eq!(
            sanitize_channel_name("runtime/player".to_string()),
            "runtime_player"
        );
        assert_eq!(sanitize_channel_name(String::new()), "runtime");
    }

    #[test]
    fn diagnostic_log_lines_include_level_scope_and_escape_newlines() {
        let line = diagnostic_log_line(
            "2026-05-04-16-30-00",
            DiagnosticLogLevel::Warn,
            "runtime_asset_path",
            "first\nsecond",
        );

        assert_eq!(
            line,
            "[2026-05-04-16-30-00] [warn] [runtime_asset_path] first\\nsecond\n"
        );
    }

    #[test]
    fn disabled_file_sink_skips_directory_candidates() {
        let mut notes = Vec::new();
        let (path, file) = open_first_available_log_file(
            "runtime",
            vec![LogDirectoryCandidate {
                source: "test",
                path: PathBuf::from("should-not-be-created"),
            }],
            false,
            &mut notes,
        );

        assert!(path.is_none());
        assert!(file.is_none());
        assert_eq!(notes.len(), 1);
        assert!(notes[0].message.contains("file-backed log sink disabled"));
    }
}
