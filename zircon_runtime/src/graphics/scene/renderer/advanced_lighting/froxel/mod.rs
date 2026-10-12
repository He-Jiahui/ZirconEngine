#[path = "../../froxel/apply_binding.rs"]
mod apply_binding;
#[path = "../../froxel/executors/mod.rs"]
mod executors;
#[path = "integrate.rs"]
mod integrate;
#[path = "../../froxel/light_scatter.rs"]
mod light_scatter;
#[path = "../../froxel/media_inject.rs"]
mod media_inject;
#[path = "../../froxel/resolved_settings.rs"]
mod resolved_settings;
#[path = "../../froxel/temporal_reprojection.rs"]
mod temporal_reprojection;
#[path = "../../froxel/view_reconstruction.rs"]
mod view_reconstruction;

pub(crate) use temporal_reprojection::GpuFroxelTemporalReprojection;
pub(crate) use view_reconstruction::{FroxelViewReconstruction, GpuFroxelViewParams};

pub(crate) use apply_binding::{
    volumetric_apply_bind_group_layout_entries, VolumetricApplyFallbackResources,
    VOLUMETRIC_APPLY_PARAMS_BINDING, VOLUMETRIC_INTEGRATED_BINDING, VOLUMETRIC_SAMPLER_BINDING,
};
pub(crate) use resolved_settings::{resolved_volumetric_fog_settings, volumetric_history_quality};

pub(crate) use executors::registrations as executor_registrations;
pub use executors::{
    VOLUMETRIC_INTEGRATE_EXECUTOR_ID, VOLUMETRIC_LIGHT_SCATTER_EXECUTOR_ID,
    VOLUMETRIC_MEDIA_INJECT_EXECUTOR_ID,
};
pub(crate) use integrate::{
    FroxelIntegratePipeline, FroxelIntegrateRequest, VOLUMETRIC_INTEGRATE_PIPELINE_LABEL,
    VOLUMETRIC_INTEGRATE_WORKGROUP_SIZE,
};
pub(crate) use light_scatter::{
    volumetric_ambient_radiance, FroxelLightScatterPipeline, FroxelLightScatterRequest,
    VOLUMETRIC_LIGHT_SCATTER_PIPELINE_LABEL, VOLUMETRIC_LIGHT_SCATTER_WORKGROUP_SIZE,
};
pub(crate) use media_inject::{
    FroxelMediaInjectPipeline, FroxelMediaInjectRequest, VOLUMETRIC_MEDIA_INJECT_PIPELINE_LABEL,
    VOLUMETRIC_MEDIA_INJECT_WORKGROUP_SIZE,
};

#[cfg(test)]
#[path = "../../froxel/tests/factory_coverage.rs"]
mod factory_coverage_tests;
