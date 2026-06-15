pub(super) const SSAO_PIPELINE_LABEL: &str = "zircon-ssao-pipeline";
pub(super) const CLUSTERED_LIGHTING_PIPELINE_LABEL: &str = "zircon-cluster-pipeline";
pub(super) const HZB_BUILD_PIPELINE_LABEL: &str = "zircon-hzb-build-pipeline";
pub(super) const HZB_OCCLUSION_CULL_PIPELINE_LABEL: &str = "zircon-hzb-occlusion-cull-pipeline";
pub(super) const EXPOSURE_HISTOGRAM_PIPELINE_LABEL: &str = "zircon-exposure-histogram-pipeline";
pub(super) const EXPOSURE_RESOLVE_PIPELINE_LABEL: &str = "zircon-exposure-resolve-pipeline";
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
pub(super) const HZB_BUILD_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
pub(super) const HZB_OCCLUSION_CULL_WORKGROUP_SIZE: [u32; 3] = [64, 1, 1];
pub(super) const EXPOSURE_HISTOGRAM_WORKGROUP_SIZE: [u32; 3] = [16, 16, 1];
pub(super) const EXPOSURE_RESOLVE_WORKGROUP_SIZE: [u32; 3] = [1, 1, 1];
