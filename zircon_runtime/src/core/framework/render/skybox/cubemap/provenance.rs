use super::super::super::IblBakeArtifactDescriptor;
use super::SourceCubemapEnvironment;

impl SourceCubemapEnvironment {
    /// Records artifact provenance without changing GPU texture content identity.
    pub fn with_bake_artifact_hash(mut self, bake_artifact_hash: [u32; 4]) -> Self {
        self.bake_artifact_hash = bake_artifact_hash;
        self
    }

    /// The descriptor that produced all active artifact-backed environment sections.
    ///
    /// It is provenance only and deliberately remains outside the GPU upload key.
    pub fn accepted_bake_artifact_descriptor(&self) -> Option<IblBakeArtifactDescriptor> {
        self.accepted_bake_artifact_descriptor
    }

    pub(in super::super::super) fn with_accepted_bake_artifact_descriptor(
        mut self,
        descriptor: IblBakeArtifactDescriptor,
    ) -> Self {
        self.accepted_bake_artifact_descriptor = Some(descriptor);
        self
    }
}
