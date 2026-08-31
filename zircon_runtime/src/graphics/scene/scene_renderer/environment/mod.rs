mod environment_capture_filter_wgpu;
mod environment_capture_gpu_target;
mod environment_capture_render_plan;
mod environment_capture_scene_batch;
mod environment_capture_scene_uniforms;
mod environment_capture_source_submission;
mod environment_capture_wgpu_recorder;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_compute_executor;
pub(in crate::graphics) mod ibl_bake_graph_plan;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_runtime_writeback;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_shader_plan;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_binding;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_command_plan;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_dispatch;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_pipeline_cache;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_readback;
mod lightmap_binding;
mod probe_buffer;
pub(crate) mod procedural_environment;
pub(in crate::graphics) mod realtime_ibl_capture_wgpu;
pub(in crate::graphics) mod realtime_ibl_cpu_timing;
pub(in crate::graphics) mod realtime_ibl_gpu_resources;
pub(in crate::graphics) mod realtime_ibl_gpu_timestamps;
pub(in crate::graphics) mod realtime_ibl_graph_plan;
#[cfg(test)]
pub(in crate::graphics::scene::scene_renderer) mod realtime_ibl_profile_test_support;
pub(in crate::graphics) mod realtime_ibl_runtime;
pub(in crate::graphics) mod realtime_ibl_time_slice;
pub(in crate::graphics) mod realtime_ibl_wgpu_recorder;
mod scene_bind_group_layout;
mod shader_source_identity;

pub use crate::core::framework::render::{
    RealtimeIblFailureKind, RealtimeIblFailureOperation, RealtimeIblFailureReport,
    RealtimeIblReadiness, RealtimeIblStatusReport,
};
pub(in crate::graphics) use environment_capture_filter_wgpu::{
    EnvironmentCaptureFilterWgpuRecordReport, EnvironmentCaptureFilterWgpuRecorder,
};
pub(in crate::graphics) use environment_capture_gpu_target::{
    EnvironmentCaptureGpuOutput, EnvironmentCaptureGpuTarget, EnvironmentCaptureGpuTargetPlan,
};
pub(in crate::graphics) use environment_capture_render_plan::{
    EnvironmentCaptureRenderPass, EnvironmentCaptureRenderPlan,
};
pub(in crate::graphics) use environment_capture_scene_batch::{
    EnvironmentCaptureSceneBatch, EnvironmentCaptureSceneView,
};
pub(in crate::graphics) use environment_capture_scene_uniforms::{
    EnvironmentCaptureLightGridPlan, EnvironmentCaptureLightGridWorkspace,
    EnvironmentCaptureSceneUniformPlan, EnvironmentCaptureSceneUniformWorkspace,
};
pub(in crate::graphics) use environment_capture_source_submission::{
    EnvironmentCapturePersistenceSubmission, EnvironmentCapturePersistenceSubmissionStatus,
    EnvironmentCaptureProbePublication, EnvironmentCaptureResidentOutput,
    EnvironmentCaptureSourceSubmission, EnvironmentCaptureSourceSubmissionStatus,
    EnvironmentCaptureSubmission,
};
pub(in crate::graphics) use environment_capture_wgpu_recorder::{
    EnvironmentCaptureWgpuRecordReport, EnvironmentCaptureWgpuRecorder,
};
pub(in crate::graphics::scene::scene_renderer) use ibl_bake_wgpu_pipeline_cache::IblBakeWgpuPipelineCache;
pub(in crate::graphics::scene::scene_renderer) use lightmap_binding::{
    lightmap_bind_group_layout_entries, LightmapGpuBindings, SceneLightmapResources,
};
pub(in crate::graphics) use probe_buffer::{
    reflection_probe_bind_group_layout_entries, ProbeCubemapSlotReservation,
    ReflectionProbeGpuBindings, SceneReflectionProbeResources, PLANAR_REFLECTION_TEXTURE_SIZE,
};
pub use realtime_ibl_cpu_timing::RealtimeIblCpuTimingReport;
pub use realtime_ibl_gpu_timestamps::RealtimeIblGpuTimingReport;
pub(in crate::graphics::scene::scene_renderer) use realtime_ibl_runtime::{
    RealtimeIblCompiledGraphCacheStats, RealtimeIblPendingSubmission, RealtimeIblPreparedFrame,
    RealtimeIblRuntime,
};
pub(in crate::graphics::scene::scene_renderer) use scene_bind_group_layout::scene_bind_group_layout_entries;
