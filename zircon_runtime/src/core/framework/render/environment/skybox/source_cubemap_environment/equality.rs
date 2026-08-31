use super::SourceCubemapEnvironment;

// Upload bytes are derived submission cache, not environment content identity.
impl PartialEq for SourceCubemapEnvironment {
    fn eq(&self, other: &Self) -> bool {
        self.mip_chain == other.mip_chain
            && self.irradiance_sh9 == other.irradiance_sh9
            && self.irradiance_cube == other.irradiance_cube
            && self.pmrem_hash == other.pmrem_hash
            && self.bake_artifact_hash == other.bake_artifact_hash
            && self.accepted_bake_artifact_descriptor == other.accepted_bake_artifact_descriptor
            && self.intensity == other.intensity
            && self.rotation_radians == other.rotation_radians
            && self.source_revision == other.source_revision
            && self.source_hash == other.source_hash
    }
}
