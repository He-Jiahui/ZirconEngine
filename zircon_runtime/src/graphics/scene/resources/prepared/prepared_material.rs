use std::sync::Arc;

use crate::asset::TextureUploadSupport;
use crate::core::framework::render::RenderMaterialReadinessReport;
use crate::core::resource::{ResourceId, ResourceLocator};

use super::super::{
    GpuMaterialUniformResource, GpuTextureResource, MaterialRuntime, OutputTargetTextureResource,
};

pub(in crate::graphics::scene::resources) struct PreparedMaterial {
    /// The complete generation visible to draw construction. `None` means the
    /// engine-owned error proxy remains authoritative while a cold candidate
    /// waits for pipeline admission.
    pub(in crate::graphics::scene::resources) published: Option<PreparedMaterialBundle>,
    /// The immediately preceding published generation, retained as the bounded
    /// context-qualified fallback while the current generation reaches newly
    /// observed graph/quality/fog/geometry contexts.
    pub(in crate::graphics::scene::resources) previous_published: Option<PreparedMaterialBundle>,
    pub(in crate::graphics::scene::resources) staged_candidate: Option<PreparedMaterialBundle>,
    pub(in crate::graphics::scene::resources) staged_pipeline_failed: bool,
    pub(in crate::graphics::scene::resources) staged_pipeline_admission_cycle:
        StagedMaterialPipelineAdmissionCycle,
    pub(in crate::graphics::scene::resources) rejected_candidate:
        Option<RejectedPreparedMaterialCandidate>,
}

/// Aggregates admission across every camera submitted for one viewport.
///
/// A deferred requirement from any camera prevents the shared material
/// generation from becoming draw-visible at the viewport-terminal boundary.
#[derive(Default)]
pub(in crate::graphics::scene::resources) struct StagedMaterialPipelineAdmissionCycle {
    observed: bool,
    deferred: bool,
}

impl StagedMaterialPipelineAdmissionCycle {
    pub(in crate::graphics::scene::resources) fn record(&mut self, deferred: bool) {
        self.observed = true;
        self.deferred |= deferred;
    }

    /// Returns `None` when no draw referenced the candidate, or `Some` with
    /// the all-ready result for the completed viewport cycle.
    pub(in crate::graphics::scene::resources) fn finish(&mut self) -> Option<bool> {
        let result = self.observed.then_some(!self.deferred);
        *self = Self::default();
        result
    }
}

/// The fields that become draw-visible as one material generation.
///
/// Keeping cache identity, runtime state, textures, and both uniform bindings in one owner
/// prevents a staged publication or context fallback from mixing generations.
pub(in crate::graphics::scene::resources) struct PreparedMaterialBundle {
    /// Process-local identity for the complete draw-visible bundle.
    pub(in crate::graphics::scene::resources) draw_generation: u64,
    pub(in crate::graphics::scene::resources) revision: Option<u64>,
    pub(in crate::graphics::scene::resources) material_dependency: PreparedMaterialDependency,
    pub(in crate::graphics::scene::resources) shader_dependency: PreparedMaterialShaderDependency,
    pub(in crate::graphics::scene::resources) texture_dependencies:
        Vec<PreparedMaterialTextureDependency>,
    pub(in crate::graphics::scene::resources) texture_support: TextureUploadSupport,
    pub(in crate::graphics::scene::resources) runtime: MaterialRuntime,
    pub(in crate::graphics::scene::resources) textures: PreparedMaterialTextureSet,
    pub(in crate::graphics::scene::resources) uniform: Arc<GpuMaterialUniformResource>,
    pub(in crate::graphics::scene::resources) standard_uniform: Arc<GpuMaterialUniformResource>,
}

impl PreparedMaterialBundle {
    pub(in crate::graphics::scene::resources) fn candidate_identity(
        &self,
    ) -> PreparedMaterialCandidateIdentity {
        PreparedMaterialCandidateIdentity::new(
            self.revision,
            self.material_dependency,
            &self.shader_dependency,
            &self.texture_dependencies,
            self.texture_support,
        )
    }
}

/// Identifies the material asset that produced a prepared bundle, including its parent closure.
///
/// `id` can differ from the requested draw material when the engine-owned missing-material
/// fallback is used. `dependency_revision` is maintained by the resource registry's reverse
/// dependency graph, so stable-frame validation does not traverse the parent chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::resources) struct PreparedMaterialDependency {
    pub(in crate::graphics::scene::resources) id: ResourceId,
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) dependency_revision: u64,
}

