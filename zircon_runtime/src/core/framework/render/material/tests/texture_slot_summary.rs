use super::*;

#[test]
fn material_texture_slot_summary_counts_resolved_and_fallback_slots() {
    let mut slots = BTreeMap::new();
    slots.insert(
        "mask_map".to_string(),
        Some(ResourceId::from_stable_label("texture:mask")),
    );
    slots.insert("detail_map".to_string(), None);

    let summary = RenderMaterialTextureSlotSummary::from_non_standard_slots(&slots);

    assert_eq!(summary.total_count, 2);
    assert_eq!(summary.resolved_count, 1);
    assert_eq!(summary.fallback_count, 1);
}

#[test]
fn material_texture_slot_summary_counts_authored_standard_slot_states() {
    let texture_ids = [
        Some(ResourceId::from_stable_label("texture:base")),
        None,
        Some(ResourceId::from_stable_label("texture:normal")),
    ];

    let summary = RenderMaterialTextureSlotSummary::from_texture_ids(&texture_ids);

    assert_eq!(summary.total_count, 3);
    assert_eq!(summary.resolved_count, 2);
    assert_eq!(summary.fallback_count, 1);
}

#[test]
fn material_texture_slot_state_lists_slot_keys_and_resolution_state() {
    let mut slots = BTreeMap::new();
    let detail_id = ResourceId::from_stable_label("texture:detail");
    slots.insert("mask_map".to_string(), None);
    slots.insert("detail_map".to_string(), Some(detail_id));

    let states = RenderMaterialTextureSlotState::from_non_standard_slots(&slots);

    assert_eq!(
        states,
        vec![
            RenderMaterialTextureSlotState {
                slot: "detail_map".to_string(),
                expected_dimension: None,
                actual_dimension: None,
                texture_id: Some(detail_id),
                fallback: None,
            },
            RenderMaterialTextureSlotState {
                slot: "mask_map".to_string(),
                expected_dimension: None,
                actual_dimension: None,
                texture_id: None,
                fallback: None,
            },
        ]
    );
    assert!(states[0].is_resolved());
    assert!(!states[1].is_resolved());
    assert!(states[1].uses_fallback());
}

#[test]
fn material_texture_slot_state_keeps_fallback_reference_and_reason() {
    let reference = AssetReference::from_locator(
        crate::core::resource::ResourceLocator::parse("res://textures/container.ktx2")
            .expect("valid texture locator"),
    );

    let states = RenderMaterialTextureSlotState::from_resolved_slots([(
        "base_color",
        None,
        Some(RenderMaterialTextureSlotFallback::not_upload_ready(
            reference.clone(),
            "ktx2 texture format or level index is not upload-ready",
        )),
    )]);

    assert_eq!(states.len(), 1);
    assert_eq!(states[0].slot, "base_color");
    assert_eq!(states[0].texture_id, None);
    assert_eq!(
        states[0].fallback,
        Some(RenderMaterialTextureSlotFallback {
            reference,
            reason: RenderMaterialTextureSlotFallbackReason::NotUploadReady {
                detail: "ktx2 texture format or level index is not upload-ready".to_string(),
            },
        })
    );
}

#[test]
fn material_texture_dimension_preserves_cube_array_shader_and_asset_shape() {
    assert_eq!(
        RenderMaterialTextureDimension::from_shader_kind("texture_cube_array"),
        RenderMaterialTextureDimension::CubeArray
    );

    let descriptor = RenderImageDescriptor {
        width: 1,
        height: 1,
        depth_or_array_layers: 12,
        dimension: RenderImageDimension::Cube,
        format: "rgba8unorm".to_string(),
        color_space: crate::core::framework::render::RenderImageColorSpace::Linear,
        metadata: crate::core::framework::render::TextureMetadata::default(),
        sampler: crate::core::framework::render::RenderSamplerDescriptor::default(),
        usage: Vec::new(),
        asset_usage: Vec::new(),
        mip_count: 1,
        fallback: crate::core::framework::render::RenderImageFallbackKind::MissingImage,
    };

    assert_eq!(
        RenderMaterialTextureDimension::from_image_descriptor(&descriptor),
        RenderMaterialTextureDimension::CubeArray
    );
}
