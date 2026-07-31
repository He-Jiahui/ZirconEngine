mod apply_binding;
mod executors;
mod integrate;
mod light_scatter;
mod media_inject;
mod resolved_settings;
mod temporal_reprojection;
mod view_reconstruction;

pub(crate) use temporal_reprojection::GpuFroxelTemporalReprojection;
pub(crate) use view_reconstruction::{FroxelViewReconstruction, GpuFroxelViewParams};

pub(crate) use apply_binding::{
    VOLUMETRIC_APPLY_PARAMS_BINDING, VOLUMETRIC_INTEGRATED_BINDING, VOLUMETRIC_SAMPLER_BINDING,
    VolumetricApplyFallbackResources, volumetric_apply_bind_group_layout_entries,
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
    FroxelLightScatterPipeline, FroxelLightScatterRequest, VOLUMETRIC_LIGHT_SCATTER_PIPELINE_LABEL,
    VOLUMETRIC_LIGHT_SCATTER_WORKGROUP_SIZE, volumetric_ambient_radiance,
};
pub(crate) use media_inject::{
    FroxelMediaInjectPipeline, FroxelMediaInjectRequest, VOLUMETRIC_MEDIA_INJECT_PIPELINE_LABEL,
    VOLUMETRIC_MEDIA_INJECT_WORKGROUP_SIZE,
};
