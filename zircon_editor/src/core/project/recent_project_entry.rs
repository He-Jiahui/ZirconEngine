use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::ProjectManifestSummary;

use super::{RecentProjectValidation, StoredRecentProjectEntry};

/// Recent-project identity projected from the authoritative project manifest summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProjectEntry {
    pub summary: ProjectManifestSummary,
    pub path: String,
    pub last_opened_unix_ms: u64,
    #[serde(default)]
    pub validation: RecentProjectValidation,
}

impl RecentProjectEntry {
    pub(crate) fn into_stored(self) -> StoredRecentProjectEntry {
        StoredRecentProjectEntry {
            summary: self.summary,
            path: self.path,
            last_opened_unix_ms: self.last_opened_unix_ms,
        }
    }
}
