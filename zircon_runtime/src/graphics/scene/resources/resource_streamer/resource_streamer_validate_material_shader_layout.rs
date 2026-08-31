use crate::asset::ShaderAsset;
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderStage,
};
#[cfg(test)]
use crate::graphics::scene::gpu_scene::{
    GPU_SCENE_INSTANCE_DATA_BINDING, GPU_SCENE_LIGHT_DATA_BINDING,
    GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING, GPU_SCENE_PRIMITIVE_DATA_BINDING,
    GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
};
use crate::graphics::scene::resources::{
    GPU_SCENE_DRAW_BIND_GROUP, MATERIAL_BASE_COLOR_SAMPLER_BINDING,
    MATERIAL_BASE_COLOR_TEXTURE_BINDING, MATERIAL_BIND_GROUP,
    MATERIAL_CLEARCOAT_NORMAL_SAMPLER_BINDING, MATERIAL_CLEARCOAT_NORMAL_TEXTURE_BINDING,
    MATERIAL_EMISSIVE_SAMPLER_BINDING, MATERIAL_EMISSIVE_TEXTURE_BINDING,
    MATERIAL_METALLIC_ROUGHNESS_SAMPLER_BINDING, MATERIAL_METALLIC_ROUGHNESS_TEXTURE_BINDING,
    MATERIAL_NORMAL_SAMPLER_BINDING, MATERIAL_NORMAL_TEXTURE_BINDING,
    MATERIAL_OCCLUSION_SAMPLER_BINDING, MATERIAL_OCCLUSION_TEXTURE_BINDING,
    MATERIAL_UNIFORM_BINDING, RendererShaderBindingContract, gpu_scene_shader_binding_contract,
    material_shader_binding_contract,
};

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
    push_bind_group_diagnostics(
        shader,
        MATERIAL_BIND_GROUP,
        "renderer material ABI",
        material_shader_binding_contract(),
        diagnostics,
    );
}

fn push_gpu_scene_bind_group_diagnostics(
    shader: &ShaderAsset,
    diagnostics: &mut Vec<RenderMaterialValidationError>,
) {
    push_bind_group_diagnostics(
        shader,
        GPU_SCENE_DRAW_BIND_GROUP,
        "renderer GPUScene ABI",
        gpu_scene_shader_binding_contract(),
        diagnostics,
    );
}

fn push_bind_group_diagnostics(
    shader: &ShaderAsset,
    group: u32,
    abi_name: &str,
    expected_bindings: &[RendererShaderBindingContract],
    diagnostics: &mut Vec<RenderMaterialValidationError>,
) {
    let mut groups = shader
        .pipeline_layout
        .bind_groups
        .iter()
        .filter(|bind_group| bind_group.group == group);
    let Some(bind_group) = groups.next() else {
        diagnostics.push(material_abi_diagnostic(
            bind_group_path(group),
            format!(
                "{abi_name} requires @group({group}) with {}",
                expected_binding_list(expected_bindings)
            ),
        ));
        return;
    };

    let duplicate_group_count = groups.count();
    if duplicate_group_count != 0 {
        diagnostics.push(material_abi_diagnostic(
            bind_group_path(group),
            format!(
                "{abi_name} expects one bind group descriptor for group {group}, but shader declares {}",
                duplicate_group_count + 1
            ),
        ));
    }

    for expected in expected_bindings {
        push_expected_binding_diagnostics(group, abi_name, bind_group, expected, diagnostics);
    }
    push_extra_binding_diagnostics(group, abi_name, bind_group, expected_bindings, diagnostics);
}

fn push_expected_binding_diagnostics(
    group: u32,
    abi_name: &str,
    bind_group: &RenderShaderBindGroupLayoutDescriptor,
    expected: &RendererShaderBindingContract,
    diagnostics: &mut Vec<RenderMaterialValidationError>,
) {
    let mut bindings = bind_group
        .bindings
        .iter()
        .filter(|binding| binding.binding == expected.binding);
    let Some(binding) = bindings.next() else {
        diagnostics.push(material_abi_diagnostic(
            binding_path(group, expected.binding),
            format!(
                "{abi_name} requires group {group} binding {} to declare the {}",
                expected.binding, expected.label
            ),
        ));
        return;
    };

    let duplicate_binding_count = bindings.count();
    if duplicate_binding_count != 0 {
        diagnostics.push(material_abi_diagnostic(
            binding_path(group, expected.binding),
            format!(
                "{abi_name} expects one descriptor for group {group} binding {}, but shader declares {}",
                expected.binding,
                duplicate_binding_count + 1
            ),
        ));
    }

    if binding.resource_type != expected.resource_type {
        diagnostics.push(material_abi_diagnostic(
            binding_path(group, expected.binding),
            format!(
                "{abi_name} requires group {group} binding {} to be {:?}, but shader declares {:?}",
                expected.binding, expected.resource_type, binding.resource_type
            ),
        ));
    }

    if !binding_visibility_is_compatible(binding, expected.allowed_visibility) {
        diagnostics.push(material_abi_diagnostic(
            binding_path(group, expected.binding),
            format!(
                "{abi_name} requires group {group} binding {} visibility to be empty or a subset of the {} stage set",
                expected.binding,
                visibility_description(expected.allowed_visibility)
            ),
        ));
    }
}

fn push_extra_binding_diagnostics(
    group: u32,
    abi_name: &str,
    bind_group: &RenderShaderBindGroupLayoutDescriptor,
    expected_bindings: &[RendererShaderBindingContract],
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

fn binding_visibility_is_compatible(
    binding: &RenderShaderBindingDescriptor,
    allowed_visibility: &[RenderShaderStage],
) -> bool {
    binding.visibility.is_empty()
        || binding
            .visibility
            .iter()
            .all(|stage| allowed_visibility.contains(stage))
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

fn expected_binding_list(expected_bindings: &[RendererShaderBindingContract]) -> String {
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
        assert!(
            !diagnostics
                .iter()
                .any(|diagnostic| diagnostic_path(diagnostic) == Some("pipeline_layout.group1"))
        );
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
}
