use super::*;
use crate::asset::{AssetUri, ShaderSourceLanguage};
use crate::core::framework::render::{RenderShaderPipelineLayoutDescriptor, ShaderAssetKind};

#[test]
fn renderer_material_layout_diagnostics_keep_empty_layout_opt_out() {
    let shader = shader_with_layout(Vec::new());

    assert!(renderer_material_layout_diagnostics(&shader).is_empty());
}

#[test]
fn renderer_material_layout_diagnostics_accept_current_renderer_abi() {
    let shader = shader_with_layout(vec![
        bind_group(
            MATERIAL_BIND_GROUP,
            vec![
                binding(
                    MATERIAL_BASE_COLOR_TEXTURE_BINDING,
                    RenderShaderBindingResourceType::Texture,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_BASE_COLOR_SAMPLER_BINDING,
                    RenderShaderBindingResourceType::Sampler,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_NORMAL_TEXTURE_BINDING,
                    RenderShaderBindingResourceType::Texture,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_NORMAL_SAMPLER_BINDING,
                    RenderShaderBindingResourceType::Sampler,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_METALLIC_ROUGHNESS_TEXTURE_BINDING,
                    RenderShaderBindingResourceType::Texture,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_METALLIC_ROUGHNESS_SAMPLER_BINDING,
                    RenderShaderBindingResourceType::Sampler,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_OCCLUSION_TEXTURE_BINDING,
                    RenderShaderBindingResourceType::Texture,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_OCCLUSION_SAMPLER_BINDING,
                    RenderShaderBindingResourceType::Sampler,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_EMISSIVE_TEXTURE_BINDING,
                    RenderShaderBindingResourceType::Texture,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_EMISSIVE_SAMPLER_BINDING,
                    RenderShaderBindingResourceType::Sampler,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_UNIFORM_BINDING,
                    RenderShaderBindingResourceType::UniformBuffer,
                    vec![RenderShaderStage::Vertex, RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_CLEARCOAT_NORMAL_TEXTURE_BINDING,
                    RenderShaderBindingResourceType::Texture,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_CLEARCOAT_NORMAL_SAMPLER_BINDING,
                    RenderShaderBindingResourceType::Sampler,
                    vec![RenderShaderStage::Fragment],
                ),
            ],
        ),
        bind_group(
            GPU_SCENE_DRAW_BIND_GROUP,
            vec![
                binding(
                    GPU_SCENE_PRIMITIVE_DATA_BINDING,
                    RenderShaderBindingResourceType::StorageBuffer,
                    vec![RenderShaderStage::Vertex, RenderShaderStage::Fragment],
                ),
                binding(
                    GPU_SCENE_INSTANCE_DATA_BINDING,
                    RenderShaderBindingResourceType::StorageBuffer,
                    vec![RenderShaderStage::Vertex, RenderShaderStage::Fragment],
                ),
                binding(
                    GPU_SCENE_LIGHT_DATA_BINDING,
                    RenderShaderBindingResourceType::StorageBuffer,
                    vec![RenderShaderStage::Vertex, RenderShaderStage::Fragment],
                ),
                binding(
                    GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
                    RenderShaderBindingResourceType::StorageBuffer,
                    vec![RenderShaderStage::Vertex],
                ),
                binding(
                    GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING,
                    RenderShaderBindingResourceType::StorageBuffer,
                    vec![RenderShaderStage::Vertex],
                ),
            ],
        ),
    ]);

    assert!(renderer_material_layout_diagnostics(&shader).is_empty());
}

#[test]
fn renderer_material_layout_diagnostics_report_missing_material_and_gpu_scene_groups() {
    let shader = shader_with_layout(vec![bind_group(
        MATERIAL_BIND_GROUP,
        vec![binding(
            MATERIAL_UNIFORM_BINDING,
            RenderShaderBindingResourceType::UniformBuffer,
            vec![RenderShaderStage::Fragment],
        )],
    )]);

    let diagnostics = renderer_material_layout_diagnostics(&shader);

    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group2.binding1",
        "base-color texture"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group3",
        "@group(3)"
    ));
    assert!(!diagnostics
        .iter()
        .any(|diagnostic| diagnostic_path(diagnostic) == Some("pipeline_layout.group1")));
}

