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
    assert!(capture.double_sided);
    assert!(!capture.alpha_blend);
    assert_eq!(capture.alpha_cutoff, Some(0.42));
    assert!(!capture.unlit);
    assert_eq!(capture.lighting_model, RenderMaterialLightingModel::Pbr);
    assert_eq!(capture.shading_model_id, SHADING_MODEL_ID_STANDARD_PBR);
    assert!(capture.cast_shadows);
    assert!(material.cast_shadows);
    assert!(capture.receive_shadows);
    assert!(material.receive_shadows);
    assert_eq!(material.render_queue, 0);
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

#[test]
fn render_product_streamer_projects_blinn_phong_shading_model_into_pipeline_key() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/blinn-phong-key.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let mut material = material_with_refs("builtin://shader/pbr.wgsl", None);
    material.property_values.insert(
        "lighting_model".to_string(),
        toml::Value::String("blinn_phong".to_string()),
    );
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

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
    assert_eq!(
        capture.lighting_model,
        RenderMaterialLightingModel::BlinnPhong
    );
    assert_eq!(capture.shading_model_id, SHADING_MODEL_ID_BLINN_PHONG);
    assert_eq!(
        material.pipeline_key.shading_model_id,
        SHADING_MODEL_ID_BLINN_PHONG
    );
    assert!(material.readiness_report.is_ready());
    assert!(material.readiness_report.validation_errors.is_empty());
    assert!(material.readiness_report.fallback_usages.is_empty());
}

fn transform(scale: [f32; 2], offset: [f32; 2]) -> RenderMaterialTextureTransform {
    RenderMaterialTextureTransform { scale, offset }
}

#[test]
fn render_product_pbr_streamer_projects_material_sort_offsets_without_pipeline_variant() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/sort-offsets.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let mut material = material_with_refs("builtin://shader/pbr.wgsl", None);
    material
        .property_values
        .insert("render_queue".to_string(), toml::Value::Integer(-8));
    material
        .property_values
        .insert("material_queue".to_string(), toml::Value::Integer(12));
    material
        .property_values
        .insert("depth_bias".to_string(), toml::Value::Float(-0.75));
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("material prepares");

    let material = streamer.material(&material_id).expect("runtime material");
    assert_eq!(material.render_queue, -8);
    assert_eq!(material.material_queue, 12);
    assert_eq!(material.depth_bias, -0.75);
    assert!(!material.pipeline_key.alpha_blend);
    assert!(!material.pipeline_key.alpha_mask);
    assert!(!material.pipeline_key.double_sided);
    assert!(!material.pipeline_key.has_base_color_texture);
    assert!(material.readiness_report.is_ready());
}

#[test]
fn render_product_pbr_streamer_projects_receive_shadows_override() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/no-shadow-receiver.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let mut material = material_with_refs("builtin://shader/pbr.wgsl", None);
    material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(false));
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

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
    assert!(!material.receive_shadows);
    assert!(!capture.receive_shadows);
    assert!(material.readiness_report.is_ready());
    assert!(material.readiness_report.validation_errors.is_empty());
}

#[test]
fn render_product_pbr_streamer_projects_cast_shadows_override() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/no-shadow-caster.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let mut material = material_with_refs("builtin://shader/pbr.wgsl", None);
    material
        .property_values
        .insert("cast_shadows".to_string(), toml::Value::Boolean(false));
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

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
    assert!(!material.cast_shadows);
    assert!(!capture.cast_shadows);
    assert!(material.receive_shadows);
    assert!(capture.receive_shadows);
    assert!(material.readiness_report.is_ready());
    assert!(material.readiness_report.validation_errors.is_empty());
}

