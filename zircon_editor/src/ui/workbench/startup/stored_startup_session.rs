use serde::{Deserialize, Serialize};

use super::stored_recent_project_entry::StoredRecentProjectEntry;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StoredStartupSession {
    pub last_project_path: Option<String>,
    #[serde(default)]
    pub recent_projects: Vec<StoredRecentProjectEntry>,
}
