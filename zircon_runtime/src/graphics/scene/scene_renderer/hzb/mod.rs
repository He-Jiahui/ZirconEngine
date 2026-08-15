mod bind_group_cache;
mod hzb_occlusion_culler;
mod params_workspace;
mod phase_dispatch;
mod sampled_resource_identity;

pub(crate) use sampled_resource_identity::HzbSampledResourceIdentity;

// The parameter workspace and culler share this POD layout within the HZB domain.
pub(crate) use hzb_occlusion_culler::{
    hzb_occlusion_supported_by_limits, HzbOcclusionCuller,
    HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE, HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE,
    HZB_OCCLUSION_CULL_PIPELINE_LABEL, HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE,
    HZB_OCCLUSION_CULL_WORKGROUP_SIZE, HZB_OCCLUSION_DRAW_COUNT_RESOURCE,
    HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE, HZB_OCCLUSION_STATS_RESOURCE,
    HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
};
pub(self) use hzb_occlusion_culler::{
    HzbOcclusionCullParams, HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE,
};
