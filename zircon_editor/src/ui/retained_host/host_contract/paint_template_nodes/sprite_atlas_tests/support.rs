use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn unique_temp_root(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_{prefix}_{nanos}"))
}
