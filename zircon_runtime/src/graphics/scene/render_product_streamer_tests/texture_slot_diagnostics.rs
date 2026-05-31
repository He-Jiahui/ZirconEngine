use super::*;

#[test]
fn render_product_streamer_reports_container_textures_as_not_upload_ready() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/container-texture.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&locator("res://textures/container.ktx2")),
                ResourceKind::Texture,
                locator("res://textures/container.ktx2"),
            ),
            container_texture("res://textures/container.ktx2"),
        )
        .expect("container texture insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs(
                "builtin://shader/pbr.wgsl",
                Some("res://textures/container.ktx2"),
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
    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");

    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::TextureNotUploadReady { slot, reference, .. }
            if slot == "base_color_texture"
                && reference.locator == locator("res://textures/container.ktx2")
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Texture { slot, reference }
            if slot == "base_color_texture"
                && reference.locator == locator("res://textures/container.ktx2")
    )));
}

#[test]
fn render_product_streamer_reports_rgba8_descriptor_conversion_fallback_detail() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = locator("res://textures/height-map.png");
    let texture_id = ResourceId::from_locator(&texture_uri);
    let material_uri = locator("res://materials/height-map.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let mut descriptor = TextureAssetDescriptor::rgba8_srgb();
    descriptor.format = "rgba16float".to_string();
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            rgba_texture("res://textures/height-map.png").with_descriptor(descriptor),
        )
        .expect("texture insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs(
                "builtin://shader/pbr.wgsl",
                Some("res://textures/height-map.png"),
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
        .expect("descriptor conversion requirement falls back without blocking material runtime");
    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");
    let expected_reason =
        "rgba8 texture descriptor format rgba16float requires conversion before upload";

    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::TextureNotUploadReady { slot, reference, reason }
            if slot == "base_color_texture"
                && reference.locator == texture_uri
                && reason == expected_reason
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Texture { slot, reference }
            if slot == "base_color_texture"
                && reference.locator == texture_uri
    )));
    let standard_states = streamer
        .material_standard_texture_slot_states(&material_id)
        .expect("standard texture slot states");
    assert_eq!(standard_states.len(), 1);
    assert_eq!(standard_states[0].slot, "base_color");
    assert_eq!(standard_states[0].texture_id, None);
    let fallback = standard_states[0]
        .fallback
        .as_ref()
        .expect("descriptor conversion fallback detail");
    assert_eq!(fallback.reference.locator, texture_uri);
    assert!(matches!(
        &fallback.reason,
        RenderMaterialTextureSlotFallbackReason::NotUploadReady { detail }
            if detail == expected_reason
    ));
    assert_eq!(&report.standard_texture_slot_states, &standard_states);
    assert_eq!(
        report
            .standard_texture_slot_summary
            .expect("standard texture slot summary")
            .fallback_count,
        1
    );
}

#[test]
fn render_product_streamer_reports_compressed_mip_chain_texture_fallback_detail() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = locator("res://textures/mip-chain.astc");
    let texture_id = ResourceId::from_locator(&texture_uri);
    let material_uri = locator("res://materials/mip-chain.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            astc_mip_chain_texture("res://textures/mip-chain.astc"),
        )
        .expect("texture insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs(
                "builtin://shader/pbr.wgsl",
                Some("res://textures/mip-chain.astc"),
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
        .expect("compressed mip-chain requirement falls back without blocking material runtime");
    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");
    let expected_reason = "compressed texture mip-chain upload is not implemented";

    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::TextureNotUploadReady { slot, reference, reason }
            if slot == "base_color_texture"
                && reference.locator == texture_uri
                && reason == expected_reason
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Texture { slot, reference }
            if slot == "base_color_texture"
                && reference.locator == texture_uri
    )));
    let standard_states = streamer
        .material_standard_texture_slot_states(&material_id)
        .expect("standard texture slot states");
    assert_eq!(standard_states.len(), 1);
    assert_eq!(standard_states[0].slot, "base_color");
    assert_eq!(standard_states[0].texture_id, None);
    let fallback = standard_states[0]
        .fallback
        .as_ref()
        .expect("mip-chain fallback detail");
    assert_eq!(fallback.reference.locator, texture_uri);
    assert!(matches!(
        &fallback.reason,
        RenderMaterialTextureSlotFallbackReason::NotUploadReady { detail }
            if detail == expected_reason
    ));
    assert_eq!(&report.standard_texture_slot_states, &standard_states);
    assert_eq!(
        report
            .standard_texture_slot_summary
            .expect("standard texture slot summary")
            .fallback_count,
        1
    );
}

