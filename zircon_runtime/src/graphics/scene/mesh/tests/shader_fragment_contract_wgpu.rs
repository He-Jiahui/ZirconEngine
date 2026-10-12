use crate::graphics::shader::template::{
    ShaderFragmentOutputNumericType, ShaderFragmentOutputScalarKind,
};

use super::{shader_fragment_output_numeric_type, MeshShaderFragmentOutputContracts};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::PipelineCreationTarget;

#[test]
fn wgpu_fragment_target_projection_preserves_scalar_kind_width_and_components() {
    assert_eq!(
        shader_fragment_output_numeric_type(wgpu::TextureFormat::Rg16Float),
        Ok(
            ShaderFragmentOutputNumericType::new(ShaderFragmentOutputScalarKind::Float, 4, 2,)
                .expect("valid target type")
        )
    );
    assert_eq!(
        shader_fragment_output_numeric_type(wgpu::TextureFormat::R32Uint),
        Ok(
            ShaderFragmentOutputNumericType::new(ShaderFragmentOutputScalarKind::Uint, 4, 1,)
                .expect("valid target type")
        )
    );
    assert_eq!(
        shader_fragment_output_numeric_type(wgpu::TextureFormat::R64Uint),
        Ok(
            ShaderFragmentOutputNumericType::new(ShaderFragmentOutputScalarKind::Uint, 8, 1,)
                .expect("valid target type")
        )
    );
}

#[test]
fn production_target_contracts_use_exact_mesh_attachment_shapes() {
    let contracts = MeshShaderFragmentOutputContracts::from_wgpu_pipeline_targets(
        wgpu::TextureFormat::Bgra8UnormSrgb,
    )
    .expect("production target formats must project");
    let hit_proxy = contracts.for_target(PipelineCreationTarget::MeshPass(
        MeshPassPipelineKind::HitProxy,
    ));
    assert_eq!(
        hit_proxy.numeric_type(0),
        ShaderFragmentOutputNumericType::new(ShaderFragmentOutputScalarKind::Uint, 4, 1,)
    );
    let velocity = contracts.for_target(PipelineCreationTarget::MeshPass(
        MeshPassPipelineKind::Velocity,
    ));
    assert_eq!(
        velocity.numeric_type(0),
        ShaderFragmentOutputNumericType::new(ShaderFragmentOutputScalarKind::Float, 4, 2,)
    );
    let shadow = contracts.for_target(PipelineCreationTarget::MeshPass(
        MeshPassPipelineKind::ShadowDepthAlphaMask,
    ));
    assert_eq!(shadow.numeric_type(0), None);
}
