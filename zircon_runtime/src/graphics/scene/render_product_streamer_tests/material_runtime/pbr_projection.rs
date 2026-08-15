use super::*;
use crate::core::framework::render::RenderMaterialTextureTransform;

#[test]
fn render_product_pbr_streamer_projects_standard_material_into_runtime_key() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/pbr-key.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    for texture_uri in [
        "res://textures/base.png",
        "res://textures/normal.png",
        "res://textures/mr.png",
        "res://textures/occlusion.png",
        "res://textures/emissive.png",
    ] {
        asset_manager
            .assets::<TextureAsset>()
            .insert(
                ResourceRecord::new(
                    ResourceId::from_locator(&locator(texture_uri)),
                    ResourceKind::Texture,
                    locator(texture_uri),
                ),
                rgba_texture(texture_uri),
            )
            .expect("texture insert");
    }
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            pbr_material_with_all_texture_slots(),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    let unprepared_capture = streamer
        .material_capture_seed(&material_id)
        .expect("asset-backed material capture seed");
    assert_eq!(unprepared_capture.occlusion_strength, 0.25);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("material prepares");

    let material = streamer.material(&material_id).expect("runtime material");
    let capture = material.capture_seed();
    assert_eq!(capture.base_color.to_array(), [0.25, 0.5, 0.75, 0.8]);
    assert_eq!(capture.emissive.to_array(), [0.1, 0.2, 0.3]);
    assert_eq!(capture.metallic, 0.35);
    assert_eq!(capture.roughness, 0.65);
    assert_eq!(capture.occlusion_strength, 0.25);
    assert_eq!(material.occlusion_strength, 0.25);
    assert!(capture.double_sided);
    assert!(!capture.alpha_blend);
    assert_eq!(capture.alpha_cutoff, Some(0.42));
    assert_eq!(material.alpha_cutoff, Some(0.42));
    assert!(!capture.unlit);
    assert_eq!(capture.lighting_model, RenderMaterialLightingModel::Pbr);
    assert_eq!(capture.shading_model_id, SHADING_MODEL_ID_STANDARD_PBR);
    assert!(capture.cast_shadows);
    assert!(material.cast_shadows);
    assert!(capture.receive_shadows);
    assert!(material.receive_shadows);
    assert!(material.pipeline_key.receive_shadows);
    assert_eq!(material.render_queue, 0);
    assert_eq!(material.render_queue_value, None);
    assert_eq!(material.material_queue, 0);
    assert_eq!(material.depth_bias, 0.0);
    assert!(capture.base_color_texture.is_some());
    assert!(capture.normal_texture.is_some());
    assert!(capture.metallic_roughness_texture.is_some());
    assert!(capture.occlusion_texture.is_some());
    assert!(capture.emissive_texture.is_some());
    assert_eq!(
        capture.base_color_texture_transform,
        transform([2.0, 2.0], [0.125, 0.25])
    );
    assert_eq!(capture.base_color_texture_uv_channel, 1);
    assert_eq!(
        capture.normal_texture_transform,
        transform([1.0, 1.0], [0.0, 0.0])
    );
    assert_eq!(capture.normal_texture_uv_channel, 0);
    assert_eq!(
        capture.metallic_roughness_texture_transform,
        transform([0.5, 0.5], [0.5, 0.0])
    );
    assert_eq!(capture.metallic_roughness_texture_uv_channel, 1);
    assert_eq!(
        capture.occlusion_texture_transform,
        transform([3.0, 4.0], [0.0, 0.75])
    );
    assert_eq!(capture.occlusion_texture_uv_channel, 1);
    assert_eq!(
        capture.emissive_texture_transform,
        transform([1.5, 1.25], [0.25, 0.125])
    );
    assert_eq!(capture.emissive_texture_uv_channel, 0);
    assert_eq!(
        material.base_color_texture_transform,
        capture.base_color_texture_transform
    );
    assert_eq!(
        material.base_color_texture_uv_channel,
        capture.base_color_texture_uv_channel
    );
    assert_eq!(
        material.normal_texture_transform,
        capture.normal_texture_transform
    );
    assert_eq!(
        material.normal_texture_uv_channel,
        capture.normal_texture_uv_channel
    );
    assert_eq!(
        material.metallic_roughness_texture_transform,
        capture.metallic_roughness_texture_transform
    );
    assert_eq!(
        material.metallic_roughness_texture_uv_channel,
        capture.metallic_roughness_texture_uv_channel
    );
    assert_eq!(
        material.occlusion_texture_transform,
        capture.occlusion_texture_transform
    );
    assert_eq!(
        material.occlusion_texture_uv_channel,
        capture.occlusion_texture_uv_channel
    );
    assert_eq!(
        material.emissive_texture_transform,
        capture.emissive_texture_transform
    );
    assert_eq!(
        material.emissive_texture_uv_channel,
        capture.emissive_texture_uv_channel
    );
    assert!(material.pipeline_key.double_sided);
    assert!(!material.pipeline_key.alpha_blend);
    assert!(material.pipeline_key.alpha_mask);
    assert!(material.pipeline_key.receive_shadows);
    assert_eq!(
        material.pipeline_key.shading_model_id,
        SHADING_MODEL_ID_STANDARD_PBR
    );
    assert_eq!(
        material.pipeline_key.alpha_cutoff_bits,
        Some(0.42f32.to_bits())
    );
    assert!(material.pipeline_key.has_base_color_texture);
    assert!(material.pipeline_key.has_normal_texture);
    assert!(material.pipeline_key.has_metallic_roughness_texture);
    assert!(material.pipeline_key.has_occlusion_texture);
    assert!(material.pipeline_key.has_emissive_texture);
    assert!(!material.pipeline_key.is_transparent());
    assert!(material.non_standard_texture_slots.is_empty());
    let standard_summary = streamer
        .material_standard_texture_slot_summary(&material_id)
        .expect("standard texture slot summary");
    let report_standard_summary = streamer
        .material_readiness_report(&material_id)
        .and_then(|report| report.standard_texture_slot_summary)
        .expect("readiness standard texture slot summary");
    let readiness_summary = streamer
        .material_readiness_summary(&material_id)
        .expect("readiness summary");
    assert_eq!(report_standard_summary, standard_summary);
    assert_eq!(
        readiness_summary.standard_texture_slot_summary,
        Some(standard_summary)
    );
    let standard_states = streamer
        .material_standard_texture_slot_states(&material_id)
        .expect("standard texture slot states");
    let report_standard_states = streamer
        .material_readiness_report(&material_id)
        .map(|report| report.standard_texture_slot_states.clone())
        .expect("readiness standard texture slot states");
    assert_eq!(report_standard_states, standard_states);
    assert_eq!(standard_states.len(), 5);
    assert!(standard_states.iter().all(|state| state.is_resolved()));
    assert_eq!(standard_states[0].slot, "base_color");
    assert_eq!(standard_states[1].slot, "normal");
    assert_eq!(standard_states[2].slot, "metallic_roughness");
    assert_eq!(standard_states[3].slot, "occlusion");
    assert_eq!(standard_states[4].slot, "emissive");
    assert_eq!(standard_summary.total_count, 5);
    assert_eq!(standard_summary.resolved_count, 5);
    assert_eq!(standard_summary.fallback_count, 0);
    assert!(
        material.readiness_report.is_ready(),
        "readiness report: {:?}",
        material.readiness_report
    );
}

fn transform(scale: [f32; 2], offset: [f32; 2]) -> RenderMaterialTextureTransform {
    RenderMaterialTextureTransform { scale, offset }
}
