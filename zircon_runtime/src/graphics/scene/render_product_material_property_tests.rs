use std::sync::Arc;

use crate::asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, ProjectAssetManager, ShaderAsset,
    ShaderMaterialPropertyAsset, ShaderSourceLanguage,
};
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialPropertyValueSummary,
};
use crate::core::resource::{
    MaterialMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::backend::RenderBackend;

use super::resources::ResourceStreamer;

#[test]
fn render_product_material_properties_prepare_uniform_payload() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/runtime-property-uniform.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/runtime-property-uniform.zshader");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_property_schema("res://shaders/runtime-property-uniform.zshader"),
        )
        .expect("shader insert");
    let mut material = material_with_shader("res://shaders/runtime-property-uniform.zshader");
    material
        .property_values
        .insert("custom_gain".to_string(), toml::Value::Float(2.5));
    material
        .property_values
        .insert("use_rim".to_string(), toml::Value::Boolean(true));
    material.property_values.insert(
        "debug_label".to_string(),
        toml::Value::String("author-only".to_string()),
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
        .expect("shader property values prepare");

    let material = streamer.material(&material_id).expect("runtime material");
    let payload = &material.shader_property_uniform_payload;
    assert_eq!(payload.layout.len(), 3);
    assert_eq!(payload.layout[0].name, "custom_gain");
    assert_eq!(payload.layout[0].offset, 0);
    assert_eq!(f32_at(&payload.bytes, 0), 2.5);
    assert_eq!(payload.layout[1].name, "rim_color");
    assert_eq!(payload.layout[1].offset, 16);
    assert_eq!(f32_at(&payload.bytes, 16), 0.25);
    assert_eq!(f32_at(&payload.bytes, 20), 0.5);
    assert_eq!(f32_at(&payload.bytes, 24), 0.75);
    assert_eq!(f32_at(&payload.bytes, 28), 1.0);
    assert_eq!(payload.layout[2].name, "use_rim");
    assert_eq!(payload.layout[2].offset, 32);
    assert_eq!(u32_at(&payload.bytes, 32), 1);
    assert_eq!(payload.bytes.len(), 48);
    assert_eq!(payload.unsupported.len(), 1);
    assert_eq!(payload.unsupported[0].name, "debug_label");
    assert_eq!(
        streamer.material_uniform_payload_byte_len(&material_id),
        Some(48)
    );
    assert_eq!(
        streamer.material_uniform_buffer_byte_len(&material_id),
        Some(64)
    );
}

#[test]
fn render_product_streamer_exposes_material_uniform_debug_counts() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/runtime-property-uniform-counts.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/runtime-property-uniform-counts.zshader");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_property_schema("res://shaders/runtime-property-uniform-counts.zshader"),
        )
        .expect("shader insert");
    let mut material =
        material_with_shader("res://shaders/runtime-property-uniform-counts.zshader");
    material
        .property_values
        .insert("custom_gain".to_string(), toml::Value::Float(2.5));
    material
        .property_values
        .insert("use_rim".to_string(), toml::Value::Boolean(true));
    material.property_values.insert(
        "debug_label".to_string(),
        toml::Value::String("author-only".to_string()),
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

    assert_eq!(streamer.material_uniform_field_count(&material_id), None);
    assert_eq!(
        streamer.material_uniform_unsupported_count(&material_id),
        None
    );
    assert_eq!(streamer.material_uniform_summary(&material_id), None);
    assert!(streamer.material(&material_id).is_none());

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader property debug counts prepare");

    assert_eq!(streamer.material_uniform_field_count(&material_id), Some(3));
    assert_eq!(
        streamer.material_uniform_unsupported_count(&material_id),
        Some(1)
    );
    let summary = streamer
        .material_uniform_summary(&material_id)
        .expect("material uniform summary");
    assert_eq!(summary.payload_byte_len, 48);
    assert_eq!(summary.field_count, 3);
    assert_eq!(summary.unsupported_count, 1);
    let report_summary = streamer
        .material_readiness_report(&material_id)
        .and_then(|report| report.uniform_summary)
        .expect("material readiness uniform summary");
    assert_eq!(report_summary, summary);
    let material = streamer.material(&material_id).expect("runtime material");
    let value_summary =
        RenderMaterialPropertyValueSummary::from_values(&material.shader_property_values);
    assert_eq!(value_summary.total_count, 4);
    assert_eq!(value_summary.uniform_eligible_count(), 3);
    assert_eq!(value_summary.non_uniform_count(), 1);
    assert_eq!(value_summary.float_count, 1);
    assert_eq!(value_summary.bool_count, 1);
    assert_eq!(value_summary.vec4_count, 1);
    assert_eq!(value_summary.string_count, 1);
}

