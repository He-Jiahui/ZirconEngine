use std::collections::BTreeMap;

use zircon_runtime::core::framework::render::RenderMeshSnapshot;
use zircon_runtime::core::math::{Vec3, Vec4};
use zircon_runtime::core::resource::ResourceId;
use zircon_runtime::graphics::{RuntimePrepareCollectorContext, RuntimePrepareMaterialCaptureSeed};

use crate::hybrid_gi::renderer::{HybridGiMaterialCaptureSeed, HybridGiMaterialCaptureSource};

#[derive(Default)]
pub(super) struct RuntimePrepareMaterialCaptureCache {
    seeds: BTreeMap<ResourceId, HybridGiMaterialCaptureSeed>,
    texture_samples: BTreeMap<ResourceId, Vec4>,
}

impl RuntimePrepareMaterialCaptureCache {
    pub(super) fn from_context(
        context: &RuntimePrepareCollectorContext<'_>,
        scene_meshes: &[RenderMeshSnapshot],
    ) -> Self {
        let mut cache = Self::default();
        for mesh in scene_meshes {
            let material_id = mesh.material.id();
            if cache.seeds.contains_key(&material_id) {
                continue;
            }
            let Some(seed) = context.material_capture_seed(&material_id) else {
                continue;
            };
            cache.cache_texture_samples(context, &seed);
            cache.seeds.insert(
                material_id,
                hybrid_gi_material_capture_seed_from_runtime(seed),
            );
        }
        cache
    }

    fn cache_texture_samples(
        &mut self,
        context: &RuntimePrepareCollectorContext<'_>,
        seed: &RuntimePrepareMaterialCaptureSeed,
    ) {
        for texture_id in [
            seed.base_color_texture,
            seed.normal_texture,
            seed.metallic_roughness_texture,
            seed.occlusion_texture,
            seed.emissive_texture,
        ]
        .into_iter()
        .flatten()
        {
            if self.texture_samples.contains_key(&texture_id) {
                continue;
            }
            if let Some(sample) = context.sample_texture_rgba(Some(texture_id), [0.5, 0.5]) {
                self.texture_samples.insert(texture_id, sample);
            }
        }
    }
}

impl HybridGiMaterialCaptureSource for RuntimePrepareMaterialCaptureCache {
    fn material_capture_seed(&self, id: &ResourceId) -> Option<HybridGiMaterialCaptureSeed> {
        self.seeds.get(id).copied()
    }

    fn sample_texture_rgba(&self, id: Option<ResourceId>, _uv: [f32; 2]) -> Option<Vec4> {
        self.texture_samples.get(&id?).copied()
    }
}

fn hybrid_gi_material_capture_seed_from_runtime(
    seed: RuntimePrepareMaterialCaptureSeed,
) -> HybridGiMaterialCaptureSeed {
    HybridGiMaterialCaptureSeed {
        base_color: seed.base_color,
        emissive: seed.emissive,
        metallic: seed.metallic,
        roughness: seed.roughness,
        occlusion_strength: seed.occlusion_strength,
        double_sided: seed.double_sided,
        alpha_blend: seed.alpha_blend,
        alpha_cutoff: seed.alpha_cutoff,
        cast_shadows: seed.cast_shadows,
        base_color_texture: seed.base_color_texture,
        normal_texture: seed.normal_texture,
        metallic_roughness_texture: seed.metallic_roughness_texture,
        occlusion_texture: seed.occlusion_texture,
        emissive_texture: seed.emissive_texture,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_returns_seed_and_center_texture_sample() {
        let material_id = ResourceId::from_stable_label("res://materials/cache.mat");
        let texture_id = ResourceId::from_stable_label("res://textures/cache.png");
        let mut cache = RuntimePrepareMaterialCaptureCache::default();
        cache.seeds.insert(
            material_id,
            HybridGiMaterialCaptureSeed {
                base_color: Vec4::ONE,
                emissive: Vec3::ZERO,
                metallic: 0.0,
                roughness: 1.0,
                occlusion_strength: 0.25,
                double_sided: false,
                alpha_blend: false,
                alpha_cutoff: None,
                cast_shadows: true,
                base_color_texture: Some(texture_id),
                normal_texture: None,
                metallic_roughness_texture: None,
                occlusion_texture: None,
                emissive_texture: None,
            },
        );
        cache
            .texture_samples
            .insert(texture_id, Vec4::new(0.25, 0.5, 0.75, 1.0));

        assert_eq!(
            cache
                .material_capture_seed(&material_id)
                .unwrap()
                .base_color_texture,
            Some(texture_id)
        );
        assert_eq!(
            cache
                .material_capture_seed(&material_id)
                .unwrap()
                .occlusion_strength,
            0.25
        );
        assert_eq!(
            cache.sample_texture_rgba(Some(texture_id), [0.25, 0.75]),
            Some(Vec4::new(0.25, 0.5, 0.75, 1.0))
        );
    }
}
