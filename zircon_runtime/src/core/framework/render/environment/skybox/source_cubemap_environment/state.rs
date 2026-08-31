use crate::core::math::Real;

use super::super::super::{
    IblBakeArtifactDescriptor, SourceCubemapIrradianceCube, SourceCubemapIrradianceSh9,
    SourceCubemapMipChain, SourceCubemapUploadArtifact,
};
use super::SourceCubemapUploadKey;

#[derive(Clone, Debug)]
pub struct SourceCubemapEnvironment {
    pub mip_chain: SourceCubemapMipChain,
    pub irradiance_sh9: SourceCubemapIrradianceSh9,
    pub irradiance_cube: Option<SourceCubemapIrradianceCube>,
    /// Cached identity of the PMREM section supplied by a bake artifact.
    pub pmrem_hash: [u32; 4],
    pub bake_artifact_hash: [u32; 4],
    pub(in super::super) accepted_bake_artifact_descriptor: Option<IblBakeArtifactDescriptor>,
    pub intensity: Real,
    pub rotation_radians: Real,
    pub source_revision: u64,
    pub source_hash: [u32; 4],
    pub(in super::super) upload_artifact:
        Option<(SourceCubemapUploadKey, SourceCubemapUploadArtifact)>,
}

impl SourceCubemapEnvironment {
    pub fn new(
        mip_chain: SourceCubemapMipChain,
        source_revision: u64,
        source_hash: [u32; 4],
    ) -> Self {
        let irradiance_sh9 = *mip_chain.irradiance_sh9();
        Self {
            mip_chain,
            irradiance_sh9,
            irradiance_cube: None,
            pmrem_hash: [0; 4],
            bake_artifact_hash: [0; 4],
            accepted_bake_artifact_descriptor: None,
            intensity: 1.0,
            rotation_radians: 0.0,
            source_revision,
            source_hash,
            upload_artifact: None,
        }
    }

    pub fn irradiance_cube(&self) -> Option<&SourceCubemapIrradianceCube> {
        self.irradiance_cube.as_ref()
    }
}
