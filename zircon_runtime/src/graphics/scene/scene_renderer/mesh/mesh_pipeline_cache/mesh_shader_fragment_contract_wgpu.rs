use crate::graphics::scene::scene_renderer::deferred::{
    GBUFFER_ALBEDO_FORMAT, GBUFFER_EMISSIVE_FORMAT, GBUFFER_MATERIAL_FORMAT,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline::{
    HIT_PROXY_TOKEN_FORMAT, HIT_PROXY_WORLD_NORMAL_FORMAT, HIT_PROXY_WORLD_POSITION_DEPTH_FORMAT,
    MESH_TAA_REACTIVE_MASK_TARGET_FORMAT, MESH_VELOCITY_TARGET_FORMAT,
};
use crate::graphics::scene::scene_renderer::prepass::NORMAL_FORMAT;
use crate::graphics::shader::template::{
    ShaderFragmentOutputNumericType, ShaderFragmentOutputScalarKind,
};

use super::mesh_pipeline_cache::PipelineCreationTarget;
use super::mesh_shader_fragment_contract::{
    MeshShaderFragmentOutputContract, MeshShaderFragmentTargetRequirement,
};

pub(super) struct MeshShaderFragmentOutputContracts {
    base: MeshShaderFragmentOutputContract,
    gbuffer: MeshShaderFragmentOutputContract,
    empty: MeshShaderFragmentOutputContract,
    hit_proxy: MeshShaderFragmentOutputContract,
    velocity: MeshShaderFragmentOutputContract,
    taa_reactive_mask: MeshShaderFragmentOutputContract,
}

impl MeshShaderFragmentOutputContracts {
    pub(super) fn from_wgpu_pipeline_targets(
        base_target_format: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        Ok(Self {
            base: MeshShaderFragmentOutputContract::from_wgpu_color_target_formats([(
                0,
                base_target_format,
            )])?,
            gbuffer: MeshShaderFragmentOutputContract::from_wgpu_color_target_formats([
                (0, GBUFFER_ALBEDO_FORMAT),
                (1, NORMAL_FORMAT),
                (2, GBUFFER_MATERIAL_FORMAT),
                (3, GBUFFER_EMISSIVE_FORMAT),
            ])?,
            empty: MeshShaderFragmentOutputContract::default(),
            hit_proxy: MeshShaderFragmentOutputContract::from_wgpu_color_target_formats([
                (0, HIT_PROXY_TOKEN_FORMAT),
                (1, HIT_PROXY_WORLD_POSITION_DEPTH_FORMAT),
                (2, HIT_PROXY_WORLD_NORMAL_FORMAT),
            ])?,
            velocity: MeshShaderFragmentOutputContract::from_wgpu_color_target_formats([(
                0,
                MESH_VELOCITY_TARGET_FORMAT,
            )])?,
            taa_reactive_mask: MeshShaderFragmentOutputContract::from_wgpu_color_target_formats([
                (0, MESH_TAA_REACTIVE_MASK_TARGET_FORMAT),
            ])?,
        })
    }

    pub(super) fn for_target(
        &self,
        target: PipelineCreationTarget,
    ) -> &MeshShaderFragmentOutputContract {
        match target {
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base) => &self.base,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::GBuffer) => &self.gbuffer,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::HitProxy) => &self.hit_proxy,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Velocity) => &self.velocity,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::TaaReactiveMask)
            | PipelineCreationTarget::MeshPass(MeshPassPipelineKind::TaaReactiveMaterialMask) => {
                &self.taa_reactive_mask
            }
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::DepthPrepass)
            | PipelineCreationTarget::MeshPass(MeshPassPipelineKind::ShadowDepth)
            | PipelineCreationTarget::MeshPass(MeshPassPipelineKind::ShadowDepthAlphaMask)
            | PipelineCreationTarget::Oit => &self.empty,
        }
    }
}

impl MeshShaderFragmentOutputContract {
    fn from_wgpu_color_target_formats(
        targets: impl IntoIterator<Item = (u32, wgpu::TextureFormat)>,
    ) -> Result<Self, String> {
        Self::new(
            targets
                .into_iter()
                .map(|(location, format)| {
                    shader_fragment_output_numeric_type(format)
                        .map(|numeric_type| {
                            MeshShaderFragmentTargetRequirement::new(location, numeric_type)
                        })
                        .map_err(|message| format!("@location({location}) {message}"))
                })
                .collect::<Result<Vec<_>, _>>()?,
        )
    }
}

fn shader_fragment_output_numeric_type(
    format: wgpu::TextureFormat,
) -> Result<ShaderFragmentOutputNumericType, String> {
    let (scalar_kind, scalar_width) = if format == wgpu::TextureFormat::R64Uint {
        (ShaderFragmentOutputScalarKind::Uint, 8)
    } else {
        let scalar_kind = match format.sample_type(None, None) {
            Some(wgpu::TextureSampleType::Float { .. }) => ShaderFragmentOutputScalarKind::Float,
            Some(wgpu::TextureSampleType::Uint) => ShaderFragmentOutputScalarKind::Uint,
            Some(wgpu::TextureSampleType::Sint) => ShaderFragmentOutputScalarKind::Sint,
            Some(wgpu::TextureSampleType::Depth) | None => {
                return Err(format!(
                    "Mesh color target format {format:?} has no numeric fragment-output type"
                ));
            }
        };
        (scalar_kind, 4)
    };
    ShaderFragmentOutputNumericType::new(scalar_kind, scalar_width, format.components())
        .ok_or_else(|| format!("Mesh color target format {format:?} has invalid components"))
}

#[cfg(test)]
mod tests {
    use crate::graphics::shader::template::{
        ShaderFragmentOutputNumericType, ShaderFragmentOutputScalarKind,
    };

    use super::{MeshShaderFragmentOutputContracts, shader_fragment_output_numeric_type};
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
}
