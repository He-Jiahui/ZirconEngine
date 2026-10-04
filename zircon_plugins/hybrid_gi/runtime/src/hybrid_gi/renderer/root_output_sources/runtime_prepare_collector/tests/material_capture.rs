use super::*;

#[test]
fn cache_returns_seed_and_center_texture_sample() {
    let material_id = ResourceId::from_stable_label("res://materials/cache.mat");
    let texture_id = ResourceId::from_stable_label("res://textures/cache.png");
    let texture = HybridGiMaterialCaptureTextureKey::new(texture_id, 7);
    let mut cache = RuntimePrepareMaterialCaptureCache::default();
    cache.seeds.insert(
        material_id,
        HybridGiMaterialCaptureSeed {
            base_color: Vec4::ONE,
            emissive: Vec3::ZERO,
            metallic: 0.0,
            roughness: 1.0,
            occlusion_strength: 0.25,
            normal_scale: 1.0,
            double_sided: false,
            alpha_blend: false,
            alpha_cutoff: None,
            cast_shadows: true,
            base_color_texture: Some(texture),
            normal_texture: None,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            emissive_texture: None,
        },
    );
    cache
        .texture_samples
        .insert(texture, Vec4::new(0.25, 0.5, 0.75, 1.0));

    assert_eq!(
        cache
            .material_capture_seed(&material_id)
            .unwrap()
            .base_color_texture,
        Some(texture)
    );
    assert_eq!(
        cache
            .material_capture_seed(&material_id)
            .unwrap()
            .occlusion_strength,
        0.25
    );
    assert_eq!(
        cache.sample_texture_rgba(Some(texture), [0.25, 0.75]),
        Some(Vec4::new(0.25, 0.5, 0.75, 1.0))
    );

    let next_texture = HybridGiMaterialCaptureTextureKey::new(texture_id, 8);
    cache.texture_samples.insert(next_texture, Vec4::ZERO);
    assert_eq!(cache.texture_samples.len(), 2);
}

#[test]
fn runtime_cache_uses_generation_bound_samples_without_latest_asset_reads() {
    let production = include_str!("../material_capture.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("runtime material capture test boundary");

    assert!(production.contains("BTreeMap<HybridGiMaterialCaptureTextureKey, Vec4>"));
    assert!(production.contains("base_color_texture_center_rgba"));
    assert!(production.contains("base_color_texture_revision"));
    assert!(!production.contains("context.sample_texture_rgba"));
}
