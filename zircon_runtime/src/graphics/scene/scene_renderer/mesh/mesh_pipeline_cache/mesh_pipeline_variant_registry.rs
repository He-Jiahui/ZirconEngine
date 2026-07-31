use std::collections::HashMap;

use crate::core::framework::render::{
    GEOMETRY_SOURCE_ID_STATIC_MESH, GeometrySourceId, ShaderPassType, ShaderQualityTier,
    ShaderVariantKey, ShaderVariantMissReport,
};
use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshPassPipelineKind, MeshPipelineVariantId,
};

const FIRST_CACHE_PIPELINE_VARIANT_ID: u32 = 1;
const DEFAULT_MESH_SHADER_VARIANT_PLATFORM_TOKEN: &str = "wgpu-runtime";

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
    ) -> Self {
        let mut shader_variant_key = pipeline_key.shader_variant_key_for_geometry(
            shader_pass_type_for_mesh_pipeline_kind(kind),
            geometry_source,
            DEFAULT_MESH_SHADER_VARIANT_PLATFORM_TOKEN,
        );
        shader_variant_key.quality = shader_quality;
        Self {
            kind,
            pipeline_key: pipeline_key.clone(),
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
    miss_report: ShaderVariantMissReport,
}

pub(crate) trait MeshPipelineVariantResolver {
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
        let key = MeshPipelineVariantKey::new(kind, pipeline_key, geometry_source, shader_quality);
        if let Some(id) = self.variant_ids.get(&key) {
            self.miss_report.record_memory_hit(key.shader_variant_key());
            return *id;
        }

        self.miss_report.record_request(key.shader_variant_key());
        let id = MeshPipelineVariantId::new(
            FIRST_CACHE_PIPELINE_VARIANT_ID + self.variant_keys.len() as u32,
        );
        self.variant_keys.push(key.clone());
        self.variant_ids.insert(key, id);
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

    pub(crate) fn miss_report(&self) -> ShaderVariantMissReport {
        self.miss_report.clone()
    }

    pub(crate) fn reset_miss_report(&mut self) {
        self.miss_report = ShaderVariantMissReport::default();
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

    pub(crate) fn record_compile_miss(&mut self, key: &ShaderVariantKey) {
        self.miss_report.record_compile_miss(key);
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.variant_keys.len()
    }
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
    }
}

impl MeshPipelineVariantResolver for MeshPipelineVariantRegistry {
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
mod tests {
    use crate::core::framework::render::{
        GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
        SHADING_MODEL_ID_STANDARD_PBR, ShaderFeatureBits, ShaderPassType, ShaderQualityTier,
    };
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        MeshPassPipelineKind, MeshPipelineVariantId,
    };

    use super::MeshPipelineVariantRegistry;

    #[test]
    fn mesh_pipeline_variant_registry_reuses_pass_pipeline_shape_id() {
        let mut registry = MeshPipelineVariantRegistry::default();
        let key = default_pipeline_key();

        let first =
            registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
        let second =
            registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);

        assert_eq!(first, second);
        assert_eq!(registry.len(), 1);
        assert_eq!(
            registry.key_for_variant(first).map(|key| key.kind()),
            Some(MeshPassPipelineKind::Base)
        );
    }

    #[test]
    fn mesh_pipeline_variant_registry_separates_pass_and_pipeline_shape() {
        let mut registry = MeshPipelineVariantRegistry::default();
        let base_key = default_pipeline_key();
        let mut alpha_key = base_key.clone();
        alpha_key.alpha_mask = true;

        let base = registry.resolve_variant(
            MeshPassPipelineKind::Base,
            &base_key,
            ShaderQualityTier::Medium,
        );
        let alpha = registry.resolve_variant(
            MeshPassPipelineKind::Base,
            &alpha_key,
            ShaderQualityTier::Medium,
        );
        let shadow = registry.resolve_variant(
            MeshPassPipelineKind::ShadowDepth,
            &base_key,
            ShaderQualityTier::Medium,
        );

        assert_ne!(base, MeshPipelineVariantId::new(0));
        assert_ne!(base, alpha);
        assert_ne!(base, shadow);
        assert_ne!(alpha, shadow);
        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn mesh_pipeline_variant_registry_derives_material_shader_variant_key() {
        let mut registry = MeshPipelineVariantRegistry::default();
        let mut key = default_pipeline_key();
        key.alpha_mask = true;

        let variant = registry.resolve_variant_for_geometry(
            MeshPassPipelineKind::ShadowDepthAlphaMask,
            &key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
            ShaderQualityTier::Medium,
        );
        let variant_key = registry.key_for_variant(variant).unwrap();
        let shader_variant = variant_key.shader_variant_key();

        assert_eq!(shader_variant.pass_type, ShaderPassType::Shadow);
        assert_eq!(
            shader_variant.geometry_source,
            GEOMETRY_SOURCE_ID_SKINNED_MESH
        );
        assert_eq!(shader_variant.shading_model, SHADING_MODEL_ID_STANDARD_PBR);
        assert!(
            shader_variant
                .features
                .contains(ShaderFeatureBits::ALPHA_TEST)
        );
        assert!(
            shader_variant
                .canonical_string()
                .contains("|platform=wgpu-runtime")
        );
    }

    #[test]
    fn mesh_pipeline_variant_registry_separates_shader_quality_tiers() {
        let mut registry = MeshPipelineVariantRegistry::default();
        let key = default_pipeline_key();

        let medium =
            registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
        let high =
            registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::High);

        assert_ne!(medium, high);
        assert_eq!(registry.len(), 2);
        assert_eq!(
            registry
                .key_for_variant(high)
                .map(|key| key.shader_variant_key().quality),
            Some(ShaderQualityTier::High)
        );
    }

    #[test]
    fn mesh_pipeline_variant_registry_separates_geometry_sources() {
        let mut registry = MeshPipelineVariantRegistry::default();
        let key = default_pipeline_key();

        let static_variant = registry.resolve_variant_for_geometry(
            MeshPassPipelineKind::Base,
            &key,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            ShaderQualityTier::Medium,
        );
        let skinned_variant = registry.resolve_variant_for_geometry(
            MeshPassPipelineKind::Base,
            &key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
            ShaderQualityTier::Medium,
        );

        assert_ne!(static_variant, skinned_variant);
        assert_eq!(registry.len(), 2);
        assert_eq!(
            registry
                .key_for_variant(skinned_variant)
                .map(|key| key.shader_variant_key().geometry_source),
            Some(GEOMETRY_SOURCE_ID_SKINNED_MESH)
        );
    }

    #[test]
    fn mesh_pipeline_variant_registry_maps_depth_prepass_to_depth_prepass_pass_type() {
        let mut registry = MeshPipelineVariantRegistry::default();
        let key = default_pipeline_key();

        let variant = registry.resolve_variant(
            MeshPassPipelineKind::DepthPrepass,
            &key,
            ShaderQualityTier::Medium,
        );
        let variant_key = registry.key_for_variant(variant).unwrap();

        assert_eq!(
            variant_key.shader_variant_key().pass_type,
            ShaderPassType::DepthPrepass
        );
    }

    #[test]
    fn mesh_pipeline_variant_registry_maps_deferred_gbuffer_to_gbuffer_pass_type() {
        let mut registry = MeshPipelineVariantRegistry::default();
        let key = default_pipeline_key();

        let variant = registry.resolve_variant_for_geometry(
            MeshPassPipelineKind::GBuffer,
            &key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
            ShaderQualityTier::High,
        );
        let variant_key = registry.key_for_variant(variant).unwrap();

        assert_eq!(variant_key.kind(), MeshPassPipelineKind::GBuffer);
        assert_eq!(
            variant_key.shader_variant_key().pass_type,
            ShaderPassType::GBuffer
        );
        assert_eq!(
            variant_key.shader_variant_key().geometry_source,
            GEOMETRY_SOURCE_ID_SKINNED_MESH
        );
        assert_eq!(
            variant_key.shader_variant_key().quality,
            ShaderQualityTier::High
        );
    }

    #[test]
    fn mesh_pipeline_variant_registry_maps_taa_reactive_to_taa_reactive_pass_type() {
        let mut registry = MeshPipelineVariantRegistry::default();
        let key = default_pipeline_key();

        let reactive_variant = registry.resolve_variant(
            MeshPassPipelineKind::TaaReactiveMask,
            &key,
            ShaderQualityTier::Medium,
        );
        let material_variant = registry.resolve_variant(
            MeshPassPipelineKind::TaaReactiveMaterialMask,
            &key,
            ShaderQualityTier::Medium,
        );
        let reactive_key = registry
            .key_for_variant(reactive_variant)
            .expect("TAA reactive mask variant key");
        let material_key = registry
            .key_for_variant(material_variant)
            .expect("TAA reactive material mask variant key");

        assert_eq!(
            reactive_key.shader_variant_key().pass_type,
            ShaderPassType::TaaReactiveMask
        );
        assert_eq!(
            material_key.shader_variant_key().pass_type,
            ShaderPassType::TaaReactiveMask
        );
        assert_ne!(reactive_variant, material_variant);
    }

    #[test]
    fn mesh_pipeline_variant_registry_counts_variant_misses_and_memory_hits() {
        let mut registry = MeshPipelineVariantRegistry::default();
        let key = default_pipeline_key();

        let first =
            registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
        let second =
            registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
        assert_eq!(first, second);

        let report = registry.miss_report();
        assert_eq!(report.request_count, 2);
        assert_eq!(report.compile_miss_count, 0);
        assert_eq!(report.memory_hit_count, 1);

        let variant_key = registry
            .key_for_variant(first)
            .expect("base variant key")
            .shader_variant_key()
            .clone();
        registry.record_compile_miss(&variant_key);
        assert_eq!(registry.miss_report().compile_miss_count, 1);

        registry.reset_miss_report();
        assert_eq!(registry.miss_report(), Default::default());
    }
}
