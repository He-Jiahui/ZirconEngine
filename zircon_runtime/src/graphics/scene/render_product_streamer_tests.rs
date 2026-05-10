use std::sync::Arc;

use crate::asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, ProjectAssetManager, ShaderAsset,
    ShaderSourceLanguage, TextureAsset, TexturePayload,
};
use crate::core::framework::render::{RenderMaterialFallbackReason, RenderMaterialValidationError};
use crate::core::resource::{
    MaterialMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::backend::RenderBackend;

use super::resources::ResourceStreamer;

#[test]
fn render_product_streamer_material_report_exposes_missing_shader_and_texture_fallbacks() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/streamer-missing.material.toml");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            missing_refs_material(),
        )
        .expect("material insert");
    let mut streamer = ResourceStreamer::new(asset_manager, &device, &queue, &texture_layout);

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
    let material_uri = locator("res://materials/wrong-kind.material.toml");
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
    let mut streamer = ResourceStreamer::new(asset_manager, &device, &queue, &texture_layout);

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
    let material_uri = locator("res://materials/missing-wgsl.material.toml");
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
    let mut streamer = ResourceStreamer::new(asset_manager, &device, &queue, &texture_layout);

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
    let material_uri = locator("res://materials/repeated-missing-wgsl.material.toml");
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
    let mut streamer = ResourceStreamer::new(asset_manager, &device, &queue, &texture_layout);

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
fn render_product_streamer_dependency_readiness_change_invalidates_material_cache() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = locator("res://textures/cache-change.png");
    let texture_id = ResourceId::from_locator(&texture_uri);
    let material_uri = locator("res://materials/cache-change.material.toml");
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
        ResourceStreamer::new(asset_manager.clone(), &device, &queue, &texture_layout);

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
    let material_uri = locator("res://materials/container-texture.material.toml");
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
    let mut streamer = ResourceStreamer::new(asset_manager, &device, &queue, &texture_layout);

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
    }
}

fn rgba_texture(uri: &str) -> TextureAsset {
    TextureAsset {
        uri: locator(uri),
        width: 1,
        height: 1,
        rgba: vec![255, 255, 255, 255],
        payload: TexturePayload::Rgba8,
    }
}

fn container_texture(uri: &str) -> TextureAsset {
    TextureAsset {
        uri: locator(uri),
        width: 1,
        height: 1,
        rgba: Vec::new(),
        payload: TexturePayload::Container {
            format: "bc7_rgba_unorm_srgb".to_string(),
            bytes: vec![1, 2, 3, 4],
            mip_count: 1,
            array_layers: 1,
        },
    }
}

fn wgsl_shader(uri: &str) -> ShaderAsset {
    ShaderAsset {
        uri: locator(uri),
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(1.0); }".to_string(),
        wgsl_source: "".to_string(),
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn glsl_without_runtime_wgsl(uri: &str) -> ShaderAsset {
    ShaderAsset {
        uri: locator(uri),
        source_language: ShaderSourceLanguage::Glsl,
        source: "void main() {}".to_string(),
        wgsl_source: "".to_string(),
        entry_points: Vec::new(),
        dependencies: Vec::new(),
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
