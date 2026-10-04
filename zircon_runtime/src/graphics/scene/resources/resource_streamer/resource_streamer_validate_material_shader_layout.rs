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
    gpu_scene_shader_binding_contract, material_shader_binding_contract,
    RendererShaderBindingContract, GPU_SCENE_DRAW_BIND_GROUP, MATERIAL_BASE_COLOR_SAMPLER_BINDING,
    MATERIAL_BASE_COLOR_TEXTURE_BINDING, MATERIAL_BIND_GROUP,
    MATERIAL_CLEARCOAT_NORMAL_SAMPLER_BINDING, MATERIAL_CLEARCOAT_NORMAL_TEXTURE_BINDING,
    MATERIAL_EMISSIVE_SAMPLER_BINDING, MATERIAL_EMISSIVE_TEXTURE_BINDING,
    MATERIAL_METALLIC_ROUGHNESS_SAMPLER_BINDING, MATERIAL_METALLIC_ROUGHNESS_TEXTURE_BINDING,
    MATERIAL_NORMAL_SAMPLER_BINDING, MATERIAL_NORMAL_TEXTURE_BINDING,
    MATERIAL_OCCLUSION_SAMPLER_BINDING, MATERIAL_OCCLUSION_TEXTURE_BINDING,
    MATERIAL_UNIFORM_BINDING,
};

/// 将显式布局与 renderer material、GPUScene 的绑定 ABI 比较并生成 readiness 诊断；空布局保留不参与此校验的约定。
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
#[path = "tests/resource_streamer_validate_material_shader_layout.rs"]
mod tests;