#[test]
fn render_product_pbr_streamer_keeps_authored_texture_key_bits_when_upload_falls_back() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/container-key.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&locator("res://textures/container-key.ktx2")),
                ResourceKind::Texture,
                locator("res://textures/container-key.ktx2"),
            ),
            container_texture("res://textures/container-key.ktx2"),
        )
        .expect("container texture insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs(
                "builtin://shader/pbr.wgsl",
                Some("res://textures/container-key.ktx2"),
            ),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("container texture falls back instead of uploading unsupported bytes");

    let material = streamer.material(&material_id).expect("runtime material");
    assert!(material.pipeline_key.has_base_color_texture);
    assert!(material.base_color_texture.is_none());
    let standard_summary = streamer
        .material_standard_texture_slot_summary(&material_id)
        .expect("standard texture slot summary");
    let readiness_summary = streamer
        .material_readiness_summary(&material_id)
        .expect("readiness summary");
    assert_eq!(
        readiness_summary.standard_texture_slot_summary,
        Some(standard_summary)
    );
    let standard_states = streamer
        .material_standard_texture_slot_states(&material_id)
        .expect("standard texture slot states");
    assert_eq!(standard_states.len(), 1);
    assert_eq!(standard_states[0].slot, "base_color");
    assert_eq!(standard_states[0].texture_id, None);
    assert!(!standard_states[0].is_resolved());
    let standard_fallback = standard_states[0]
        .fallback
        .as_ref()
        .expect("standard fallback detail");
    assert_eq!(
        standard_fallback.reference.locator,
        locator("res://textures/container-key.ktx2")
    );
    assert!(matches!(
        &standard_fallback.reason,
        RenderMaterialTextureSlotFallbackReason::NotUploadReady { detail }
            if detail.contains("upload-ready")
    ));
    assert_eq!(standard_summary.total_count, 1);
    assert_eq!(standard_summary.resolved_count, 0);
    assert_eq!(standard_summary.fallback_count, 1);
    assert!(!material.readiness_report.is_ready());
    assert!(material.readiness_report.uses_fallback());
}

#[test]
fn render_product_streamer_bridges_shader_standard_texture_alias_into_pbr_slot() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/alias-bridge.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/alias-bridge.zshader");
    let legacy_texture_id = ResourceId::from_locator(&locator("res://textures/legacy-base.png"));
    let shader_texture_id = ResourceId::from_locator(&locator("res://textures/shader-albedo.png"));
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri,
            ),
            shader_with_texture_slot("res://shaders/alias-bridge.zshader", "albedo"),
        )
        .expect("shader insert");
    for texture_uri in [
        "res://textures/legacy-base.png",
        "res://textures/shader-albedo.png",
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
    let mut material = material_with_refs(
        "res://shaders/alias-bridge.zshader",
        Some("res://textures/legacy-base.png"),
    );
    material.texture_slots.insert(
        "albedo".to_string(),
        MaterialTextureSlotValue::new(asset_reference("res://textures/shader-albedo.png")),
    );
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader standard texture alias prepares");

    let material = streamer.material(&material_id).expect("runtime material");
    let capture = material.capture_seed();
    assert_eq!(capture.base_color_texture, Some(shader_texture_id));
    assert_ne!(capture.base_color_texture, Some(legacy_texture_id));
    assert!(material.pipeline_key.has_base_color_texture);
    assert!(!material.non_standard_texture_slots.contains_key("albedo"));
    assert!(material.readiness_report.is_ready());
    assert!(material.readiness_report.validation_errors.is_empty());
    assert!(material.readiness_report.fallback_usages.is_empty());
}

#[test]
fn render_product_streamer_shader_standard_alias_shadows_unresolved_legacy_texture() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/alias-shadow.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/alias-shadow.zshader");
    let shader_texture_id =
        ResourceId::from_locator(&locator("res://textures/shader-albedo-shadow.png"));
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri,
            ),
            shader_with_texture_slot("res://shaders/alias-shadow.zshader", "albedo"),
        )
        .expect("shader insert");
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(
                shader_texture_id,
                ResourceKind::Texture,
                locator("res://textures/shader-albedo-shadow.png"),
            ),
            rgba_texture("res://textures/shader-albedo-shadow.png"),
        )
        .expect("shader texture insert");
    let mut material = material_with_refs(
        "res://shaders/alias-shadow.zshader",
        Some("res://textures/missing-legacy-base.png"),
    );
    material.texture_slots.insert(
        "albedo".to_string(),
        MaterialTextureSlotValue::new(asset_reference("res://textures/shader-albedo-shadow.png")),
    );
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader standard texture alias shadows stale legacy texture");

    let material = streamer.material(&material_id).expect("runtime material");
    assert_eq!(material.base_color_texture, Some(shader_texture_id));
    assert!(!material.non_standard_texture_slots.contains_key("albedo"));
    assert!(material.readiness_report.is_ready());
    assert!(material.readiness_report.validation_errors.is_empty());
    assert!(material.readiness_report.fallback_usages.is_empty());
    assert!(material
        .readiness_report
        .dependencies
        .textures
        .iter()
        .all(|reference| reference.locator != locator("res://textures/missing-legacy-base.png")));
}

