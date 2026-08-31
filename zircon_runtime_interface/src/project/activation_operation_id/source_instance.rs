use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use super::ProjectLaunchInstanceIdError;

/// Stable process-instance component of a project activation operation identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ProjectLaunchInstanceId(Uuid);

impl ProjectLaunchInstanceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn try_from_uuid(value: Uuid) -> Result<Self, ProjectLaunchInstanceIdError> {
        if value.is_nil() {
            return Err(ProjectLaunchInstanceIdError::Nil);
        }
        Ok(Self(value))
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl<'de> Deserialize<'de> for ProjectLaunchInstanceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from_uuid(Uuid::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl Default for ProjectLaunchInstanceId {
    fn default() -> Self {
        Self::new()
    }
}
