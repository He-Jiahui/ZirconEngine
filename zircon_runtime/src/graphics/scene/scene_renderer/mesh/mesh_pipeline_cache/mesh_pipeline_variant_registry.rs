use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderPipelineDiagnosticStage,
    ShaderPipelineFallbackAction, ShaderPipelineFallbackState, ShaderPipelineTarget,
    ShaderQualityTier, ShaderVariantKey, ShaderVariantMissReport, GEOMETRY_SOURCE_ID_STATIC_MESH,
    SHADER_PIPELINE_TARGET_COUNT, SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshPassPipelineKind, MeshPipelineVariantId,
};

use super::pipeline_creation_metrics::shader_pipeline_target_for_mesh_kind;

const FIRST_CACHE_PIPELINE_VARIANT_ID: u32 = 1;
const DEFAULT_MESH_SHADER_VARIANT_PLATFORM_TOKEN: &str = "wgpu-runtime";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct MeshPipelineResolverConfigurationEpoch(u64);

impl MeshPipelineResolverConfigurationEpoch {
    fn advance(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeshPipelineVariantKey {
    kind: MeshPassPipelineKind,
    pipeline_key: PipelineKey,
    shader_variant_key: ShaderVariantKey,
}

impl MeshPipelineVariantKey {
    fn new(
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        geometry_source: GeometrySourceId,
        shader_quality: ShaderQualityTier,
        environment_only_pbr_base_profile: bool,
    ) -> Self {
        let pbr_ior_override = pipeline_key.pbr_ior_override;
        let pipeline_key = pipeline_key.pipeline_variant_identity();
        let mut shader_variant_key = pipeline_key.shader_variant_key_for_geometry(
            shader_pass_type_for_mesh_pipeline_kind(kind),
            geometry_source,
            DEFAULT_MESH_SHADER_VARIANT_PLATFORM_TOKEN,
        );
        if environment_only_pbr_base_profile
            && !pbr_ior_override
            && supports_environment_only_pbr_base_profile(kind, &pipeline_key)
        {
            shader_variant_key.features = shader_variant_key.features.union(
                ShaderFeatureBits::new(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR),
            );
        }
        shader_variant_key.quality = shader_quality;
        Self {
            kind,
            pipeline_key,
            shader_variant_key,
        }
    }

    pub(crate) const fn kind(&self) -> MeshPassPipelineKind {
        self.kind
    }

    pub(crate) const fn pipeline_key(&self) -> &PipelineKey {
        &self.pipeline_key
    }

    pub(crate) const fn shader_variant_key(&self) -> &ShaderVariantKey {
        &self.shader_variant_key
    }
}

#[derive(Default)]
pub(crate) struct MeshPipelineVariantRegistry {
    variant_ids: HashMap<MeshPipelineVariantKey, MeshPipelineVariantId>,
    variant_keys: Vec<MeshPipelineVariantKey>,
    registered_shader_variants: HashSet<ShaderVariantKey>,
    registered_pipeline_variants_by_target: [usize; SHADER_PIPELINE_TARGET_COUNT],
    miss_report: ShaderVariantMissReport,
    environment_only_pbr_base_profile: bool,
    configuration_epoch: MeshPipelineResolverConfigurationEpoch,
}

/// 在命令构建阶段登记稳定变体；配置代际用于判断已有命令的解析结果是否可复用。
pub(crate) trait MeshPipelineVariantResolver {
    fn configuration_epoch(&self) -> MeshPipelineResolverConfigurationEpoch {
        MeshPipelineResolverConfigurationEpoch::default()
    }

    fn resolve_variant_for_geometry(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        geometry_source: GeometrySourceId,
        shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId;
}

impl MeshPipelineVariantRegistry {
    pub(crate) fn resolve_variant(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId {
        self.resolve_variant_for_geometry(
            kind,
            pipeline_key,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            shader_quality,
        )
    }

    pub(crate) fn resolve_variant_for_geometry(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        geometry_source: GeometrySourceId,
        shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId {
        let key = MeshPipelineVariantKey::new(
            kind,
            pipeline_key,
            geometry_source,
            shader_quality,
            self.environment_only_pbr_base_profile,
        );
        if let Some(id) = self.variant_ids.get(&key) {
            self.miss_report.record_memory_hit(key.shader_variant_key());
            return *id;
        }

        self.miss_report.record_request(key.shader_variant_key());
        let id = next_pipeline_variant_id(self.variant_keys.len());
        self.registered_shader_variants
            .insert(key.shader_variant_key().clone());
        let target = shader_pipeline_target_for_mesh_kind(kind);
        self.registered_pipeline_variants_by_target[target.index()] =
            self.registered_pipeline_variants_by_target[target.index()].saturating_add(1);
        self.variant_keys.push(key.clone());
        self.variant_ids.insert(key, id);
        self.record_registered_variant_counts();
        id
    }

    pub(crate) fn key_for_variant(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> Option<&MeshPipelineVariantKey> {
        let index = variant_id
            .value()
            .checked_sub(FIRST_CACHE_PIPELINE_VARIANT_ID)? as usize;
        self.variant_keys.get(index)
    }

    /// An unknown variant stays on the generic binding contract. This fails
    /// closed when a caller cannot prove that its shader omits group 1.
    pub(crate) fn base_pipeline_requires_forward_receiver(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> bool {
        self.key_for_variant(variant_id).map_or(true, |key| {
            !key.shader_variant_key()
                .features
                .contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR)
        })
    }

    pub(crate) fn miss_report(&self) -> ShaderVariantMissReport {
        self.miss_report.clone()
    }

    pub(crate) fn reset_miss_report(&mut self) {
        self.miss_report = ShaderVariantMissReport::default();
        self.record_registered_variant_counts();
    }

    pub(crate) fn enable_environment_only_pbr_base_profile(&mut self) {
        if self.environment_only_pbr_base_profile {
            return;
        }
        self.environment_only_pbr_base_profile = true;
        self.configuration_epoch.advance();
    }

    pub(crate) fn disable_environment_only_pbr_base_profile(&mut self) {
        if !self.environment_only_pbr_base_profile {
            return;
        }
        self.environment_only_pbr_base_profile = false;
        self.configuration_epoch.advance();
    }

    pub(crate) const fn environment_only_pbr_base_profile_enabled(&self) -> bool {
        self.environment_only_pbr_base_profile
    }

    pub(crate) const fn configuration_epoch(&self) -> MeshPipelineResolverConfigurationEpoch {
        self.configuration_epoch
    }

    pub(crate) fn record_disk_hit(&mut self, key: &ShaderVariantKey) {
        self.miss_report.record_disk_hit(key);
    }

    pub(crate) fn record_disk_write(&mut self, key: &ShaderVariantKey) {
        self.miss_report.record_disk_write(key);
    }

    pub(crate) fn record_disk_error(&mut self, key: &ShaderVariantKey) {
        self.miss_report.record_disk_error(key);
    }

    pub(crate) fn record_pipeline_diagnostic(
        &mut self,
        key: &ShaderVariantKey,
        stage: ShaderPipelineDiagnosticStage,
        message: impl Into<String>,
    ) {
        self.miss_report
            .record_pipeline_diagnostic(key, stage, message);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_pipeline_fallback(
        &mut self,
        pipeline_variant_id: MeshPipelineVariantId,
        entity_id: u64,
        consumer: &str,
        state: ShaderPipelineFallbackState,
        action: ShaderPipelineFallbackAction,
        reason: &str,
        state_age_microseconds: u64,
    ) {
        let key = pipeline_variant_id
            .value()
            .checked_sub(FIRST_CACHE_PIPELINE_VARIANT_ID)
            .map(|index| index as usize)
            .and_then(|index| self.variant_keys.get(index))
            .map(MeshPipelineVariantKey::shader_variant_key);
        let Some(key) = key else {
            self.miss_report.record_unresolved_pipeline_fallback(
                pipeline_variant_id.value(),
                entity_id,
                consumer,
                state,
                action,
                reason,
                state_age_microseconds,
            );
            return;
        };
        self.miss_report.record_pipeline_fallback(
            key,
            pipeline_variant_id.value(),
            entity_id,
            consumer,
            state,
            action,
            reason,
            state_age_microseconds,
        );
    }

    pub(crate) fn record_compile_miss(&mut self, key: &ShaderVariantKey) {
        self.miss_report.record_compile_miss(key);
    }

    fn record_registered_variant_counts(&mut self) {
        self.miss_report.record_registered_variant_counts(
            self.variant_keys.len(),
            self.registered_shader_variants.len(),
            self.variant_keys.len(),
        );
        for target in ShaderPipelineTarget::ALL {
            self.miss_report
                .record_registered_pipeline_target_variant_count(
                    target,
                    self.registered_pipeline_variants_by_target[target.index()],
                );
        }
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.variant_keys.len()
    }
}

fn next_pipeline_variant_id(variant_count: usize) -> MeshPipelineVariantId {
    let variant_offset =
        u32::try_from(variant_count).expect("mesh pipeline variant count exceeds u32 capacity");
    let value = FIRST_CACHE_PIPELINE_VARIANT_ID
        .checked_add(variant_offset)
        .expect("mesh pipeline variant ID space is exhausted");
    MeshPipelineVariantId::new(value)
}

fn supports_environment_only_pbr_base_profile(
    kind: MeshPassPipelineKind,
    pipeline_key: &PipelineKey,
) -> bool {
    kind == MeshPassPipelineKind::Base
        && pipeline_key.uses_fallback_shader()
        && pipeline_key.shading_model_id == SHADING_MODEL_ID_STANDARD_PBR
        && !pipeline_key.is_transparent()
        && !pipeline_key.is_alpha_mask()
        && !pipeline_key.unlit
        && !pipeline_key.receive_shadows
        && !pipeline_key.pbr_clearcoat
        && !pipeline_key.pbr_anisotropy
        && !pipeline_key.pbr_transmission
        && !pipeline_key.volumetric_fog
}

fn shader_pass_type_for_mesh_pipeline_kind(kind: MeshPassPipelineKind) -> ShaderPassType {
    match kind {
        MeshPassPipelineKind::GBuffer => ShaderPassType::GBuffer,
        MeshPassPipelineKind::DepthPrepass => ShaderPassType::DepthPrepass,
        MeshPassPipelineKind::Base => ShaderPassType::Forward,
        MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask => {
            ShaderPassType::Shadow
        }
        MeshPassPipelineKind::Velocity => ShaderPassType::Velocity,
        MeshPassPipelineKind::TaaReactiveMask | MeshPassPipelineKind::TaaReactiveMaterialMask => {
            ShaderPassType::TaaReactiveMask
        }
        MeshPassPipelineKind::HitProxy => ShaderPassType::HitProxy,
    }
}

impl MeshPipelineVariantResolver for MeshPipelineVariantRegistry {
    fn configuration_epoch(&self) -> MeshPipelineResolverConfigurationEpoch {
        MeshPipelineVariantRegistry::configuration_epoch(self)
    }

    fn resolve_variant_for_geometry(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        geometry_source: GeometrySourceId,
        shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId {
        MeshPipelineVariantRegistry::resolve_variant_for_geometry(
            self,
            kind,
            pipeline_key,
            geometry_source,
            shader_quality,
        )
    }
}

#[cfg(test)]
#[path = "tests/mesh_pipeline_variant_registry.rs"]
mod tests;
