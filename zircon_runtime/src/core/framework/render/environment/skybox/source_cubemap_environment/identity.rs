use super::super::super::{
    IblBakeArtifactContents, IblBakeArtifactRequest, IblBakeKey, SourceCubemapIrradianceCube,
};
use super::{SourceCubemapEnvironment, SourceCubemapUploadKey};

impl SourceCubemapEnvironment {
    pub fn texture_upload_key(&self) -> SourceCubemapUploadKey {
        SourceCubemapUploadKey {
            source_revision: self.source_revision,
            source_hash: self.source_hash,
            pmrem_hash: self.pmrem_hash,
            irradiance_cube_hash: self
                .irradiance_cube
                .as_ref()
                .map_or([0; 4], SourceCubemapIrradianceCube::content_hash),
        }
    }

    pub fn ibl_bake_key(&self) -> IblBakeKey {
        IblBakeKey::source_cubemap(self.source_revision, self.source_hash)
    }

    pub fn ibl_bake_artifact_request(
        &self,
        required_contents: IblBakeArtifactContents,
    ) -> IblBakeArtifactRequest {
        IblBakeArtifactRequest::new(
            self.ibl_bake_key(),
            self.mip_chain.source_face_size(),
            self.mip_chain.source_mip_count(),
        )
        .with_pmrem_layout(
            self.mip_chain.pmrem_face_size(),
            self.mip_chain.pmrem_mip_count(),
        )
        .with_required_contents(required_contents)
    }
}
