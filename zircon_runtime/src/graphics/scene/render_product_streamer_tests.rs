use std::sync::Arc;

use crate::asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, MaterialTextureSlotValue,
    ProjectAssetManager, ShaderAsset, ShaderEntryPointAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset, TextureAsset,
};
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialFallbackReason, RenderMaterialPropertyValue,
    RenderMaterialValidationError, RenderShaderBindGroupLayoutDescriptor,
    RenderShaderBindingDescriptor, RenderShaderBindingResourceType, RenderShaderDefinitionValue,
    RenderShaderPipelineLayoutDescriptor, RenderShaderStage,
};
use crate::core::resource::{
    MaterialMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::backend::RenderBackend;

use super::resources::ResourceStreamer;

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
    assert!(capture.base_color_texture.is_some());
    assert!(capture.normal_texture.is_some());
    assert!(capture.metallic_roughness_texture.is_some());
    assert!(capture.occlusion_texture.is_some());
    assert!(capture.emissive_texture.is_some());
    assert!(material.pipeline_key.double_sided);
    assert!(!material.pipeline_key.alpha_blend);
    assert!(material.pipeline_key.alpha_mask);
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
    assert!(material.readiness_report.is_ready());
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
    assert!(!material.readiness_report.is_ready());
    assert!(material.readiness_report.uses_fallback());
}

#[test]
fn render_product_pbr_streamer_records_missing_material_fallback_runtime() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let missing_material_id = ResourceId::from_stable_label("res://materials/not-registered");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(missing_material_id),
        )
        .expect("missing material uses fallback runtime");

    let report = streamer
        .material_readiness_report(&missing_material_id)
        .expect("fallback material readiness report");
    assert!(!report.is_ready());
    assert!(report.uses_fallback());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedMaterialReference { material }
            if *material == missing_material_id
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Material { material }
            if *material == missing_material_id
    )));
}

#[test]
fn render_product_streamer_material_report_exposes_missing_shader_and_texture_fallbacks() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/streamer-missing.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            missing_refs_material(),
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
        .expect("material prepares with fallback resources");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");

    assert!(!report.is_ready());
    assert!(report.uses_fallback());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedShaderReference { reference }
            if reference.locator == locator("res://shaders/missing-streamer.wgsl")
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedTextureReference { slot, reference }
            if slot == "base_color_texture"
                && reference.locator == locator("res://textures/missing-streamer.png")
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Shader { reference }
            if reference.locator == locator("res://shaders/missing-streamer.wgsl")
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Texture { slot, reference }
            if slot == "base_color_texture"
                && reference.locator == locator("res://textures/missing-streamer.png")
    )));
}

#[test]
fn render_product_streamer_reports_wrong_kind_shader_and_texture_refs() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/wrong-kind.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&locator("res://shaders/not-a-shader.wgsl")),
                ResourceKind::Texture,
                locator("res://shaders/not-a-shader.wgsl"),
            ),
            rgba_texture("res://shaders/not-a-shader.wgsl"),
        )
        .expect("wrong-kind shader texture insert");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&locator("res://textures/not-a-texture.png")),
                ResourceKind::Shader,
                locator("res://textures/not-a-texture.png"),
            ),
            wgsl_shader("res://textures/not-a-texture.png"),
        )
        .expect("wrong-kind texture shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs(
                "res://shaders/not-a-shader.wgsl",
                Some("res://textures/not-a-texture.png"),
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
        .expect("wrong-kind dependencies use fallbacks");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");

    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedShaderReference { reference }
            if reference.locator == locator("res://shaders/not-a-shader.wgsl")
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedTextureReference { slot, reference }
            if slot == "base_color_texture"
                && reference.locator == locator("res://textures/not-a-texture.png")
    )));
}

#[test]
fn render_product_streamer_stores_missing_runtime_wgsl_before_returning_error() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/missing-wgsl.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&locator("res://shaders/source-only.glsl")),
                ResourceKind::Shader,
                locator("res://shaders/source-only.glsl"),
            ),
            glsl_without_runtime_wgsl("res://shaders/source-only.glsl"),
        )
        .expect("shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/source-only.glsl", None),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    let error = streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect_err("missing runtime WGSL blocks material readiness");
    let report = streamer
        .material_readiness_report(&material_id)
        .expect("stored readiness report after blocking error");

    assert!(error.to_string().contains("not render-ready"));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::MissingRuntimeShaderSource
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Shader { reference }
            if reference.locator == locator("res://shaders/source-only.glsl")
    )));
}

