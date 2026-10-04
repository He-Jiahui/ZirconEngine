use super::{reflect_validated_shader_module, ShaderBindingResourceType};

#[test]
fn target_server_reflects_a_validated_naga_module_without_graphics() {
    let module = naga::front::wgsl::parse_str(
        "struct Material { tint: vec4<f32>, };\n\
             @group(0) @binding(0) var<uniform> material: Material;\n\
             @fragment\n\
             fn main() -> @location(0) vec4<f32> { return material.tint; }",
    )
    .expect("target-server WGSL fixture parses through Naga");
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    let module_info = validator
        .validate(&module)
        .expect("target-server WGSL fixture validates through Naga");

    let reflection = reflect_validated_shader_module(&module, &module_info);
    assert_eq!(reflection.resource_bindings.len(), 1);
    let binding = &reflection.resource_bindings[0];
    assert_eq!(binding.identity.group, 0);
    assert_eq!(binding.identity.binding, 0);
    assert_eq!(
        binding.identity.resource_type,
        ShaderBindingResourceType::UniformBuffer
    );
    assert!(binding.visibility.contains(naga::ShaderStage::Fragment));
}
