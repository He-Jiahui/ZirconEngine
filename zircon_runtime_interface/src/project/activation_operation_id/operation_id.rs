use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use super::{
    ProjectActivationOperationIdError, ProjectActivationOperationSequence, ProjectLaunchInstanceId,
};

/// Versioned-launch identity that binds an origin instance, its monotonic sequence, and a nonce.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct ProjectActivationOperationId {
    origin_instance: ProjectLaunchInstanceId,
    sequence: ProjectActivationOperationSequence,
    nonce: Uuid,
}

impl ProjectActivationOperationId {
    pub fn try_from_parts(
        origin_instance: ProjectLaunchInstanceId,
        sequence: ProjectActivationOperationSequence,
        nonce: Uuid,
    ) -> Result<Self, ProjectActivationOperationIdError> {
        if nonce.is_nil() {
            return Err(ProjectActivationOperationIdError::NilNonce);
        }
        Ok(Self {
            origin_instance,
            sequence,
            nonce,
        })
    }

    pub const fn origin_instance(self) -> ProjectLaunchInstanceId {
        self.origin_instance
    }

    pub const fn sequence(self) -> ProjectActivationOperationSequence {
        self.sequence
    }

    pub const fn nonce(self) -> Uuid {
        self.nonce
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectActivationOperationIdWire {
    origin_instance: ProjectLaunchInstanceId,
    sequence: ProjectActivationOperationSequence,
    nonce: Uuid,
}

impl<'de> Deserialize<'de> for ProjectActivationOperationId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProjectActivationOperationIdWire::deserialize(deserializer)?;
        Self::try_from_parts(wire.origin_instance, wire.sequence, wire.nonce)
            .map_err(serde::de::Error::custom)
    }
}
