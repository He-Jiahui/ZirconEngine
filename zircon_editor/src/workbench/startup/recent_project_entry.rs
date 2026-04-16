use serde::{Deserialize, Serialize};

use super::recent_project_validation::RecentProjectValidation;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProjectEntry {
    pub display_name: String,
    pub path: String,
    pub last_opened_unix_ms: u64,
    #[serde(default)]
    pub validation: RecentProjectValidation,
}
