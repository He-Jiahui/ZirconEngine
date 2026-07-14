use crate::asset::ShaderAsset;
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderStage,
};

const RENDERER_MATERIAL_TEXTURE_BIND_GROUP: u32 = 2;
const RENDERER_MATERIAL_UNIFORM_BINDING: u32 = 0;
const RENDERER_BASE_COLOR_TEXTURE_BINDING: u32 = 1;
const RENDERER_BASE_COLOR_SAMPLER_BINDING: u32 = 2;
const RENDERER_NORMAL_TEXTURE_BINDING: u32 = 3;
const RENDERER_NORMAL_SAMPLER_BINDING: u32 = 4;
const RENDERER_METALLIC_ROUGHNESS_TEXTURE_BINDING: u32 = 5;
const RENDERER_METALLIC_ROUGHNESS_SAMPLER_BINDING: u32 = 6;
const RENDERER_OCCLUSION_TEXTURE_BINDING: u32 = 7;
const RENDERER_OCCLUSION_SAMPLER_BINDING: u32 = 8;
const RENDERER_EMISSIVE_TEXTURE_BINDING: u32 = 9;
const RENDERER_EMISSIVE_SAMPLER_BINDING: u32 = 10;
const RENDERER_CLEARCOAT_NORMAL_TEXTURE_BINDING: u32 = 11;
const RENDERER_CLEARCOAT_NORMAL_SAMPLER_BINDING: u32 = 12;
const RENDERER_GPU_SCENE_BIND_GROUP: u32 = 3;
const RENDERER_GPU_SCENE_PRIMITIVE_BINDING: u32 = 0;
const RENDERER_GPU_SCENE_INSTANCE_BINDING: u32 = 1;
const RENDERER_GPU_SCENE_LIGHT_BINDING: u32 = 2;
const RENDERER_CURRENT_SKINNED_PALETTE_BINDING: u32 = 3;
const RENDERER_PREVIOUS_SKINNED_PALETTE_BINDING: u32 = 4;