#[test]
fn render_product_streamer_repeated_blocking_material_ensure_remains_blocking() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/repeated-missing-wgsl.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&locator("res://shaders/repeated-source-only.glsl")),
                ResourceKind::Shader,
                locator("res://shaders/repeated-source-only.glsl"),
            ),
            glsl_without_runtime_wgsl("res://shaders/repeated-source-only.glsl"),
        )
        .expect("shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/repeated-source-only.glsl", None),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    for _ in 0..2 {
        let error = streamer
            .ensure_material(
                &device,
                &queue,
                &texture_layout,
                ResourceHandle::<MaterialMarker>::new(material_id),
            )
            .expect_err("cached blocking material must remain blocking");
        assert!(error.to_string().contains("not render-ready"));
    }
}

#[test]
fn render_product_streamer_material_report_includes_shader_readiness_diagnostics() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/shader-readiness.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/shader-readiness.zshader");
    let mut shader = wgsl_shader("res://shaders/shader-readiness.zshader");
    shader.entry_points = vec![ShaderEntryPointAsset {
        name: "fs_main".to_string(),
        stage: "pixel".to_string(),
    }];
    shader.shader_defs = vec![
        RenderShaderDefinitionValue::from("USE_UNLIT"),
        RenderShaderDefinitionValue::from(" USE_UNLIT "),
    ];
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri,
            ),
            shader,
        )
        .expect("shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/shader-readiness.zshader", None),
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
        .expect("shader readiness diagnostics are non-blocking material report rows");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");
    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::ShaderReadiness
            && path == "entry_points.fs_main"
            && diagnostic.contains("unsupported stage `pixel`")
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::ShaderReadiness
            && path == "shader_defs.USE_UNLIT"
            && diagnostic.contains("duplicated")
    )));
}

#[test]
fn render_product_streamer_reports_shader_material_layout_abi_diagnostics() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/material-layout-abi.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/material-layout-abi.zshader");
    let mut shader = wgsl_shader("res://shaders/material-layout-abi.zshader");
    shader.pipeline_layout = RenderShaderPipelineLayoutDescriptor {
        bind_groups: vec![RenderShaderBindGroupLayoutDescriptor {
            group: 3,
            label: Some("material".to_string()),
            bindings: vec![
                RenderShaderBindingDescriptor {
                    binding: 0,
                    label: Some("material_texture".to_string()),
                    resource_type: RenderShaderBindingResourceType::Texture,
                    visibility: vec![RenderShaderStage::Fragment],
                },
                RenderShaderBindingDescriptor {
                    binding: 1,
                    label: Some("material_sampler".to_string()),
                    resource_type: RenderShaderBindingResourceType::Sampler,
                    visibility: vec![RenderShaderStage::Fragment],
                },
            ],
        }],
        push_constant_ranges: Vec::new(),
    };
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri,
            ),
            shader,
        )
        .expect("shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/material-layout-abi.zshader", None),
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
        .expect("shader material ABI diagnostics are non-blocking readiness rows");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");
    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::RendererMaterialAbi
            && path == "pipeline_layout.group3.binding0"
            && diagnostic.contains("uniform buffer")
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::RendererMaterialAbi
            && path == "pipeline_layout.group3.binding1"
            && diagnostic.contains("supports only group 3 binding 0")
    )));
}