#[test]
fn render_product_streamer_reports_shader_texture_slot_upload_fallback_by_slot_key() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/custom-slot.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/custom-slot.zshader");
    let texture_uri = locator("res://textures/custom-mask.ktx2");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_texture_slot("res://shaders/custom-slot.zshader", "mask_map"),
        )
        .expect("shader insert");
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&texture_uri),
                ResourceKind::Texture,
                texture_uri.clone(),
            ),
            container_texture("res://textures/custom-mask.ktx2"),
        )
        .expect("texture insert");
    let mut material = material_with_refs("res://shaders/custom-slot.zshader", None);
    material.texture_slots.insert(
        "mask_map".to_string(),
        MaterialTextureSlotValue::new(asset_reference("res://textures/custom-mask.ktx2")),
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
        .expect("shader texture slot fallback is non-blocking");
    let material = streamer.material(&material_id).expect("runtime material");
    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");

    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::TextureNotUploadReady { slot, reference, .. }
            if slot == "mask_map"
                && reference.locator == locator("res://textures/custom-mask.ktx2")
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Texture { slot, reference }
            if slot == "mask_map"
                && reference.locator == locator("res://textures/custom-mask.ktx2")
    )));
    assert_eq!(
        material.non_standard_texture_slots.get("mask_map"),
        Some(&None),
        "fallback shader slot keeps the slot key without a prepared texture id"
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
    assert_eq!(summary.resolved_count, 0);
    assert_eq!(summary.fallback_count, 1);
    let slot_states = streamer
        .material_texture_slot_states(&material_id)
        .expect("texture slot states");
    assert_eq!(slot_states.len(), 1);
    assert_eq!(slot_states[0].slot, "mask_map");
    assert_eq!(slot_states[0].texture_id, None);
    assert!(!slot_states[0].is_resolved());
    let slot_fallback = slot_states[0]
        .fallback
        .as_ref()
        .expect("shader texture fallback detail");
    assert_eq!(
        slot_fallback.reference.locator,
        locator("res://textures/custom-mask.ktx2")
    );
    assert!(matches!(
        &slot_fallback.reason,
        RenderMaterialTextureSlotFallbackReason::NotUploadReady { detail }
            if detail.contains("upload-ready")
    ));
    assert_eq!(&report.non_standard_texture_slot_states, &slot_states);
    let readiness_summary = streamer
        .material_readiness_summary(&material_id)
        .expect("readiness summary");
    assert_eq!(readiness_summary.texture_slot_summary, Some(summary));
    assert!(!readiness_summary.is_ready);
    assert!(readiness_summary.uses_fallback);
    assert_eq!(readiness_summary.validation_error_count, 1);
    assert_eq!(readiness_summary.fallback_usage_count, 1);
    assert_eq!(readiness_summary.diagnostic_count, 0);
}

#[test]
fn render_product_streamer_reports_unresolved_shader_texture_slot_by_slot_key() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/missing-custom-slot.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/missing-custom-slot.zshader");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_texture_slot("res://shaders/missing-custom-slot.zshader", "mask_map"),
        )
        .expect("shader insert");
    let mut material = material_with_refs("res://shaders/missing-custom-slot.zshader", None);
    material.texture_slots.insert(
        "mask_map".to_string(),
        MaterialTextureSlotValue::new(asset_reference("res://textures/missing-custom-mask.png")),
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
        .expect("unresolved shader texture slot uses fallback texture");
    let material = streamer.material(&material_id).expect("runtime material");
    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");

    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedTextureReference { slot, reference }
            if slot == "mask_map"
                && reference.locator == locator("res://textures/missing-custom-mask.png")
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Texture { slot, reference }
            if slot == "mask_map"
                && reference.locator == locator("res://textures/missing-custom-mask.png")
    )));
    assert_eq!(
        material.non_standard_texture_slots.get("mask_map"),
        Some(&None),
        "unresolved shader slot keeps the slot key without a prepared texture id"
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
    assert_eq!(summary.resolved_count, 0);
    assert_eq!(summary.fallback_count, 1);
    let slot_states = streamer
        .material_texture_slot_states(&material_id)
        .expect("texture slot states");
    assert_eq!(slot_states.len(), 1);
    assert_eq!(slot_states[0].slot, "mask_map");
    assert_eq!(slot_states[0].texture_id, None);
    assert!(!slot_states[0].is_resolved());
    let slot_fallback = slot_states[0]
        .fallback
        .as_ref()
        .expect("unresolved shader texture fallback detail");
    assert_eq!(
        slot_fallback.reference.locator,
        locator("res://textures/missing-custom-mask.png")
    );
    assert!(matches!(
        &slot_fallback.reason,
        RenderMaterialTextureSlotFallbackReason::UnresolvedReference
    ));
    assert_eq!(&report.non_standard_texture_slot_states, &slot_states);
    let readiness_summary = streamer
        .material_readiness_summary(&material_id)
        .expect("readiness summary");
    assert_eq!(readiness_summary.texture_slot_summary, Some(summary));
    assert!(!readiness_summary.is_ready);
    assert!(readiness_summary.uses_fallback);
    assert_eq!(readiness_summary.validation_error_count, 1);
    assert_eq!(readiness_summary.fallback_usage_count, 1);
    assert_eq!(readiness_summary.diagnostic_count, 0);
}

fn astc_mip_chain_texture(uri: &str) -> TextureAsset {
    TextureAsset::new_container(
        locator(uri),
        4,
        4,
        "astc/4x4x1",
        astc_4x4_level_bytes(),
        2,
        1,
    )
}

fn astc_4x4_level_bytes() -> Vec<u8> {
    let mut bytes = vec![0_u8; 32];
    bytes[0..4].copy_from_slice(b"\x13\xAB\xA1\x5C");
    bytes[4] = 4;
    bytes[5] = 4;
    bytes[6] = 1;
    write_u24_le(&mut bytes, 7, 4);
    write_u24_le(&mut bytes, 10, 4);
    write_u24_le(&mut bytes, 13, 1);
    bytes[16..32].fill(1);
    bytes
}

fn write_u24_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 3].copy_from_slice(&value.to_le_bytes()[..3]);
}
