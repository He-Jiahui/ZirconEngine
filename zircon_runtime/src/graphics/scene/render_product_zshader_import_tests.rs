use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::{
    AlphaMode, AssetMetaDocument, AssetReference, AssetSourceUnit, AssetUri, AssetUuid,
    ImportedAsset, MaterialAsset, ProjectAssetManager, ProjectManager, ProjectManifest,
    ProjectPaths, ShaderAsset,
};
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError, RenderShaderBindingResourceType,
};
use crate::core::resource::{
    MaterialMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::backend::RenderBackend;

use super::resources::ResourceStreamer;

#[test]
fn render_product_streamer_reports_imported_zshader_material_layout_abi_diagnostics() {
    let root = unique_temp_project_root("render_product_imported_zshader_layout_abi");
    let shader_uri = AssetUri::parse("res://shaders/imported_layout_shader").unwrap();
    write_imported_layout_shader_project(&root, &shader_uri);

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    let shader = match manager.load_artifact(&shader_uri).unwrap() {
        ImportedAsset::Shader(shader) => shader,
        other => panic!("unexpected imported shader artifact: {other:?}"),
    };
    assert_eq!(
        shader.pipeline_layout.bind_groups[0].bindings[0].resource_type,
        RenderShaderBindingResourceType::Texture
    );

    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
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

    let _ = fs::remove_dir_all(root);
}

fn write_imported_layout_shader_project(root: &PathBuf, shader_uri: &AssetUri) {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new("ImportedLayoutShaderSandbox", shader_uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();

    let shader_meta_path = paths
        .assets_root()
        .join("shaders")
        .join("imported_layout_shader.zmeta");
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), ResourceKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths
        .assets_root()
        .join("shaders")
        .join("imported_layout_shader");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("imported.zshader"),
        r#"
version = 1
wgsl_files = ["imported.wgsl"]

[pipeline_layout]
push_constant_ranges = []

[[pipeline_layout.bind_groups]]
group = 3
label = "material"

[[pipeline_layout.bind_groups.bindings]]
binding = 0
label = "material_texture"
resource_type = "texture"
visibility = ["fragment"]

[[pipeline_layout.bind_groups.bindings]]
binding = 1
label = "material_sampler"
resource_type = "sampler"
visibility = ["fragment"]
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("imported.wgsl"),
        r#"
@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0);
}
"#,
    )
    .unwrap();
}

fn material_for_shader(shader_uri: &AssetUri) -> MaterialAsset {
    MaterialAsset {
        name: Some("ImportedLayoutMaterial".to_string()),
        shader: AssetReference::from_locator(shader_uri.clone()),
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

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_{label}_{unique}"))
}
