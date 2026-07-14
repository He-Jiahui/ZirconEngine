use crate::core::framework::render::{
    ComputeDispatchBuilder, ComputeDispatchPlan, ComputeKernelRef, PostProcessGraphResourceNames,
    RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind, ShaderDispatchExtent,
    ShaderResourceAccess, ShaderResourceDescriptor, ShaderResourceKind,
};

pub(super) const SSAO_PIPELINE_LABEL: &str = "zircon-ssao-pipeline";
pub(super) const CLUSTERED_LIGHTING_PIPELINE_LABEL: &str = "zircon-cluster-pipeline";
pub(super) const HZB_OCCLUSION_CULL_PIPELINE_LABEL: &str = "zircon-hzb-occlusion-cull-pipeline";
pub(super) const EXPOSURE_HISTOGRAM_PIPELINE_LABEL: &str = "zircon-exposure-histogram-pipeline";
pub(super) const EXPOSURE_RESOLVE_PIPELINE_LABEL: &str = "zircon-exposure-resolve-pipeline";
pub(super) const COLOR_LUT_BAKE_PIPELINE_LABEL: &str = "zircon-color-lut-bake-pipeline";
pub(super) const HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE: &str =
    "mesh.indirect-compaction-metadata";
pub(super) const HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE: &str =
    "mesh.compacted-indirect-args";
pub(super) const HZB_OCCLUSION_DRAW_COUNT_RESOURCE: &str = "mesh.indirect-draw-count";
pub(super) const HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE: &str = "mesh.indirect-args";
pub(super) const HZB_OCCLUSION_STATS_RESOURCE: &str = "visibility.hzb-occlusion-stats";
pub(super) const HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE: &str =
    "mesh.visible-instance-index";
pub(super) const SSAO_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
pub(super) const CLUSTERED_LIGHTING_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
pub(super) const HZB_OCCLUSION_CULL_WORKGROUP_SIZE: [u32; 3] = [64, 1, 1];
pub(super) const EXPOSURE_HISTOGRAM_WORKGROUP_SIZE: [u32; 3] = [16, 16, 1];
pub(super) const EXPOSURE_RESOLVE_WORKGROUP_SIZE: [u32; 3] = [1, 1, 1];
pub(super) const COLOR_LUT_BAKE_WORKGROUP_SIZE: [u32; 3] = [4, 4, 4];

const CLUSTERED_LIGHTING_SHADER: &str = "builtin://shaders/compute/clustered_lighting";
const CLUSTERED_LIGHTING_KERNEL: &str = "cs_main";

pub(super) fn clustered_lighting_dispatch_plan() -> ComputeDispatchPlan {
    let builder = ComputeDispatchBuilder::new(
        ComputeKernelRef::from_locator_str(CLUSTERED_LIGHTING_SHADER, CLUSTERED_LIGHTING_KERNEL)
            .expect("builtin clustered lighting compute shader locator must be valid"),
    )
    .with_pipeline_label(CLUSTERED_LIGHTING_PIPELINE_LABEL)
    .with_workgroup_size(CLUSTERED_LIGHTING_WORKGROUP_SIZE)
    .bind_storage_write(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
    .bind_storage_write(PostProcessGraphResourceNames::LIGHT_ZBINS)
    .bind_storage_write(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
    .bind_storage_write(PostProcessGraphResourceNames::LIGHT_LIST)
    .dispatch_extent(ShaderDispatchExtent::ClusterGrid);

    builder
        .build(
            ShaderAssetKind::Compute,
            &[RenderShaderEntryPointDescriptor {
                name: CLUSTERED_LIGHTING_KERNEL.to_string(),
                stage: RenderShaderStage::Compute,
            }],
            &[
                storage_write_resource(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS),
                storage_write_resource(PostProcessGraphResourceNames::LIGHT_ZBINS),
                storage_write_resource(PostProcessGraphResourceNames::LIGHT_TILE_MASKS),
                storage_write_resource(PostProcessGraphResourceNames::LIGHT_LIST),
            ],
        )
        .expect("builtin clustered lighting compute dispatch contract must be valid")
}

fn storage_write_resource(name: &str) -> ShaderResourceDescriptor {
    ShaderResourceDescriptor {
        name: name.to_string(),
        kind: ShaderResourceKind::StorageBuffer,
        access: Some(ShaderResourceAccess::Write),
    }
}
