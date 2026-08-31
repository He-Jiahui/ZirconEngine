use super::*;
use crate::core::framework::render::RenderMaterialTextureTransform;

#[test]
fn render_product_pbr_streamer_projects_standard_material_into_runtime_key() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
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
            &backend,
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("material prepares");
    assert!(
        streamer.material_capture_seed(&material_id).is_none(),
        "a staged material must not leak into capture before pipeline admission"
    );
    assert!(streamer.publish_staged_material_candidate(material_id));

    let material = streamer.material(&material_id).expect("runtime material");
    let capture = streamer
        .material_capture_seed(&material_id)
        .expect("published material capture seed");
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
    assert!(material.pipeline_key.has_normal_texture);
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

#[test]
fn cold_material_capture_inherits_parent_values_through_canonical_resolution() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let parent_uri = locator("res://materials/capture-parent.zmaterial");
    let parent_id = ResourceId::from_locator(&parent_uri);
    let child_uri = locator("res://materials/capture-child.zmaterial");
    let child_id = ResourceId::from_locator(&child_uri);
    let mut parent = material_with_refs("builtin://shader/pbr.wgsl", None);
    parent
        .property_values
        .insert("roughness".to_string(), toml::Value::Float(0.23));
    let mut child = material_with_refs("builtin://shader/pbr.wgsl", None);
    child.parent = Some(AssetReference::from_locator(parent_uri.clone()));

    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(parent_id, ResourceKind::Material, parent_uri),
            parent,
        )
        .expect("parent material insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(child_id, ResourceKind::Material, child_uri),
            child,
        )
        .expect("child material insert");
    let streamer = ResourceStreamer::new_for_test(asset_manager, device, queue, &texture_layout);

    let capture = streamer
        .material_capture_seed(&child_id)
        .expect("cold effective material capture seed");

    assert_eq!(capture.roughness, 0.23);
}

#[test]
fn published_material_capture_retains_texture_revision_and_sample_until_bundle_publication() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = locator("res://textures/capture-generation.png");
    let texture_id = ResourceId::from_locator(&texture_uri);
    let material_uri = locator("res://materials/capture-generation.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            TextureAsset::new_rgba8(texture_uri.clone(), 1, 1, vec![255, 255, 255, 255]),
        )
        .expect("initial texture insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs(
                "builtin://shader/pbr.wgsl",
                Some("res://textures/capture-generation.png"),
            ),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager.clone(), device, queue, &texture_layout);

    streamer
        .ensure_material(
            &backend,
            device,
            queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("initial material prepare");
    assert!(streamer.publish_staged_material_candidate(material_id));
    let initial = streamer
        .material_capture_seed(&material_id)
        .expect("initial published capture seed");
    let initial_revision = initial
        .base_color_texture_revision
        .expect("initial capture texture revision");
    assert_eq!(
        initial.base_color_texture_center_rgba,
        Some(crate::core::math::Vec4::ONE)
    );

    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            TextureAsset::new_rgba8(texture_uri, 1, 1, vec![0, 0, 0, 255]),
        )
        .expect("texture hot reload");
    streamer
        .ensure_texture(&backend, &texture_layout, texture_id)
        .expect("new texture generation prepares");
    let retained = streamer
        .material_capture_seed(&material_id)
        .expect("last-good material capture seed");
    assert_eq!(retained.base_color_texture_revision, Some(initial_revision));
    assert_eq!(
        retained.base_color_texture_center_rgba,
        Some(crate::core::math::Vec4::ONE)
    );

    streamer
        .ensure_material(
            &backend,
            device,
            queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("replacement material stages");
    let staged = streamer
        .material_capture_seed(&material_id)
        .expect("published material remains capture authority");
    assert_eq!(staged.base_color_texture_revision, Some(initial_revision));
    assert_eq!(
        staged.base_color_texture_center_rgba,
        Some(crate::core::math::Vec4::ONE)
    );

    assert!(streamer.publish_staged_material_candidate(material_id));
    let published = streamer
        .material_capture_seed(&material_id)
        .expect("replacement published capture seed");
    assert_ne!(
        published.base_color_texture_revision,
        Some(initial_revision)
    );
    assert_eq!(
        published.base_color_texture_center_rgba,
        Some(crate::core::math::Vec4::new(0.0, 0.0, 0.0, 1.0))
    );
}

#[test]
fn render_product_streamer_preserves_clearcoat_normal_slot_metadata() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = locator("res://textures/clearcoat-normal.png");
    let texture_id = ResourceId::from_locator(&texture_uri);
    let material_uri = locator("res://materials/clearcoat-normal.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            rgba_texture("res://textures/clearcoat-normal.png"),
        )
        .expect("clearcoat normal texture insert");
    let mut material = material_with_refs("builtin://shader/pbr.wgsl", None);
    material
        .property_values
        .insert("clearcoat".to_string(), toml::Value::Float(0.8));
    material.property_values.insert(
        "clearcoat_normal_scale".to_string(),
        toml::Value::Float(0.35),
    );
    let mut clearcoat_slot =
        MaterialTextureSlotValue::new(asset_reference("res://textures/clearcoat-normal.png"));
    clearcoat_slot.transform = Some(RenderMaterialTextureTransform {
        scale: [0.5, 0.75],
        offset: [0.1, 0.2],
        rotation: 0.4,
    });
    clearcoat_slot.uv_channel = 1;
    material
        .texture_slots
        .insert("clearcoat_normal".to_string(), clearcoat_slot);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("clearcoat material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, device, queue, &texture_layout);

    streamer
        .ensure_material(
            &backend,
            device,
            queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("clearcoat material prepares");

    let runtime = streamer
        .staged_material_candidate(&material_id)
        .expect("clearcoat material stages");
    assert_eq!(runtime.clearcoat_normal_texture, Some(texture_id));
    assert_eq!(runtime.advanced_features.clearcoat_normal_scale, 0.35);
    assert_eq!(runtime.clearcoat_normal_texture_uv_channel, 1);
    assert_eq!(
        runtime.clearcoat_normal_texture_transform,
        RenderMaterialTextureTransform {
            scale: [0.5, 0.75],
            offset: [0.1, 0.2],
            rotation: 0.4,
        }
    );
}

fn transform(scale: [f32; 2], offset: [f32; 2]) -> RenderMaterialTextureTransform {
    RenderMaterialTextureTransform {
        scale,
        offset,
        rotation: 0.0,
    }
}
