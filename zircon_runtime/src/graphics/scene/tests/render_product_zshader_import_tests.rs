use std::sync::Arc;

use crate::asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, ProjectAssetManager, ShaderAsset,
    ShaderSourceLanguage,
};
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderPipelineLayoutDescriptor, RenderShaderStage,
    ShaderAssetKind,
};
use crate::core::resource::{
    MaterialMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::backend::RenderBackend;

use super::resources::ResourceStreamer;

#[test]
fn render_product_streamer_reports_shader_material_layout_abi_diagnostics() {
    let shader_uri = AssetUri::parse("res://shaders/imported_layout_shader").unwrap();
    let shader = shader_with_incompatible_material_layout(&shader_uri);
    assert_eq!(
        shader.pipeline_layout.bind_groups[0].bindings[0].resource_type,
        RenderShaderBindingResourceType::Texture
    );

    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = AssetUri::parse("res://materials/imported-layout-abi.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader,
        )
        .expect("shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_for_shader(&shader_uri),
        )
        .expect("material insert");

    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);
    streamer
        .ensure_material(
            &backend,
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("imported zshader material ABI diagnostics are non-blocking readiness rows");

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
            && path == "pipeline_layout.group2.binding0"
            && diagnostic.contains("UniformBuffer")
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::RendererMaterialAbi
            && path.starts_with("pipeline_layout.group2.binding")
            && diagnostic.contains("clearcoat-normal texture")
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::RendererMaterialAbi
            && path.starts_with("pipeline_layout.group2.binding")
            && diagnostic.contains("clearcoat-normal sampler")
    )));
}

fn shader_with_incompatible_material_layout(shader_uri: &AssetUri) -> ShaderAsset {
    ShaderAsset {
        uri: shader_uri.clone(),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: r#"
@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0);
}
"#
        .to_string(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        options: Vec::new(),
        texture_slots: Vec::new(),
        shading_model: Some("unlit".to_string()),
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: Default::default(),
        pipeline_layout: RenderShaderPipelineLayoutDescriptor {
            push_constant_ranges: Vec::new(),
            bind_groups: vec![RenderShaderBindGroupLayoutDescriptor {
                group: 2,
                label: Some("material".to_string()),
                bindings: vec![RenderShaderBindingDescriptor {
                    binding: 0,
                    label: Some("material_texture".to_string()),
                    resource_type: RenderShaderBindingResourceType::Texture,
                    visibility: vec![RenderShaderStage::Fragment],
                }],
            }],
        },
        validation_diagnostics: Vec::new(),
    }
}

fn material_for_shader(shader_uri: &AssetUri) -> MaterialAsset {
    MaterialAsset {
        name: Some("ImportedLayoutMaterial".to_string()),
        shader: AssetReference::from_locator(shader_uri.clone()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
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

fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-render-product-zshader-import-test-texture-layout"),
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
