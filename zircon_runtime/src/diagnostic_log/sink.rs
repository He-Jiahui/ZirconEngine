use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use super::level::{DiagnosticLogFilter, DiagnosticLogLevel};
pub use super::platform::DiagnosticLogLocation;
use super::platform::{log_directory_candidates, LogDirectoryCandidate};
use super::timestamp::current_log_timestamp;

static LOG_STATE: OnceLock<DiagnosticLogState> = OnceLock::new();

pub fn initialize_process_log(channel: impl Into<String>) -> Option<PathBuf> {
    initialize_process_log_with_location(channel, DiagnosticLogLocation::LocalFirst)
}

pub fn initialize_process_log_with_filter(
    channel: impl Into<String>,
    filter: DiagnosticLogFilter,
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
        DiagnosticLogFilter::from_env_or_default(),
    )
}

pub fn initialize_process_log_with_location_and_filter(
    channel: impl Into<String>,
    location: DiagnosticLogLocation,
    filter: DiagnosticLogFilter,
) -> Option<PathBuf> {
    let requested_channel = sanitize_channel_name(channel.into());
    let state = LOG_STATE
        .get_or_init(|| DiagnosticLogState::new(requested_channel.clone(), location, filter));
    write_log(
        "diagnostic_log",
        format!(
            "active channel={} requested_channel={} active_filter={} requested_filter={} file={}",
            state.channel,
            requested_channel,
            state.filter,
            filter,
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
    LOG_STATE
        .get()
        .is_some_and(|state| state.filter.allows(level))
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
    filter: DiagnosticLogFilter,
    file_path: Option<PathBuf>,
    file: Mutex<Option<File>>,
}

impl DiagnosticLogState {
    fn new(channel: String, location: DiagnosticLogLocation, filter: DiagnosticLogFilter) -> Self {
        let timestamp = current_log_timestamp();
        let candidates = log_directory_candidates(&timestamp, location);
        let mut notes = Vec::new();
        let (file_path, mut file) = open_first_available_log_file(&channel, candidates, &mut notes);

        for note in &notes {
            write_initial_note(&mut file, filter, note.level, &note.message);
        }

        Self {
            channel,
            filter,
            file_path,
            file: Mutex::new(file),
        }
    }

    fn write(&self, level: DiagnosticLogLevel, scope: &str, message: &str) {
        if !self.filter.allows(level) {
            return;
        }
        let line = diagnostic_log_line(&current_log_timestamp(), level, scope, message);
        eprint!("{line}");

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
    filter: DiagnosticLogFilter,
    level: DiagnosticLogLevel,
    message: &str,
) {
    if !filter.allows(level) {
        return;
    }

    let line = diagnostic_log_line(&current_log_timestamp(), level, "diagnostic_log", message);
    eprint!("{line}");
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
    notes: &mut Vec<DiagnosticLogNote>,
) -> (Option<PathBuf>, Option<File>) {
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
    use super::{diagnostic_log_line, sanitize_channel_name};
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
}
