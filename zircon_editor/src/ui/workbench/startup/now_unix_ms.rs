use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// recent显示用wall clock，早于epoch回零；不能用于单调计时或代次排序。
pub(crate) fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}
