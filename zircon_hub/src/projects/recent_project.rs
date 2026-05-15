use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const RECENT_PROJECT_LIMIT: usize = 8;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProject {
    pub display_name: String,
    pub path: PathBuf,
    pub last_opened_unix_ms: u64,
}

impl RecentProject {
    pub fn new(
        display_name: impl Into<String>,
        path: impl Into<PathBuf>,
        last_opened_unix_ms: u64,
    ) -> Self {
        Self {
            display_name: display_name.into(),
            path: path.into(),
            last_opened_unix_ms,
        }
    }

    pub fn with_now(display_name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self::new(display_name, path, now_unix_ms())
    }
}

pub fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}
