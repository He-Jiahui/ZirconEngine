use serde::{Deserialize, Serialize};
use zircon_runtime_interface::hub_protocol::HubRecentProjectV1;
use zircon_runtime_interface::project::ProjectManifestSummary;

use super::RecentProjectValidation;

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
    pub(crate) fn from_shared(
        project: HubRecentProjectV1,
        validation: RecentProjectValidation,
    ) -> Self {
        Self {
            summary: project.summary,
            path: project.path.to_string_lossy().into_owned(),
            last_opened_unix_ms: project.last_opened_unix_ms,
            validation,
        }
    }
}
