use crate::asset::ShaderAsset;
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderStage,
};

const RENDERER_MATERIAL_BIND_GROUP: u32 = 3;
const RENDERER_MATERIAL_UNIFORM_BINDING: u32 = 0;

pub(super) fn renderer_material_layout_diagnostics(
    shader: &ShaderAsset,
) -> Vec<RenderMaterialValidationError> {
    if shader.pipeline_layout.bind_groups.is_empty() {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    let material_groups = shader
        .pipeline_layout
        .bind_groups
        .iter()
        .filter(|group| group.group == RENDERER_MATERIAL_BIND_GROUP)
        .collect::<Vec<_>>();

    if material_groups.is_empty() {
        diagnostics.push(material_abi_diagnostic(
            material_group_path(),
            format!(
                "renderer material ABI requires @group({RENDERER_MATERIAL_BIND_GROUP}) \
                 @binding({RENDERER_MATERIAL_UNIFORM_BINDING}) as a uniform buffer"
            ),
        ));
        return diagnostics;
    }

    if material_groups.len() > 1 {
        diagnostics.push(material_abi_diagnostic(
            material_group_path(),
            format!(
                "renderer material ABI expects one bind group descriptor for group \
                 {RENDERER_MATERIAL_BIND_GROUP}, but shader declares {}",
                material_groups.len()
            ),
        ));
    }

    let material_group = material_groups[0];
    push_material_uniform_binding_diagnostics(material_group, &mut diagnostics);
    push_extra_material_binding_diagnostics(material_group, &mut diagnostics);
    diagnostics
}

fn push_material_uniform_binding_diagnostics(
    material_group: &RenderShaderBindGroupLayoutDescriptor,
    diagnostics: &mut Vec<RenderMaterialValidationError>,
) {
    let material_uniform_bindings = material_group
        .bindings
        .iter()
        .filter(|binding| binding.binding == RENDERER_MATERIAL_UNIFORM_BINDING)
        .collect::<Vec<_>>();

    if material_uniform_bindings.is_empty() {
        diagnostics.push(material_abi_diagnostic(
            material_binding_path(RENDERER_MATERIAL_UNIFORM_BINDING),
            format!(
                "renderer material ABI requires group {RENDERER_MATERIAL_BIND_GROUP} binding \
                 {RENDERER_MATERIAL_UNIFORM_BINDING} to declare the material property uniform"
            ),
        ));
        return;
    }

    if material_uniform_bindings.len() > 1 {
        diagnostics.push(material_abi_diagnostic(
            material_binding_path(RENDERER_MATERIAL_UNIFORM_BINDING),
            format!(
                "renderer material ABI expects one descriptor for group \
                 {RENDERER_MATERIAL_BIND_GROUP} binding {RENDERER_MATERIAL_UNIFORM_BINDING}, \
                 but shader declares {}",
                material_uniform_bindings.len()
            ),
        ));
    }

    let material_uniform = material_uniform_bindings[0];
    if material_uniform.resource_type != RenderShaderBindingResourceType::UniformBuffer {
        diagnostics.push(material_abi_diagnostic(
            material_binding_path(RENDERER_MATERIAL_UNIFORM_BINDING),
            format!(
                "renderer material ABI requires group {RENDERER_MATERIAL_BIND_GROUP} binding \
                 {RENDERER_MATERIAL_UNIFORM_BINDING} to be a uniform buffer, but shader declares {:?}",
                material_uniform.resource_type
            ),
        ));
    }

    if !material_uniform_has_mesh_visibility(material_uniform) {
        diagnostics.push(material_abi_diagnostic(
            material_binding_path(RENDERER_MATERIAL_UNIFORM_BINDING),
            format!(
                "renderer material ABI requires group {RENDERER_MATERIAL_BIND_GROUP} binding \
                 {RENDERER_MATERIAL_UNIFORM_BINDING} visibility to include vertex or fragment stage"
            ),
        ));
    }
}

fn push_extra_material_binding_diagnostics(
    material_group: &RenderShaderBindGroupLayoutDescriptor,
    diagnostics: &mut Vec<RenderMaterialValidationError>,
) {
    for binding in material_group
        .bindings
        .iter()
        .filter(|binding| binding.binding != RENDERER_MATERIAL_UNIFORM_BINDING)
    {
        diagnostics.push(material_abi_diagnostic(
            material_binding_path(binding.binding),
            format!(
                "renderer material ABI currently supports only group \
                 {RENDERER_MATERIAL_BIND_GROUP} binding {RENDERER_MATERIAL_UNIFORM_BINDING}; \
                 shader declares unsupported material binding {}",
                binding.binding
            ),
        ));
    }
}

fn material_uniform_has_mesh_visibility(binding: &RenderShaderBindingDescriptor) -> bool {
    binding.visibility.is_empty()
        || binding.visibility.contains(&RenderShaderStage::Vertex)
        || binding.visibility.contains(&RenderShaderStage::Fragment)
}

fn material_abi_diagnostic(path: String, diagnostic: String) -> RenderMaterialValidationError {
    RenderMaterialValidationError::ShaderReadinessDiagnostic {
        source: RenderMaterialDiagnosticSource::RendererMaterialAbi,
        path,
        diagnostic,
    }
}

fn material_group_path() -> String {
    format!("pipeline_layout.group{RENDERER_MATERIAL_BIND_GROUP}")
}

fn material_binding_path(binding: u32) -> String {
    format!("{}.binding{binding}", material_group_path())
}
