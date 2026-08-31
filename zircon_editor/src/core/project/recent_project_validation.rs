use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecentProjectValidation {
    Valid,
    RequiresMigration,
    Missing,
    InvalidManifest,
    #[default]
    InvalidProject,
}
