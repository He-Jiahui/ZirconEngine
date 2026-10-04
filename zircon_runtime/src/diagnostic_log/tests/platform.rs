use std::path::Path;

use super::log_directory_under_root;

#[test]
fn log_directory_uses_logs_timestamp_under_root() {
    let path = log_directory_under_root(Path::new("engine"), "2026-05-04-12-30-45");

    assert!(path.ends_with(Path::new("engine/logs/2026-05-04-12-30-45")));
}
