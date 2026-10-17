#[path = "capture_filter_wgpu.rs"]
mod environment_capture_filter_wgpu;
#[path = "capture_gpu_target.rs"]
mod environment_capture_gpu_target;
#[path = "capture_render_plan.rs"]
mod environment_capture_render_plan;
#[path = "capture_scene_batch.rs"]
mod environment_capture_scene_batch;
#[path = "capture_scene_uniforms.rs"]
mod environment_capture_scene_uniforms;
#[path = "../../environment/capture_source_submission.rs"]
mod environment_capture_source_submission;
#[path = "capture_wgpu_recorder.rs"]
mod environment_capture_wgpu_recorder;
#[path = "../../environment/ibl_bake_compute_executor.rs"]
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_compute_executor;
#[path = "ibl_bake_graph_plan.rs"]
pub(in crate::graphics) mod ibl_bake_graph_plan;
#[cfg(feature = "native-host")]
#[path = "../../environment/ibl_bake_runtime_writeback.rs"]
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_runtime_writeback;
#[path = "ibl_bake_shader_plan.rs"]
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_shader_plan;
#[path = "ibl_bake_wgpu_binding.rs"]
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_binding;
#[path = "../../environment/ibl_bake_wgpu_command_plan.rs"]
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_command_plan;
#[path = "ibl_bake_wgpu_dispatch.rs"]
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_dispatch;
#[path = "../../environment/ibl_bake_wgpu_pipeline_cache.rs"]
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_pipeline_cache;
#[path = "ibl_bake_wgpu_readback.rs"]
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_readback;
#[path = "lightmap_binding.rs"]
mod lightmap_binding;
#[path = "probe_buffer/mod.rs"]
mod probe_buffer;
#[path = "procedural_environment.rs"]
pub(crate) mod procedural_environment;
#[path = "../../environment/realtime_ibl_capture_wgpu.rs"]
pub(in crate::graphics) mod realtime_ibl_capture_wgpu;
#[path = "../../environment/realtime_ibl_cpu_timing.rs"]
pub(in crate::graphics) mod realtime_ibl_cpu_timing;
#[path = "../../environment/realtime_ibl_gpu_resources.rs"]
pub(in crate::graphics) mod realtime_ibl_gpu_resources;
#[path = "../../environment/realtime_ibl_gpu_timestamps.rs"]
pub(in crate::graphics) mod realtime_ibl_gpu_timestamps;
#[path = "../../environment/realtime_ibl_graph_plan.rs"]
pub(in crate::graphics) mod realtime_ibl_graph_plan;
#[cfg(test)]
#[path = "../../tests/realtime_ibl_profile_test_support.rs"]
pub(in crate::graphics::scene::scene_renderer) mod realtime_ibl_profile_test_support;
#[path = "realtime_ibl_runtime.rs"]
pub(in crate::graphics) mod realtime_ibl_runtime;
#[path = "../../environment/realtime_ibl_time_slice.rs"]
pub(in crate::graphics) mod realtime_ibl_time_slice;
#[path = "../../environment/realtime_ibl_wgpu_recorder.rs"]
pub(in crate::graphics) mod realtime_ibl_wgpu_recorder;
#[path = "../../environment/scene_bind_group_layout.rs"]
mod scene_bind_group_layout;
#[path = "shader_source_identity.rs"]
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
pub(in crate::graphics) use probe_buffer::ProbeCubemapSlotReservation;
pub(in crate::graphics::scene::scene_renderer) use probe_buffer::{
    reflection_probe_bind_group_layout_entries, ReflectionProbeGpuBindings,
    SceneReflectionProbeResources, PLANAR_REFLECTION_TEXTURE_SIZE,
};
pub use realtime_ibl_cpu_timing::RealtimeIblCpuTimingReport;
pub use realtime_ibl_gpu_timestamps::RealtimeIblGpuTimingReport;
pub(in crate::graphics::scene::scene_renderer) use realtime_ibl_runtime::{
    RealtimeIblCompiledGraphCacheStats, RealtimeIblPendingSubmission, RealtimeIblPreparedFrame,
    RealtimeIblRuntime,
};
pub(in crate::graphics::scene::scene_renderer) use scene_bind_group_layout::scene_bind_group_layout_entries;
