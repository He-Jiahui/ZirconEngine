use crate::asset::TextureUploadSupport;

use super::super::super::prepared::{PreparedMaterialBundle, PreparedMaterialCandidateIdentity};
use super::super::ResourceStreamer;
use super::material_readiness::{
    prepared_material_cache_identity_is_current, prepared_material_candidate_identity_is_current,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PreparedMaterialCacheSlot {
    Published,
    Staged,
    RejectedStaged,
    RejectedCandidate,
}

impl ResourceStreamer {
    pub(super) fn prepared_material_bundle_cache_is_current(
        &self,
        prepared: &PreparedMaterialBundle,
        requested_revision: Option<u64>,
        texture_support: TextureUploadSupport,
    ) -> bool {
        prepared_material_cache_identity_is_current(
            prepared.revision,
            requested_revision,
            &prepared.material_dependency,
            prepared.texture_support,
            texture_support,
            &prepared.shader_dependency,
            &prepared.texture_dependencies,
            |id| self.material_dependency_identity_for_id(id),
            |locator| self.shader_dependency_identity_for_locator(locator),
            |locator| self.texture_dependency_revision_for_locator(locator),
        )
    }

    pub(super) fn prepared_material_candidate_cache_is_current(
        &self,
        identity: &PreparedMaterialCandidateIdentity,
        requested_revision: Option<u64>,
        texture_support: TextureUploadSupport,
    ) -> bool {
        prepared_material_candidate_identity_is_current(
            identity,
            requested_revision,
            texture_support,
            |id| self.material_dependency_identity_for_id(id),
            |locator| self.shader_dependency_identity_for_locator(locator),
            |locator| self.texture_dependency_revision_for_locator(locator),
        )
    }
}