pub(super) fn renderer_material_layout_diagnostics(
    shader: &ShaderAsset,
) -> Vec<RenderMaterialValidationError> {
    if shader.pipeline_layout.bind_groups.is_empty() {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    push_material_bind_group_diagnostics(shader, &mut diagnostics);
    push_gpu_scene_bind_group_diagnostics(shader, &mut diagnostics);
    diagnostics
}

fn push_material_bind_group_diagnostics(
    shader: &ShaderAsset,
    diagnostics: &mut Vec<RenderMaterialValidationError>,
) {
    let expected = [
        ExpectedRendererBinding {
            binding: RENDERER_MATERIAL_UNIFORM_BINDING,
            label: "material property uniform",
            resource_type: RenderShaderBindingResourceType::UniformBuffer,
            required_visibility: &[RenderShaderStage::Vertex, RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_BASE_COLOR_TEXTURE_BINDING,
            label: "base-color texture",
            resource_type: RenderShaderBindingResourceType::Texture,
            required_visibility: &[RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_BASE_COLOR_SAMPLER_BINDING,
            label: "base-color sampler",
            resource_type: RenderShaderBindingResourceType::Sampler,
            required_visibility: &[RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_NORMAL_TEXTURE_BINDING,
            label: "normal texture",
            resource_type: RenderShaderBindingResourceType::Texture,
            required_visibility: &[RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_NORMAL_SAMPLER_BINDING,
            label: "normal sampler",
            resource_type: RenderShaderBindingResourceType::Sampler,
            required_visibility: &[RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_METALLIC_ROUGHNESS_TEXTURE_BINDING,
            label: "metallic-roughness texture",
            resource_type: RenderShaderBindingResourceType::Texture,
            required_visibility: &[RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_METALLIC_ROUGHNESS_SAMPLER_BINDING,
            label: "metallic-roughness sampler",
            resource_type: RenderShaderBindingResourceType::Sampler,
            required_visibility: &[RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_OCCLUSION_TEXTURE_BINDING,
            label: "occlusion texture",
            resource_type: RenderShaderBindingResourceType::Texture,
            required_visibility: &[RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_OCCLUSION_SAMPLER_BINDING,
            label: "occlusion sampler",
            resource_type: RenderShaderBindingResourceType::Sampler,
            required_visibility: &[RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_EMISSIVE_TEXTURE_BINDING,
            label: "emissive texture",
            resource_type: RenderShaderBindingResourceType::Texture,
            required_visibility: &[RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_EMISSIVE_SAMPLER_BINDING,
            label: "emissive sampler",
            resource_type: RenderShaderBindingResourceType::Sampler,
            required_visibility: &[RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_CLEARCOAT_NORMAL_TEXTURE_BINDING,
            label: "clearcoat-normal texture",
            resource_type: RenderShaderBindingResourceType::Texture,
            required_visibility: &[RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_CLEARCOAT_NORMAL_SAMPLER_BINDING,
            label: "clearcoat-normal sampler",
            resource_type: RenderShaderBindingResourceType::Sampler,
            required_visibility: &[RenderShaderStage::Fragment],
        },
    ];

    push_bind_group_diagnostics(
        shader,
        RENDERER_MATERIAL_TEXTURE_BIND_GROUP,
        "renderer material ABI",
        &expected,
        diagnostics,
    );
}

fn push_gpu_scene_bind_group_diagnostics(
    shader: &ShaderAsset,
    diagnostics: &mut Vec<RenderMaterialValidationError>,
) {
    let expected = [
        ExpectedRendererBinding {
            binding: RENDERER_GPU_SCENE_PRIMITIVE_BINDING,
            label: "GPUScene primitive storage buffer",
            resource_type: RenderShaderBindingResourceType::StorageBuffer,
            required_visibility: &[RenderShaderStage::Vertex, RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_GPU_SCENE_INSTANCE_BINDING,
            label: "GPUScene instance storage buffer",
            resource_type: RenderShaderBindingResourceType::StorageBuffer,
            required_visibility: &[RenderShaderStage::Vertex, RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_GPU_SCENE_LIGHT_BINDING,
            label: "GPUScene light storage buffer",
            resource_type: RenderShaderBindingResourceType::StorageBuffer,
            required_visibility: &[RenderShaderStage::Vertex, RenderShaderStage::Fragment],
        },
        ExpectedRendererBinding {
            binding: RENDERER_CURRENT_SKINNED_PALETTE_BINDING,
            label: "current skinned joint palette storage",
            resource_type: RenderShaderBindingResourceType::StorageBuffer,
            required_visibility: &[RenderShaderStage::Vertex],
        },
        ExpectedRendererBinding {
            binding: RENDERER_PREVIOUS_SKINNED_PALETTE_BINDING,
            label: "previous skinned joint palette storage",
            resource_type: RenderShaderBindingResourceType::StorageBuffer,
            required_visibility: &[RenderShaderStage::Vertex],
        },
    ];

    push_bind_group_diagnostics(
        shader,
        RENDERER_GPU_SCENE_BIND_GROUP,
        "renderer GPUScene ABI",
        &expected,
        diagnostics,
    );
}

struct ExpectedRendererBinding {
    binding: u32,
    label: &'static str,
    resource_type: RenderShaderBindingResourceType,
    required_visibility: &'static [RenderShaderStage],
}

fn push_bind_group_diagnostics(
    shader: &ShaderAsset,
    group: u32,
    abi_name: &str,
    expected_bindings: &[ExpectedRendererBinding],
    diagnostics: &mut Vec<RenderMaterialValidationError>,
) {
    let groups = matching_bind_groups(shader, group);
    if groups.is_empty() {
        diagnostics.push(material_abi_diagnostic(
            bind_group_path(group),
            format!(
                "{abi_name} requires @group({group}) with {}",
                expected_binding_list(expected_bindings)
            ),
        ));
        return;
    }

    if groups.len() > 1 {
        diagnostics.push(material_abi_diagnostic(
            bind_group_path(group),
            format!(
                "{abi_name} expects one bind group descriptor for group {group}, but shader declares {}",
                groups.len()
            ),
        ));
    }

    let bind_group = groups[0];
    for expected in expected_bindings {
        push_expected_binding_diagnostics(group, abi_name, bind_group, expected, diagnostics);
    }
    push_extra_binding_diagnostics(group, abi_name, bind_group, expected_bindings, diagnostics);
}

fn push_expected_binding_diagnostics(
    group: u32,
    abi_name: &str,
    bind_group: &RenderShaderBindGroupLayoutDescriptor,
    expected: &ExpectedRendererBinding,
    diagnostics: &mut Vec<RenderMaterialValidationError>,
) {
    let bindings = bind_group
        .bindings
        .iter()
        .filter(|binding| binding.binding == expected.binding)
        .collect::<Vec<_>>();

    if bindings.is_empty() {
        diagnostics.push(material_abi_diagnostic(
            binding_path(group, expected.binding),
            format!(
                "{abi_name} requires group {group} binding {} to declare the {}",
                expected.binding, expected.label
            ),
        ));
        return;
    }

    if bindings.len() > 1 {
        diagnostics.push(material_abi_diagnostic(
            binding_path(group, expected.binding),
            format!(
                "{abi_name} expects one descriptor for group {group} binding {}, but shader declares {}",
                expected.binding,
                bindings.len()
            ),
        ));
    }

    let binding = bindings[0];
    if binding.resource_type != expected.resource_type {
        diagnostics.push(material_abi_diagnostic(
            binding_path(group, expected.binding),
            format!(
                "{abi_name} requires group {group} binding {} to be {:?}, but shader declares {:?}",
                expected.binding, expected.resource_type, binding.resource_type
            ),
        ));
    }

    if !binding_has_required_visibility(binding, expected.required_visibility) {
        diagnostics.push(material_abi_diagnostic(
            binding_path(group, expected.binding),
            format!(
                "{abi_name} requires group {group} binding {} visibility to include {} stage",
                expected.binding,
                visibility_description(expected.required_visibility)
            ),
        ));
    }
}

fn push_extra_binding_diagnostics(
    group: u32,
    abi_name: &str,
    bind_group: &RenderShaderBindGroupLayoutDescriptor,
    expected_bindings: &[ExpectedRendererBinding],
    diagnostics: &mut Vec<RenderMaterialValidationError>,
) {
    for binding in bind_group.bindings.iter().filter(|binding| {
        !expected_bindings
            .iter()
            .any(|expected| expected.binding == binding.binding)
    }) {
        diagnostics.push(material_abi_diagnostic(
            binding_path(group, binding.binding),
            format!(
                "{abi_name} currently supports only group {group} {}; shader declares unsupported binding {}",
                expected_binding_list(expected_bindings),
                binding.binding
            ),
        ));
    }
}

fn matching_bind_groups(
    shader: &ShaderAsset,
    group: u32,
) -> Vec<&RenderShaderBindGroupLayoutDescriptor> {
    shader
        .pipeline_layout
        .bind_groups
        .iter()
        .filter(|bind_group| bind_group.group == group)
        .collect::<Vec<_>>()
}

fn binding_has_required_visibility(
    binding: &RenderShaderBindingDescriptor,
    required_visibility: &[RenderShaderStage],
) -> bool {
    binding.visibility.is_empty()
        || required_visibility
            .iter()
            .any(|stage| binding.visibility.contains(stage))
}

fn material_abi_diagnostic(path: String, diagnostic: String) -> RenderMaterialValidationError {
    RenderMaterialValidationError::ShaderReadinessDiagnostic {
        source: RenderMaterialDiagnosticSource::RendererMaterialAbi,
        path,
        diagnostic,
    }
}

fn bind_group_path(group: u32) -> String {
    format!("pipeline_layout.group{group}")
}

fn binding_path(group: u32, binding: u32) -> String {
    format!("{}.binding{binding}", bind_group_path(group))
}

fn expected_binding_list(expected_bindings: &[ExpectedRendererBinding]) -> String {
    let bindings = expected_bindings
        .iter()
        .map(|binding| binding.binding.to_string())
        .collect::<Vec<_>>();
    if bindings.len() == 1 {
        format!("binding {}", bindings[0])
    } else {
        format!("bindings {}", bindings.join(", "))
    }
}

fn visibility_description(required_visibility: &[RenderShaderStage]) -> String {
    match required_visibility {
        [RenderShaderStage::Vertex, RenderShaderStage::Fragment] => {
            "vertex or fragment".to_string()
        }
        [RenderShaderStage::Vertex] => "vertex".to_string(),
        [RenderShaderStage::Fragment] => "fragment".to_string(),
        stages => stages
            .iter()
            .map(|stage| format!("{stage:?}").to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join(" or "),
    }
}

#[cfg(test)]
mod tests {
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
                RENDERER_MATERIAL_TEXTURE_BIND_GROUP,
                vec![
                    binding(
                        RENDERER_BASE_COLOR_TEXTURE_BINDING,
                        RenderShaderBindingResourceType::Texture,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_BASE_COLOR_SAMPLER_BINDING,
                        RenderShaderBindingResourceType::Sampler,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_NORMAL_TEXTURE_BINDING,
                        RenderShaderBindingResourceType::Texture,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_NORMAL_SAMPLER_BINDING,
                        RenderShaderBindingResourceType::Sampler,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_METALLIC_ROUGHNESS_TEXTURE_BINDING,
                        RenderShaderBindingResourceType::Texture,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_METALLIC_ROUGHNESS_SAMPLER_BINDING,
                        RenderShaderBindingResourceType::Sampler,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_OCCLUSION_TEXTURE_BINDING,
                        RenderShaderBindingResourceType::Texture,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_OCCLUSION_SAMPLER_BINDING,
                        RenderShaderBindingResourceType::Sampler,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_EMISSIVE_TEXTURE_BINDING,
                        RenderShaderBindingResourceType::Texture,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_EMISSIVE_SAMPLER_BINDING,
                        RenderShaderBindingResourceType::Sampler,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_MATERIAL_UNIFORM_BINDING,
                        RenderShaderBindingResourceType::UniformBuffer,
                        vec![RenderShaderStage::Vertex, RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_CLEARCOAT_NORMAL_TEXTURE_BINDING,
                        RenderShaderBindingResourceType::Texture,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_CLEARCOAT_NORMAL_SAMPLER_BINDING,
                        RenderShaderBindingResourceType::Sampler,
                        vec![RenderShaderStage::Fragment],
                    ),
                ],
            ),
            bind_group(
                RENDERER_GPU_SCENE_BIND_GROUP,
                vec![
                    binding(
                        RENDERER_GPU_SCENE_PRIMITIVE_BINDING,
                        RenderShaderBindingResourceType::StorageBuffer,
                        vec![RenderShaderStage::Vertex, RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_GPU_SCENE_INSTANCE_BINDING,
                        RenderShaderBindingResourceType::StorageBuffer,
                        vec![RenderShaderStage::Vertex, RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_GPU_SCENE_LIGHT_BINDING,
                        RenderShaderBindingResourceType::StorageBuffer,
                        vec![RenderShaderStage::Vertex, RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_CURRENT_SKINNED_PALETTE_BINDING,
                        RenderShaderBindingResourceType::StorageBuffer,
                        vec![RenderShaderStage::Vertex],
                    ),
                    binding(
                        RENDERER_PREVIOUS_SKINNED_PALETTE_BINDING,
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
            RENDERER_MATERIAL_TEXTURE_BIND_GROUP,
            vec![binding(
                RENDERER_MATERIAL_UNIFORM_BINDING,
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
                RENDERER_MATERIAL_TEXTURE_BIND_GROUP,
                vec![
                    binding(
                        RENDERER_BASE_COLOR_TEXTURE_BINDING,
                        RenderShaderBindingResourceType::Sampler,
                        vec![RenderShaderStage::Fragment],
                    ),
                    binding(
                        RENDERER_BASE_COLOR_SAMPLER_BINDING,
                        RenderShaderBindingResourceType::Sampler,
                        vec![RenderShaderStage::Compute],
                    ),
                    binding(
                        RENDERER_NORMAL_SAMPLER_BINDING,
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
                RENDERER_GPU_SCENE_BIND_GROUP,
                vec![
                    binding(
                        RENDERER_GPU_SCENE_PRIMITIVE_BINDING,
                        RenderShaderBindingResourceType::UniformBuffer,
                        vec![RenderShaderStage::Vertex],
                    ),
                    binding(
                        RENDERER_CURRENT_SKINNED_PALETTE_BINDING,
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
}