#[test]
fn render_product_streamer_prepares_shader_texture_slot_runtime_mapping() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/runtime-custom-slot.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/runtime-custom-slot.zshader");
    let texture_uri = locator("res://textures/custom-mask.png");
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_texture_slot("res://shaders/runtime-custom-slot.zshader", "mask_map"),
        )
        .expect("shader insert");
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            rgba_texture("res://textures/custom-mask.png"),
        )
        .expect("texture insert");
    let mut material = material_with_refs("res://shaders/runtime-custom-slot.zshader", None);
    material.texture_slots.insert(
        "mask_map".to_string(),
        MaterialTextureSlotValue::new(asset_reference("res://textures/custom-mask.png")),
    );
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader texture slot prepares");

    let material = streamer.material(&material_id).expect("runtime material");
    assert_eq!(
        material.non_standard_texture_slots.get("mask_map"),
        Some(&Some(texture_id))
    );
    let summary = streamer
        .material_texture_slot_summary(&material_id)
        .expect("texture slot summary");
    let report_summary = streamer
        .material_readiness_report(&material_id)
        .and_then(|report| report.texture_slot_summary)
        .expect("readiness texture slot summary");
    assert_eq!(report_summary, summary);
    assert_eq!(summary.total_count, 1);
    assert_eq!(summary.resolved_count, 1);
    assert_eq!(summary.fallback_count, 0);
    let slot_states = streamer
        .material_texture_slot_states(&material_id)
        .expect("texture slot states");
    let report_slot_states = streamer
        .material_readiness_report(&material_id)
        .map(|report| report.non_standard_texture_slot_states.clone())
        .expect("readiness texture slot states");
    assert_eq!(report_slot_states, slot_states);
    assert_eq!(slot_states.len(), 1);
    assert_eq!(slot_states[0].slot, "mask_map");
    assert_eq!(slot_states[0].texture_id, Some(texture_id));
    assert!(slot_states[0].is_resolved());
    let readiness_summary = streamer
        .material_readiness_summary(&material_id)
        .expect("readiness summary");
    assert_eq!(readiness_summary.texture_slot_summary, Some(summary));
    assert!(readiness_summary.is_ready);
    assert!(!readiness_summary.uses_fallback);
    assert_eq!(readiness_summary.validation_error_count, 0);
    assert_eq!(readiness_summary.fallback_usage_count, 0);
    assert_eq!(readiness_summary.diagnostic_count, 0);
    assert!(material.readiness_report.is_ready());
}

#[test]
fn render_product_streamer_prepares_shader_property_runtime_values() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/runtime-properties.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/runtime-properties.zshader");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_property_schema("res://shaders/runtime-properties.zshader"),
        )
        .expect("shader insert");
    let mut material = material_with_refs("res://shaders/runtime-properties.zshader", None);
    material
        .property_values
        .insert("custom_gain".to_string(), toml::Value::Float(2.5));
    material
        .property_values
        .insert("use_rim".to_string(), toml::Value::Boolean(true));
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader property values prepare");

    let material = streamer.material(&material_id).expect("runtime material");
    assert_eq!(
        material.shader_property_values.get("custom_gain"),
        Some(&RenderMaterialPropertyValue::Float { value: 2.5 })
    );
    assert_eq!(
        material.shader_property_values.get("use_rim"),
        Some(&RenderMaterialPropertyValue::Bool { value: true })
    );
    assert_eq!(
        material.shader_property_values.get("rim_color"),
        Some(&RenderMaterialPropertyValue::Vec4 {
            value: [0.25, 0.5, 0.75, 1.0],
        })
    );
    assert!(material.readiness_report.is_ready());
}