#[test]
fn render_product_streamer_dependency_readiness_change_invalidates_material_cache() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = locator("res://textures/cache-change.png");
    let texture_id = ResourceId::from_locator(&texture_uri);
    let material_uri = locator("res://materials/cache-change.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            rgba_texture("res://textures/cache-change.png"),
        )
        .expect("texture insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs(
                "builtin://shader/pbr.wgsl",
                Some("res://textures/cache-change.png"),
            ),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager.clone(), &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("initial material prepare");
    assert!(streamer
        .material_readiness_report(&material_id)
        .expect("initial report")
        .validation_errors
        .is_empty());

    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri),
            container_texture("res://textures/cache-change.png"),
        )
        .expect("texture update");
    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("non-blocking texture fallback remains allowed");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("updated report");
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::TextureNotUploadReady { slot, reference, .. }
            if slot == "base_color_texture"
                && reference.locator == locator("res://textures/cache-change.png")
    )));
}

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

fn missing_refs_material() -> MaterialAsset {
    material_with_refs(
        "res://shaders/missing-streamer.wgsl",
        Some("res://textures/missing-streamer.png"),
    )
}

fn material_with_refs(shader_uri: &str, base_color_texture_uri: Option<&str>) -> MaterialAsset {
    MaterialAsset {
        name: Some("StreamerMissingRefs".to_string()),
        shader: asset_reference(shader_uri),
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: base_color_texture_uri.map(asset_reference),
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn pbr_material_with_all_texture_slots() -> MaterialAsset {
    MaterialAsset {
        name: Some("PbrKey".to_string()),
        shader: asset_reference("builtin://shader/pbr.wgsl"),
        base_color: [0.25, 0.5, 0.75, 0.8],
        base_color_texture: Some(asset_reference("res://textures/base.png")),
        normal_texture: Some(asset_reference("res://textures/normal.png")),
        metallic: 0.35,
        roughness: 0.65,
        metallic_roughness_texture: Some(asset_reference("res://textures/mr.png")),
        occlusion_texture: Some(asset_reference("res://textures/occlusion.png")),
        emissive: [0.1, 0.2, 0.3],
        emissive_texture: Some(asset_reference("res://textures/emissive.png")),
        alpha_mode: AlphaMode::Mask { cutoff: 0.42 },
        double_sided: true,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn rgba_texture(uri: &str) -> TextureAsset {
    TextureAsset::new_rgba8(locator(uri), 1, 1, vec![255, 255, 255, 255])
}

fn container_texture(uri: &str) -> TextureAsset {
    TextureAsset::new_container(
        locator(uri),
        1,
        1,
        "bc7_rgba_unorm_srgb",
        vec![1, 2, 3, 4],
        1,
        1,
    )
}

fn wgsl_shader(uri: &str) -> ShaderAsset {
    ShaderAsset {
        uri: locator(uri),
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(1.0); }".to_string(),
        wgsl_source: "".to_string(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        texture_slots: Vec::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn shader_with_texture_slot(uri: &str, slot: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.texture_slots = vec![ShaderTextureSlotAsset {
        name: slot.to_string(),
        kind: "texture2d".to_string(),
        required: false,
        default: Some("white".to_string()),
        sampler: Some("linear_repeat".to_string()),
        group: None,
        label: None,
        editor: Default::default(),
    }];
    shader
}

fn shader_with_property_schema(uri: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.property_schema = vec![
        shader_property("custom_gain", "float", None),
        shader_property("use_rim", "bool", None),
        shader_property(
            "rim_color",
            "vec4",
            Some(toml::Value::Array(vec![
                toml::Value::Float(0.25),
                toml::Value::Float(0.5),
                toml::Value::Float(0.75),
                toml::Value::Float(1.0),
            ])),
        ),
    ];
    shader
}

fn shader_property(
    name: &str,
    kind: &str,
    default: Option<toml::Value>,
) -> crate::asset::ShaderMaterialPropertyAsset {
    crate::asset::ShaderMaterialPropertyAsset {
        name: name.to_string(),
        kind: kind.to_string(),
        required: default.is_none(),
        default,
        editor: Default::default(),
    }
}

fn glsl_without_runtime_wgsl(uri: &str) -> ShaderAsset {
    ShaderAsset {
        uri: locator(uri),
        source_language: ShaderSourceLanguage::Glsl,
        source: "void main() {}".to_string(),
        wgsl_source: "".to_string(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        texture_slots: Vec::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-render-product-streamer-test-texture-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

fn locator(uri: &str) -> AssetUri {
    AssetUri::parse(uri).unwrap()
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(locator(uri))
}
