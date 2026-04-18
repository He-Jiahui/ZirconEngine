use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StoredRecentProjectEntry {
    pub display_name: String,
    pub path: String,
    pub last_opened_unix_ms: u64,
}
