use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::ProjectManifestSummary;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StoredRecentProjectEntry {
    pub summary: ProjectManifestSummary,
    pub path: String,
    pub last_opened_unix_ms: u64,
}
