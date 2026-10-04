use crate::graphics::scene::resources::GpuMeshVertex;
use crate::graphics::shader::template::validate_material_shader_template_wgsl;

use super::{MeshShaderVertexLayoutContract, ShaderVertexInputScalarKind};

#[test]
fn standard_mesh_contract_is_projected_from_the_production_vertex_layout() {
    let contract =
        MeshShaderVertexLayoutContract::from_wgpu_vertex_buffer_layouts([GpuMeshVertex::layout()])
            .expect("the production Mesh vertex layout is unique");

    assert_eq!(contract.attribute_count(), 8);
    assert_eq!(
        contract.scalar_kind_at(0),
        Some(ShaderVertexInputScalarKind::Float)
    );
    assert_eq!(
        contract.scalar_kind_at(3),
        Some(ShaderVertexInputScalarKind::Uint)
    );
    assert_eq!(contract.scalar_kind_at(8), None);
}

#[test]
fn velocity_contract_extends_the_same_layout_with_previous_position() {
    let contract = MeshShaderVertexLayoutContract::from_wgpu_vertex_buffer_layouts([
        GpuMeshVertex::layout(),
        GpuMeshVertex::previous_position_layout(),
    ])
    .expect("the production Velocity vertex layouts are unique");

    assert_eq!(contract.attribute_count(), 9);
    assert_eq!(
        contract.scalar_kind_at(8),
        Some(ShaderVertexInputScalarKind::Float)
    );
}

#[test]
fn production_layout_contract_rejects_an_unprovided_shader_location() {
    let contract =
        MeshShaderVertexLayoutContract::from_wgpu_vertex_buffer_layouts([GpuMeshVertex::layout()])
            .expect("the production Mesh vertex layout is unique");
    let reflection = validate_material_shader_template_wgsl(
        r#"
@vertex
fn vs_main(@location(8) previous_position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(previous_position, 1.0);
}
"#,
    )
    .expect("valid WGSL")
    .reflection;

    let error = contract
        .validate(&reflection, "vs_main")
        .expect_err("the standard Mesh layout does not provide Velocity location 8");

    assert!(error.contains("@location(8)"), "unexpected error: {error}");
}