#[test]
fn renderer_material_layout_diagnostics_validate_gpu_scene_and_material_bindings() {
    let shader = shader_with_layout(vec![
        bind_group(
            MATERIAL_BIND_GROUP,
            vec![
                binding(
                    MATERIAL_BASE_COLOR_TEXTURE_BINDING,
                    RenderShaderBindingResourceType::Sampler,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    MATERIAL_BASE_COLOR_SAMPLER_BINDING,
                    RenderShaderBindingResourceType::Sampler,
                    vec![RenderShaderStage::Compute],
                ),
                binding(
                    MATERIAL_NORMAL_SAMPLER_BINDING,
                    RenderShaderBindingResourceType::Sampler,
                    vec![RenderShaderStage::Fragment],
                ),
                binding(
                    10,
                    RenderShaderBindingResourceType::Texture,
                    vec![RenderShaderStage::Fragment],
                ),
            ],
        ),
        bind_group(
            GPU_SCENE_DRAW_BIND_GROUP,
            vec![
                binding(
                    GPU_SCENE_PRIMITIVE_DATA_BINDING,
                    RenderShaderBindingResourceType::UniformBuffer,
                    vec![RenderShaderStage::Vertex],
                ),
                binding(
                    GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
                    RenderShaderBindingResourceType::Texture,
                    vec![RenderShaderStage::Vertex],
                ),
            ],
        ),
    ]);

    let diagnostics = renderer_material_layout_diagnostics(&shader);

    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group3.binding0",
        "StorageBuffer"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group3.binding1",
        "GPUScene instance"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group3.binding2",
        "GPUScene light"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group3.binding3",
        "StorageBuffer"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group3.binding4",
        "previous skinned joint palette"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group2.binding0",
        "material property uniform"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group2.binding1",
        "Texture"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group2.binding2",
        "fragment stage"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group2.binding3",
        "normal texture"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group2.binding5",
        "metallic-roughness texture"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group2.binding7",
        "occlusion texture"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group2.binding9",
        "emissive texture"
    ));
    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group2.binding10",
        "Sampler"
    ));
}

#[test]
fn renderer_material_layout_diagnostics_reject_mixed_allowed_and_compute_visibility() {
    let shader = shader_with_layout(vec![bind_group(
        MATERIAL_BIND_GROUP,
        vec![binding(
            MATERIAL_BASE_COLOR_TEXTURE_BINDING,
            RenderShaderBindingResourceType::Texture,
            vec![RenderShaderStage::Fragment, RenderShaderStage::Compute],
        )],
    )]);

    let diagnostics = renderer_material_layout_diagnostics(&shader);

    assert!(diagnostic_contains(
        &diagnostics,
        "pipeline_layout.group2.binding1",
        "subset of the fragment stage set"
    ));
}

fn shader_with_layout(bind_groups: Vec<RenderShaderBindGroupLayoutDescriptor>) -> ShaderAsset {
    ShaderAsset {
        uri: AssetUri::parse("res://tests/renderer-layout.zshader").unwrap(),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: String::new(),
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
        shading_model: None,
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: Default::default(),
        pipeline_layout: RenderShaderPipelineLayoutDescriptor {
            bind_groups,
            push_constant_ranges: Vec::new(),
        },
        validation_diagnostics: Vec::new(),
    }
}

fn bind_group(
    group: u32,
    bindings: Vec<RenderShaderBindingDescriptor>,
) -> RenderShaderBindGroupLayoutDescriptor {
    RenderShaderBindGroupLayoutDescriptor {
        group,
        label: None,
        bindings,
    }
}

fn binding(
    binding: u32,
    resource_type: RenderShaderBindingResourceType,
    visibility: Vec<RenderShaderStage>,
) -> RenderShaderBindingDescriptor {
    RenderShaderBindingDescriptor {
        binding,
        label: None,
        resource_type,
        visibility,
    }
}

fn diagnostic_contains(
    diagnostics: &[RenderMaterialValidationError],
    expected_path: &str,
    expected_text: &str,
) -> bool {
    diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic,
            RenderMaterialValidationError::ShaderReadinessDiagnostic {
                source: RenderMaterialDiagnosticSource::RendererMaterialAbi,
                path,
                diagnostic,
            } if path == expected_path && diagnostic.contains(expected_text)
        )
    })
}

fn diagnostic_path(diagnostic: &RenderMaterialValidationError) -> Option<&str> {
    match diagnostic {
        RenderMaterialValidationError::ShaderReadinessDiagnostic { path, .. } => {
            Some(path.as_str())
        }
        _ => None,
    }
}
