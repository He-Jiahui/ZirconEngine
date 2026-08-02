use std::sync::OnceLock;

use crate::core::framework::render::{
    ComputeDispatchBuilder, ComputeDispatchPlan, ComputeKernelRef, FullscreenPassBuilder,
    FullscreenPassPlan, FullscreenShaderRef, PostProcessGraphResourceNames,
    RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind, ShaderDispatchExtent,
    ShaderResourceAccess, ShaderResourceDescriptor, ShaderResourceKind,
};

pub(crate) const HZB_BUILD_PIPELINE_LABEL: &str = "zircon-hzb-build-pipeline";
pub(crate) const HZB_BUILD_MSAA_PIPELINE_LABEL: &str = "zircon-hzb-build-msaa-pipeline";
pub(crate) const HZB_BUILD_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
pub(crate) const HZB_SCENE_DEPTH_RESOURCE: &str = PostProcessGraphResourceNames::SCENE_DEPTH;
pub(crate) const HZB_SOURCE_RESOURCE: &str = "source_hzb";
pub(crate) const HZB_TARGET_RESOURCE: &str = "target_hzb";

pub(crate) const MOTION_VECTOR_TILE_MAX_PIPELINE_LABEL: &str =
    "zircon-motion-vector-tile-max-fullscreen";
pub(crate) const MOTION_VECTOR_SOURCE_RESOURCE: &str =
    PostProcessGraphResourceNames::SCENE_VELOCITY;
pub(crate) const MOTION_VECTOR_TILE_SPAN_PARAMETER: &str = "tile_span";

const HZB_BUILD_SHADER: &str = "builtin://shaders/compute/hzb_build";
const HZB_BUILD_KERNEL: &str = "cs_main";
const MOTION_VECTOR_TILE_MAX_SHADER: &str = "builtin://shaders/fullscreen/motion_vector_tile_max";
const MOTION_VECTOR_TILE_MAX_FRAGMENT: &str = "fs_main";

pub(crate) fn hzb_build_dispatch_plan() -> &'static ComputeDispatchPlan {
    static PLAN: OnceLock<ComputeDispatchPlan> = OnceLock::new();
    PLAN.get_or_init(|| hzb_build_dispatch_plan_for_variant(HZB_BUILD_PIPELINE_LABEL, 0))
}

pub(crate) fn hzb_build_msaa_dispatch_plan() -> &'static ComputeDispatchPlan {
    static PLAN: OnceLock<ComputeDispatchPlan> = OnceLock::new();
    PLAN.get_or_init(|| hzb_build_dispatch_plan_for_variant(HZB_BUILD_MSAA_PIPELINE_LABEL, 1))
}

fn hzb_build_dispatch_plan_for_variant(
    pipeline_label: &'static str,
    option_bits: u32,
) -> ComputeDispatchPlan {
    ComputeDispatchBuilder::new(
        ComputeKernelRef::from_locator_str(HZB_BUILD_SHADER, HZB_BUILD_KERNEL)
            .expect("builtin HZB compute shader locator must be valid"),
    )
    .with_pipeline_label(pipeline_label)
    .with_option_bits(option_bits)
    .with_workgroup_size(HZB_BUILD_WORKGROUP_SIZE)
    .bind_texture(HZB_SCENE_DEPTH_RESOURCE)
    .bind_texture(HZB_SOURCE_RESOURCE)
    .bind_storage_texture_write(HZB_TARGET_RESOURCE)
    .dispatch_extent(ShaderDispatchExtent::HzbFurthest)
    .build(
        ShaderAssetKind::Compute,
        &[RenderShaderEntryPointDescriptor {
            name: HZB_BUILD_KERNEL.to_string(),
            stage: RenderShaderStage::Compute,
        }],
        &[
            resource(
                HZB_SCENE_DEPTH_RESOURCE,
                ShaderResourceKind::Texture,
                ShaderResourceAccess::Read,
            ),
            resource(
                HZB_SOURCE_RESOURCE,
                ShaderResourceKind::Texture,
                ShaderResourceAccess::Read,
            ),
            resource(
                HZB_TARGET_RESOURCE,
                ShaderResourceKind::StorageTexture,
                ShaderResourceAccess::Write,
            ),
        ],
    )
    .expect("builtin HZB compute dispatch contract must be valid")
}

