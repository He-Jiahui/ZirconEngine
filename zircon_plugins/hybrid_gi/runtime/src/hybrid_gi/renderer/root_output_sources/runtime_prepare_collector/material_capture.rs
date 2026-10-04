use std::collections::BTreeMap;

use zircon_runtime::core::framework::render::RenderMeshSnapshot;
use zircon_runtime::core::math::{Vec3, Vec4};
use zircon_runtime::core::resource::ResourceId;
use zircon_runtime::graphics::{RuntimePrepareCollectorContext, RuntimePrepareMaterialCaptureSeed};

use crate::hybrid_gi::renderer::{
    HybridGiMaterialCaptureSeed, HybridGiMaterialCaptureSource, HybridGiMaterialCaptureTextureKey,
};

#[derive(Default)]
pub(super) struct RuntimePrepareMaterialCaptureCache {
    seeds: BTreeMap<ResourceId, HybridGiMaterialCaptureSeed>,
    texture_samples: BTreeMap<HybridGiMaterialCaptureTextureKey, Vec4>,
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
            cache.cache_texture_samples(&seed);
            cache.seeds.insert(
                material_id,
                hybrid_gi_material_capture_seed_from_runtime(seed),
            );
        }
        cache
    }

    fn cache_texture_samples(&mut self, seed: &RuntimePrepareMaterialCaptureSeed) {
        for (texture, sample) in [
            generation_bound_texture_sample(
                seed.base_color_texture,
                seed.base_color_texture_revision,
                seed.base_color_texture_center_rgba,
            ),
            generation_bound_texture_sample(
                seed.normal_texture,
                seed.normal_texture_revision,
                seed.normal_texture_center_rgba,
            ),
            generation_bound_texture_sample(
                seed.metallic_roughness_texture,
                seed.metallic_roughness_texture_revision,
                seed.metallic_roughness_texture_center_rgba,
            ),
            generation_bound_texture_sample(
                seed.occlusion_texture,
                seed.occlusion_texture_revision,
                seed.occlusion_texture_center_rgba,
            ),
            generation_bound_texture_sample(
                seed.emissive_texture,
                seed.emissive_texture_revision,
                seed.emissive_texture_center_rgba,
            ),
        ]
        .into_iter()
        .flatten()
        {
            self.texture_samples.entry(texture).or_insert(sample);
        }
    }
}

impl HybridGiMaterialCaptureSource for RuntimePrepareMaterialCaptureCache {
    fn material_capture_seed(&self, id: &ResourceId) -> Option<HybridGiMaterialCaptureSeed> {
        self.seeds.get(id).copied()
    }

    // 运行时准备阶段固定使用该资源版本的中心样本，忽略传入 UV 以免跨帧读取最新贴图。
    fn sample_texture_rgba(
        &self,
        texture: Option<HybridGiMaterialCaptureTextureKey>,
        _uv: [f32; 2],
    ) -> Option<Vec4> {
        self.texture_samples.get(&texture?).copied()
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
        normal_scale: seed.normal_scale,
        double_sided: seed.double_sided,
        alpha_blend: seed.alpha_blend,
        alpha_cutoff: seed.alpha_cutoff,
        cast_shadows: seed.cast_shadows,
        base_color_texture: generation_bound_texture_key(
            seed.base_color_texture,
            seed.base_color_texture_revision,
        ),
        normal_texture: generation_bound_texture_key(
            seed.normal_texture,
            seed.normal_texture_revision,
        ),
        metallic_roughness_texture: generation_bound_texture_key(
            seed.metallic_roughness_texture,
            seed.metallic_roughness_texture_revision,
        ),
        occlusion_texture: generation_bound_texture_key(
            seed.occlusion_texture,
            seed.occlusion_texture_revision,
        ),
        emissive_texture: generation_bound_texture_key(
            seed.emissive_texture,
            seed.emissive_texture_revision,
        ),
    }
}

fn generation_bound_texture_key(
    id: Option<ResourceId>,
    revision: Option<u64>,
) -> Option<HybridGiMaterialCaptureTextureKey> {
    id.zip(revision)
        .map(|(id, revision)| HybridGiMaterialCaptureTextureKey::new(id, revision))
}

fn generation_bound_texture_sample(
    id: Option<ResourceId>,
    revision: Option<u64>,
    sample: Option<Vec4>,
) -> Option<(HybridGiMaterialCaptureTextureKey, Vec4)> {
    generation_bound_texture_key(id, revision).zip(sample)
}

#[cfg(test)]
#[path = "tests/material_capture.rs"]
mod tests;