#[test]
fn render_product_streamer_reports_material_uniform_diagnostics_in_readiness_report() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/runtime-property-uniform-report.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/runtime-property-uniform-report.zshader");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_property_schema("res://shaders/runtime-property-uniform-report.zshader"),
        )
        .expect("shader insert");
    let mut material =
        material_with_shader("res://shaders/runtime-property-uniform-report.zshader");
    material
        .property_values
        .insert("custom_gain".to_string(), toml::Value::Float(2.5));
    material
        .property_values
        .insert("use_rim".to_string(), toml::Value::Boolean(true));
    material.property_values.insert(
        "debug_label".to_string(),
        toml::Value::String("author-only".to_string()),
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
        .expect("shader property readiness prepare");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("material readiness report");
    assert!(report.is_ready());
    assert!(report.validation_errors.is_empty());
    assert!(report.fallback_usages.is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].source,
        RenderMaterialDiagnosticSource::MaterialUniform
    );
    assert_eq!(report.diagnostics[0].path, "uniform.debug_label");
    assert_eq!(
        report.diagnostics[0].diagnostic,
        "material property debug_label cannot be encoded into the renderer uniform payload: unsupported property type"
    );
}

#[test]
fn render_product_streamer_reports_material_uniform_diagnostics_for_shader_string_defaults() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/runtime-property-uniform-default.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/runtime-property-uniform-default.zshader");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_string_default_schema(
                "res://shaders/runtime-property-uniform-default.zshader",
            ),
        )
        .expect("shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_shader("res://shaders/runtime-property-uniform-default.zshader"),
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
        .expect("shader property default readiness prepare");

    let material = streamer.material(&material_id).expect("runtime material");
    assert_eq!(material.shader_property_uniform_payload.layout.len(), 1);
    assert_eq!(
        material.shader_property_uniform_payload.unsupported[0].name,
        "debug_label"
    );

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("material readiness report");
    assert!(report.is_ready());
    assert!(report.validation_errors.is_empty());
    assert!(report.fallback_usages.is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].source,
        RenderMaterialDiagnosticSource::MaterialUniform
    );
    assert_eq!(report.diagnostics[0].path, "uniform.debug_label");
}

fn material_with_shader(shader_uri: &str) -> MaterialAsset {
    MaterialAsset {
        name: None,
        shader: asset_reference(shader_uri),
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 0.5,
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

fn shader_with_property_schema(uri: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.property_schema = vec![
        shader_property("custom_gain", "float", None),
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
        shader_property("use_rim", "bool", None),
        shader_property("debug_label", "string", None),
    ];
    shader
}

fn shader_with_string_default_schema(uri: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.property_schema = vec![
        shader_property("custom_gain", "float", Some(toml::Value::Float(1.0))),
        shader_property(
            "debug_label",
            "string",
            Some(toml::Value::String("schema-default".to_string())),
        ),
    ];
    shader
}

fn shader_property(
    name: &str,
    kind: &str,
    default: Option<toml::Value>,
) -> ShaderMaterialPropertyAsset {
    ShaderMaterialPropertyAsset {
        name: name.to_string(),
        kind: kind.to_string(),
        required: default.is_none(),
        default,
        editor: Default::default(),
    }
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

fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("test texture layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
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
    AssetUri::parse(uri).expect("asset uri")
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(locator(uri))
}

fn f32_at(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
