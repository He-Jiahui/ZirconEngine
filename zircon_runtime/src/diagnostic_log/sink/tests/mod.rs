mod backpressure;
mod batching;
mod durability;
mod fixtures;
mod lifecycle;
mod performance;

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