/// Lightweight identity retained for a failed candidate.
///
/// It suppresses repeated preparation only while the complete material,
/// shader, texture, and device-capability inputs remain unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::resources) struct PreparedMaterialCandidateIdentity {
    pub(in crate::graphics::scene::resources) revision: Option<u64>,
    pub(in crate::graphics::scene::resources) material_dependency: PreparedMaterialDependency,
    pub(in crate::graphics::scene::resources) shader_dependency: PreparedMaterialShaderDependency,
    pub(in crate::graphics::scene::resources) texture_dependencies:
        Vec<PreparedMaterialTextureDependency>,
    pub(in crate::graphics::scene::resources) texture_support: TextureUploadSupport,
}

impl PreparedMaterialCandidateIdentity {
    pub(in crate::graphics::scene::resources) fn new(
        revision: Option<u64>,
        material_dependency: PreparedMaterialDependency,
        shader_dependency: &PreparedMaterialShaderDependency,
        texture_dependencies: &[PreparedMaterialTextureDependency],
        texture_support: TextureUploadSupport,
    ) -> Self {
        Self {
            revision,
            material_dependency,
            shader_dependency: shader_dependency.clone(),
            texture_dependencies: texture_dependencies.to_vec(),
            texture_support,
        }
    }
}

#[derive(Clone)]
pub(in crate::graphics::scene::resources) enum PreparedMaterialTextureResource {
    Texture(Arc<GpuTextureResource>),
    OutputTarget(Arc<OutputTargetTextureResource>),
}

/// One generation's texture resource, plus the revision needed to accept mip-residency updates.
///
/// A same-revision texture replacement is a mip-streaming update and may be consumed immediately.
/// A different revision belongs to another material generation, so draw selection retains
/// `resource` until that generation is selected as a unit.
#[derive(Clone)]
pub(in crate::graphics::scene::resources) struct PreparedMaterialTextureBinding {
    pub(in crate::graphics::scene::resources) id: Option<ResourceId>,
    pub(in crate::graphics::scene::resources) revision: Option<u64>,
    pub(in crate::graphics::scene::resources) capture_sample_rgba: Option<[f32; 4]>,
    pub(in crate::graphics::scene::resources) resource: PreparedMaterialTextureResource,
}

#[derive(Clone)]
pub(in crate::graphics::scene::resources) struct PreparedMaterialTextureSet {
    pub(in crate::graphics::scene::resources) base_color: PreparedMaterialTextureBinding,
    pub(in crate::graphics::scene::resources) normal: PreparedMaterialTextureBinding,
    pub(in crate::graphics::scene::resources) metallic_roughness: PreparedMaterialTextureBinding,
    pub(in crate::graphics::scene::resources) occlusion: PreparedMaterialTextureBinding,
    pub(in crate::graphics::scene::resources) emissive: PreparedMaterialTextureBinding,
    pub(in crate::graphics::scene::resources) clearcoat_normal: PreparedMaterialTextureBinding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::resources) struct PreparedMaterialShaderDependency {
    pub(in crate::graphics::scene::resources) locator: ResourceLocator,
    pub(in crate::graphics::scene::resources) id: Option<ResourceId>,
    pub(in crate::graphics::scene::resources) revision: Option<u64>,
    pub(in crate::graphics::scene::resources) dependency_revision: Option<u64>,
}

pub(in crate::graphics::scene::resources) struct RejectedPreparedMaterialCandidate {
    pub(in crate::graphics::scene::resources) identity: Option<PreparedMaterialCandidateIdentity>,
    pub(in crate::graphics::scene::resources) readiness_report: RenderMaterialReadinessReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::resources) struct PreparedMaterialTextureDependency {
    pub(in crate::graphics::scene::resources) locator: ResourceLocator,
    pub(in crate::graphics::scene::resources) id: Option<ResourceId>,
    pub(in crate::graphics::scene::resources) revision: Option<u64>,
    pub(in crate::graphics::scene::resources) upload_unsupported_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::StagedMaterialPipelineAdmissionCycle;

    #[test]
    fn viewport_cycle_requires_an_observed_all_ready_candidate() {
        let mut cycle = StagedMaterialPipelineAdmissionCycle::default();

        assert_eq!(cycle.finish(), None);
        cycle.record(false);
        cycle.record(false);
        assert_eq!(cycle.finish(), Some(true));
        assert_eq!(cycle.finish(), None);
    }

    #[test]
    fn one_deferred_camera_blocks_the_whole_viewport_cycle() {
        let mut cycle = StagedMaterialPipelineAdmissionCycle::default();

        cycle.record(false);
        cycle.record(true);
        cycle.record(false);
        assert_eq!(cycle.finish(), Some(false));
    }
}
