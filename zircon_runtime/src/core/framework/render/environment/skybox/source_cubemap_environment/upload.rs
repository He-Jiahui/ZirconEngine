use super::super::super::{
    build_source_cubemap_upload_artifact, SourceCubemapIrradianceCube, SourceCubemapUploadArtifact,
};
use super::SourceCubemapEnvironment;

impl SourceCubemapEnvironment {
    pub fn with_irradiance_cube(mut self, irradiance_cube: SourceCubemapIrradianceCube) -> Self {
        let upload_key = self.texture_upload_key();
        self.irradiance_cube = Some(irradiance_cube);
        if self.texture_upload_key() != upload_key {
            // Drop outdated pre-encoded rows before a replacement artifact is built.
            self.upload_artifact = None;
            self.accepted_bake_artifact_descriptor = None;
        }
        self
    }

    /// Builds immutable, mip-major RGBA16F bytes before the render submission path consumes them.
    pub fn with_prepared_upload_artifact(mut self) -> Self {
        if self.prepared_upload_artifact().is_some() {
            return self;
        }
        let upload_key = self.texture_upload_key();
        let artifact =
            build_source_cubemap_upload_artifact(&self.mip_chain, self.irradiance_cube.as_ref());
        self.upload_artifact = Some((upload_key, artifact));
        self
    }

    pub fn prepared_upload_artifact(&self) -> Option<&SourceCubemapUploadArtifact> {
        let (upload_key, artifact) = self.upload_artifact.as_ref()?;
        (*upload_key == self.texture_upload_key()).then_some(artifact)
    }

    pub(in super::super::super) fn discard_prepared_upload_artifact(&mut self) {
        self.upload_artifact = None;
    }
}
