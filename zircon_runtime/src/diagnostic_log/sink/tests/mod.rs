//! sink 测试入口把格式、文件候选、批处理、背压、生命周期和性能契约组合到同一私有模块。
#[path = "backpressure.rs"]
mod backpressure;
#[path = "batching.rs"]
mod batching;
#[path = "durability.rs"]
mod durability;
#[path = "fixtures.rs"]
mod fixtures;
#[path = "lifecycle.rs"]
mod lifecycle;
#[path = "performance/mod.rs"]
mod performance;

use std::path::PathBuf;

use super::{
    append_diagnostic_log_line, open_first_available_log_file, sanitize_channel_name,
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
    let mut buffer = b"[2026-05-04-16-30-00] ".to_vec();
    append_diagnostic_log_line(
        &mut buffer,
        DiagnosticLogLevel::Warn,
        "runtime_asset_path",
        "first\nsecond",
    );
    let line = String::from_utf8(buffer).expect("diagnostic log line is UTF-8");

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

#[test]
fn diagnostic_log_lines_preserve_literal_backslashes_and_unicode() {
    let mut buffer = b"[2026-05-04-16-30-00] ".to_vec();
    append_diagnostic_log_line(
        &mut buffer,
        DiagnosticLogLevel::Debug,
        "作用域",
        "literal \\n and C:\\assets\\界面\n第二行 🧭",
    );

    assert_eq!(
        String::from_utf8(buffer).expect("diagnostic log line is UTF-8"),
        "[2026-05-04-16-30-00] [debug] [作用域] literal \\n and C:\\assets\\界面\\n第二行 🧭\n"
    );
}
