use serde::{Deserialize, Serialize};

use super::{CanonicalDescriptorIdentity, ProjectGuid, ProjectManifestDigest};

/// Versioned project identity accepted by preflight and later admission boundaries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectIdentity {
    canonical_descriptor: CanonicalDescriptorIdentity,
    project_guid: ProjectGuid,
    manifest_digest: ProjectManifestDigest,
}

impl ProjectIdentity {
    pub fn new(
        canonical_descriptor: CanonicalDescriptorIdentity,
        project_guid: ProjectGuid,
        manifest_digest: ProjectManifestDigest,
    ) -> Self {
        Self {
            canonical_descriptor,
            project_guid,
            manifest_digest,
        }
    }

    pub fn canonical_descriptor(&self) -> &CanonicalDescriptorIdentity {
        &self.canonical_descriptor
    }

    pub const fn project_guid(&self) -> ProjectGuid {
        self.project_guid
    }

    pub const fn manifest_digest(&self) -> ProjectManifestDigest {
        self.manifest_digest
    }
}
