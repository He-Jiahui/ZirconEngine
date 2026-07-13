use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecentProjectValidation {
    #[default]
    Valid,
    Missing,
    InvalidManifest,
    InvalidProject,
}