pub(crate) fn motion_vector_tile_max_pass_plan() -> &'static FullscreenPassPlan {
    static PLAN: OnceLock<FullscreenPassPlan> = OnceLock::new();
    PLAN.get_or_init(|| {
        FullscreenPassBuilder::new(
            FullscreenShaderRef::from_locator_str(
                MOTION_VECTOR_TILE_MAX_SHADER,
                MOTION_VECTOR_TILE_MAX_FRAGMENT,
            )
            .expect("builtin motion-vector fullscreen shader locator must be valid"),
        )
        .with_pipeline_label(MOTION_VECTOR_TILE_MAX_PIPELINE_LABEL)
        .set_vec4(MOTION_VECTOR_TILE_SPAN_PARAMETER, [2.0, 2.0, 0.0, 0.0])
        .bind_texture(MOTION_VECTOR_SOURCE_RESOURCE)
        .build(
            ShaderAssetKind::Fullscreen,
            &[RenderShaderEntryPointDescriptor {
                name: MOTION_VECTOR_TILE_MAX_FRAGMENT.to_string(),
                stage: RenderShaderStage::Fragment,
            }],
            &[resource(
                MOTION_VECTOR_SOURCE_RESOURCE,
                ShaderResourceKind::Texture,
                ShaderResourceAccess::Read,
            )],
        )
        .expect("builtin motion-vector fullscreen contract must be valid")
    })
}

fn resource(
    name: &str,
    kind: ShaderResourceKind,
    access: ShaderResourceAccess,
) -> ShaderResourceDescriptor {
    ShaderResourceDescriptor {
        name: name.to_string(),
        kind,
        access: Some(access),
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        ShaderDispatchExtent, ShaderResourceAccess, ShaderResourceKind,
        COMPUTE_SHADER_PARAMS_BINDING, FULLSCREEN_PASS_INPUT_GROUP,
        FULLSCREEN_TRIANGLE_VERTEX_ENTRY,
    };

    use super::*;

    #[test]
    fn hzb_compute_contract_assigns_generated_parameter_and_resource_bindings() {
        let plan = hzb_build_dispatch_plan();

        assert_eq!(plan.pipeline_label, HZB_BUILD_PIPELINE_LABEL);
        assert_eq!(plan.workgroup_size, [8, 8, 1]);
        assert_eq!(plan.dispatch_extent, ShaderDispatchExtent::HzbFurthest);
        assert_eq!(COMPUTE_SHADER_PARAMS_BINDING.binding, 0);
        assert_eq!(
            plan.resources
                .iter()
                .map(|resource| (
                    resource.name.as_str(),
                    resource.kind,
                    resource.access,
                    resource.abi.binding,
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    HZB_SCENE_DEPTH_RESOURCE,
                    ShaderResourceKind::Texture,
                    ShaderResourceAccess::Read,
                    1,
                ),
                (
                    HZB_SOURCE_RESOURCE,
                    ShaderResourceKind::Texture,
                    ShaderResourceAccess::Read,
                    2,
                ),
                (
                    HZB_TARGET_RESOURCE,
                    ShaderResourceKind::StorageTexture,
                    ShaderResourceAccess::Write,
                    3,
                ),
            ]
        );
    }

    #[test]
    fn motion_vector_fullscreen_contract_uses_generated_triangle_and_pass_input_group() {
        let plan = motion_vector_tile_max_pass_plan();

        assert_eq!(plan.pipeline_label, MOTION_VECTOR_TILE_MAX_PIPELINE_LABEL);
        assert_eq!(plan.vertex_entry, FULLSCREEN_TRIANGLE_VERTEX_ENTRY);
        assert_eq!(plan.resources.len(), 1);
        assert_eq!(plan.resources[0].name, MOTION_VECTOR_SOURCE_RESOURCE);
        assert_eq!(plan.resources[0].abi.group, FULLSCREEN_PASS_INPUT_GROUP);
        assert_eq!(plan.resources[0].abi.binding, 0);
        assert_eq!(
            plan.parameters.get(MOTION_VECTOR_TILE_SPAN_PARAMETER),
            Some(
                &crate::core::framework::render::ShaderParameterValue::Vec4 {
                    value: [2.0, 2.0, 0.0, 0.0],
                }
            )
        );
    }
}
