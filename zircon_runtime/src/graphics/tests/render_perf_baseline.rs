use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, OnceLock,
};

use crate::asset::pipeline::manager::{ProjectAssetManager, ProjectAssetManagerAccess};
use crate::core::framework::render::{
    CapturedFrame, CorePipelineKind, ProjectionMode, RenderFrameExtract, RenderFramework,
    RenderGraphParallelRecordingReport, RenderPipelineHandle, RenderStats, RenderSubmissionConfig,
    RenderViewportDescriptor, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::core::{TaskPool, TaskPoolDescriptor};
use crate::graphics::runtime::WgpuRenderFramework;
use crate::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutionContext,
    RenderPassExecutor, RenderPassExecutorRegistration, RenderPassRecordingPolicy, RenderPassStage,
    RenderPipelineAsset,
};
use crate::render_graph::QueueLane;
use crate::scene::world::World;

const STANDARD_VIEWPORT_SIZE: UVec2 = UVec2::new(320, 240);
const MAX_GRAPH_PASS_COUNT: usize = 64;
const MAX_GRAPH_RESOURCE_LIFETIME_COUNT: usize = 256;
const MAX_TRANSIENT_DENSE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_STAGING_BYTES: u64 = 1024 * 1024;
const MAX_EMPTY_SCENE_DRAW_COUNT: usize = 0;
const MAX_EMPTY_SCENE_STATE_CHANGE_COUNT: usize = 0;
const MAX_PIPELINE_ASYNC_PENDING_COUNT: u32 = 0;
const PARALLEL_RECORDING_TEST_POOL_NAME: &str = "render-perf-parallel";
const PARALLEL_RECORDING_TEST_PIPELINE_HANDLE: RenderPipelineHandle =
    RenderPipelineHandle::new(701);

static STANDARD_EMPTY_SCENE_FRAMES: OnceLock<(RenderStats, RenderStats)> = OnceLock::new();
static SUBMISSION_MODE_FRAMES: OnceLock<Vec<SubmissionModeFrame>> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
struct SubmissionModeFrame {
    config: RenderSubmissionConfig,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

struct ParallelRecordingProductFrame {
    stats: RenderStats,
    captured: CapturedFrame,
    trace: Arc<ParallelRecordingTrace>,
}

#[derive(Default)]
struct ParallelRecordingTrace {
    worker_record_count: AtomicUsize,
    non_worker_record_count: AtomicUsize,
}

impl ParallelRecordingTrace {
    fn worker_record_count(&self) -> usize {
        self.worker_record_count.load(Ordering::Relaxed)
    }

    fn non_worker_record_count(&self) -> usize {
        self.non_worker_record_count.load(Ordering::Relaxed)
    }
}

struct ParallelRecordingExecutor {
    trace: Arc<ParallelRecordingTrace>,
}

impl RenderPassExecutor for ParallelRecordingExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        let is_worker = std::thread::current()
            .name()
            .is_some_and(|name| name.starts_with(PARALLEL_RECORDING_TEST_POOL_NAME));
        let counter = if is_worker {
            &self.trace.worker_record_count
        } else {
            &self.trace.non_worker_record_count
        };
        counter.fetch_add(1, Ordering::Relaxed);

        let pass_name = context.pass_name.clone();
        context
            .require_gpu()?
            .encoder
            .insert_debug_marker(&pass_name);
        Ok(())
    }

    fn recording_policy(&self) -> RenderPassRecordingPolicy {
        RenderPassRecordingPolicy::ParallelSafe
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderPerfBaselineViolation {
    DrawCount { actual: usize, maximum: usize },
    StateChangeCount { actual: usize, maximum: usize },
    GraphPassCount { actual: usize, maximum: usize },
    GraphResourceLifetimeCount { actual: usize, maximum: usize },
    TransientDenseBytes { actual: u64, maximum: u64 },
    StagingBytes { actual: u64, maximum: u64 },
    PipelineAsyncPendingCount { actual: u32, maximum: u32 },
}

fn validate_standard_empty_scene_baseline(
    stats: &RenderStats,
) -> Result<(), RenderPerfBaselineViolation> {
    validate_maximum(
        stats.last_mesh_draw_count,
        MAX_EMPTY_SCENE_DRAW_COUNT,
        |actual, maximum| RenderPerfBaselineViolation::DrawCount { actual, maximum },
    )?;
    validate_maximum(
        stats.last_mesh_replay_state_change_count,
        MAX_EMPTY_SCENE_STATE_CHANGE_COUNT,
        |actual, maximum| RenderPerfBaselineViolation::StateChangeCount { actual, maximum },
    )?;
    validate_maximum(
        stats.last_graph_pass_count,
        MAX_GRAPH_PASS_COUNT,
        |actual, maximum| RenderPerfBaselineViolation::GraphPassCount { actual, maximum },
    )?;
    validate_maximum(
        stats.last_graph_resource_lifetime_count,
        MAX_GRAPH_RESOURCE_LIFETIME_COUNT,
        |actual, maximum| RenderPerfBaselineViolation::GraphResourceLifetimeCount {
            actual,
            maximum,
        },
    )?;
    validate_maximum(
        stats.last_graph_transient_dense_bytes_reserved,
        MAX_TRANSIENT_DENSE_BYTES,
        |actual, maximum| RenderPerfBaselineViolation::TransientDenseBytes { actual, maximum },
    )?;
    validate_maximum(
        stats.last_frame_profile.staging_total_bytes,
        MAX_STAGING_BYTES,
        |actual, maximum| RenderPerfBaselineViolation::StagingBytes { actual, maximum },
    )?;
    validate_maximum(
        stats.last_pipeline_async_pending_count,
        MAX_PIPELINE_ASYNC_PENDING_COUNT,
        |actual, maximum| RenderPerfBaselineViolation::PipelineAsyncPendingCount {
            actual,
            maximum,
        },
    )
}

fn validate_maximum<T: Copy + PartialOrd, E>(
    actual: T,
    maximum: T,
    violation: impl FnOnce(T, T) -> E,
) -> Result<(), E> {
    if actual <= maximum {
        Ok(())
    } else {
        Err(violation(actual, maximum))
    }
}

#[test]
fn render_perf_standard_empty_scene_stays_within_deterministic_baseline() {
    let (first, second) = standard_empty_scene_frames();

    validate_standard_empty_scene_baseline(&second)
        .expect("standard empty scene deterministic counts must remain within baseline");
    assert_eq!(first.last_graph_compiled_cache_miss_count, 1);
    assert_eq!(
        second.last_graph_compiled_cache_miss_count,
        first.last_graph_compiled_cache_miss_count
    );
    assert_eq!(
        second.last_graph_compiled_cache_hit_count,
        first.last_graph_compiled_cache_hit_count + 1
    );
    assert!(second.last_frame_profile.compiled_graph_cache_hit);
    assert_eq!(second.last_gpu_scene_uploaded_bytes, 0);
    assert_eq!(second.last_variant_first_frame_miss_count, 0);
}

#[test]
fn render_perf_draw_count_baseline() {
    let (_, second) = standard_empty_scene_frames();
    assert!(second.last_mesh_draw_count <= MAX_EMPTY_SCENE_DRAW_COUNT);
}

#[test]
fn render_perf_state_change_baseline() {
    let (_, second) = standard_empty_scene_frames();
    assert!(second.last_mesh_replay_state_change_count <= MAX_EMPTY_SCENE_STATE_CHANGE_COUNT);
}

#[test]
fn render_perf_upload_bytes_static_second_frame_zero() {
    let (_, second) = standard_empty_scene_frames();
    assert_eq!(second.last_gpu_scene_uploaded_bytes, 0);
    assert_eq!(second.last_frame_profile.staging_total_bytes, 0);
}

#[test]
fn render_perf_transient_peak_baseline() {
    let (_, second) = standard_empty_scene_frames();
    assert!(second.last_graph_transient_dense_bytes_reserved <= MAX_TRANSIENT_DENSE_BYTES);
}

#[test]
fn render_perf_cold_start_graph_compile_once() {
    let (first, second) = standard_empty_scene_frames();
    assert_eq!(first.last_graph_compiled_cache_miss_count, 1);
    assert_eq!(
        second.last_graph_compiled_cache_miss_count,
        first.last_graph_compiled_cache_miss_count
    );
    assert_eq!(
        second.last_graph_compiled_cache_hit_count,
        first.last_graph_compiled_cache_hit_count + 1
    );
    assert!(second.last_frame_profile.compiled_graph_cache_hit);
}

#[test]
fn render_perf_pipelined_product_parity() {
    let frames = submission_mode_frames();
    let synchronous = frame_for_config(frames, RenderSubmissionConfig::synchronous());
    let pipelined = frame_for_config(frames, RenderSubmissionConfig::pipelined());

    assert_eq!(pipelined.width, synchronous.width);
    assert_eq!(pipelined.height, synchronous.height);
    assert_eq!(pipelined.rgba, synchronous.rgba);
}

#[test]
fn render_perf_submission_mode_matrix_product_parity() {
    let frames = submission_mode_frames();
    let baseline = frames
        .first()
        .expect("submission mode matrix should contain the synchronous baseline");

    assert_eq!(frames.len(), 8);
    for frame in frames.iter().skip(1) {
        assert_eq!(frame.width, baseline.width, "config={:?}", frame.config);
        assert_eq!(frame.height, baseline.height, "config={:?}", frame.config);
        assert_eq!(frame.rgba, baseline.rgba, "config={:?}", frame.config);
    }
}

#[test]
fn render_perf_parallel_recording_product_path_preserves_topology_and_pixels() {
    let serial = render_parallel_recording_product_frame(false);
    let parallel = render_parallel_recording_product_frame(true);

    assert_eq!(serial.captured.width, parallel.captured.width);
    assert_eq!(serial.captured.height, parallel.captured.height);
    assert_eq!(serial.captured.rgba, parallel.captured.rgba);
    assert_parallel_recording_pass_order(&serial.stats);
    assert_parallel_recording_pass_order(&parallel.stats);
    assert_eq!(serial.trace.worker_record_count(), 0);
    assert_eq!(serial.trace.non_worker_record_count(), 2);
    assert_eq!(
        serial.stats.last_graph_parallel_recording_report,
        RenderGraphParallelRecordingReport::default()
    );
    assert!(parallel.trace.worker_record_count() > 0);
    assert_eq!(
        parallel.trace.worker_record_count() + parallel.trace.non_worker_record_count(),
        2
    );
    assert_eq!(
        parallel.stats.last_graph_parallel_recording_report,
        RenderGraphParallelRecordingReport::new(1, 2, 1, 2)
    );
}

#[test]
fn render_perf_draw_count_baseline_rejects_plus_one_regression() {
    let mut stats = RenderStats::default();
    stats.last_mesh_draw_count = MAX_EMPTY_SCENE_DRAW_COUNT + 1;

    assert_eq!(
        validate_standard_empty_scene_baseline(&stats),
        Err(RenderPerfBaselineViolation::DrawCount {
            actual: MAX_EMPTY_SCENE_DRAW_COUNT + 1,
            maximum: MAX_EMPTY_SCENE_DRAW_COUNT,
        })
    );
}

#[cfg(feature = "dynamic-api")]
#[test]
fn render_perf_prewarm_zero_first_frame_miss() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let manifest = crate::dynamic_api::builtin_fallback_shader_prewarm_manifest();

    let first = framework.prewarm_shader_pipelines(&manifest).unwrap();
    assert!(first.requested_count() > 0);
    assert_eq!(first.ready_count(), first.requested_count());
    assert_eq!(first.cache_hit_count(), 0);
    assert_eq!(first.failed_count(), 0, "failures={:?}", first.failures());

    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(STANDARD_VIEWPORT_SIZE))
        .unwrap();
    let mut first_extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    first_extract.apply_viewport_size(STANDARD_VIEWPORT_SIZE);
    framework
        .submit_frame_extract(viewport, first_extract)
        .unwrap();
    let first_stats = framework.query_stats().unwrap();
    let first_creation_metrics = &first_stats.last_shader_variant_miss_report;
    assert!(first_creation_metrics.render_pipeline_creation_count > 0);
    assert!(first_creation_metrics.shader_module_creation_count > 0);
    assert_eq!(
        first_creation_metrics.render_pipeline_creation_count,
        first_creation_metrics.cached_render_pipeline_count
    );

    let repeated = framework.prewarm_shader_pipelines(&manifest).unwrap();
    assert_eq!(repeated.ready_count(), repeated.requested_count());
    assert_eq!(repeated.cache_hit_count(), repeated.requested_count());
    assert_eq!(repeated.failed_count(), 0);

    let mut repeated_extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(2),
        World::new().to_render_snapshot(),
    );
    repeated_extract.apply_viewport_size(STANDARD_VIEWPORT_SIZE);
    framework
        .submit_frame_extract(viewport, repeated_extract)
        .unwrap();
    let repeated_stats = framework.query_stats().unwrap();
    let repeated_creation_metrics = &repeated_stats.last_shader_variant_miss_report;
    assert_eq!(
        repeated_creation_metrics.render_pipeline_creation_count,
        first_creation_metrics.render_pipeline_creation_count
    );
    assert_eq!(
        repeated_creation_metrics.shader_module_creation_count,
        first_creation_metrics.shader_module_creation_count
    );
    assert_eq!(
        repeated_creation_metrics.render_pipeline_creation_cpu_microseconds,
        first_creation_metrics.render_pipeline_creation_cpu_microseconds
    );
    assert_eq!(
        repeated_creation_metrics.shader_module_creation_cpu_microseconds,
        first_creation_metrics.shader_module_creation_cpu_microseconds
    );
    assert_eq!(repeated_stats.last_variant_first_frame_miss_count, 0);
}

fn standard_empty_scene_frames() -> &'static (RenderStats, RenderStats) {
    STANDARD_EMPTY_SCENE_FRAMES.get_or_init(render_two_standard_empty_scene_frames)
}

fn submission_mode_frames() -> &'static [SubmissionModeFrame] {
    SUBMISSION_MODE_FRAMES.get_or_init(render_submission_mode_matrix)
}

fn frame_for_config(
    frames: &[SubmissionModeFrame],
    config: RenderSubmissionConfig,
) -> &SubmissionModeFrame {
    frames
        .iter()
        .find(|frame| frame.config == config)
        .expect("submission mode matrix should contain the requested configuration")
}

fn render_submission_mode_matrix() -> Vec<SubmissionModeFrame> {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let mut frames = Vec::with_capacity(8);
    for pipelined_render in [false, true] {
        for parallel_record in [false, true] {
            for async_pipeline_compile in [false, true] {
                let config = if pipelined_render {
                    RenderSubmissionConfig::pipelined()
                } else {
                    RenderSubmissionConfig::synchronous()
                };
                let config = if parallel_record {
                    config.with_parallel_recording(1)
                } else {
                    config
                };
                let config = if async_pipeline_compile {
                    config.with_async_pipeline_compile()
                } else {
                    config
                };
                framework.set_submission_config(config).unwrap();
                let viewport = framework
                    .create_viewport(RenderViewportDescriptor::new(STANDARD_VIEWPORT_SIZE))
                    .unwrap();
                let mut extract = RenderFrameExtract::from_snapshot(
                    RenderWorldSnapshotHandle::new(1),
                    World::new().to_render_snapshot(),
                );
                extract.apply_viewport_size(STANDARD_VIEWPORT_SIZE);
                framework.submit_frame_extract(viewport, extract).unwrap();
                let CapturedFrame {
                    width,
                    height,
                    rgba,
                    ..
                } = framework
                    .capture_frame(viewport)
                    .unwrap()
                    .expect("standard empty-scene frame should be capturable");
                frames.push(SubmissionModeFrame {
                    config,
                    width,
                    height,
                    rgba,
                });
            }
        }
    }
    frames
}

fn render_parallel_recording_product_frame(
    parallel_recording: bool,
) -> ParallelRecordingProductFrame {
    let trace = Arc::new(ParallelRecordingTrace::default());
    let framework =
        WgpuRenderFramework::new_with_plugin_render_extensions_and_solari_and_compute_task_pool(
            ProjectAssetManagerAccess::for_test(Arc::new(ProjectAssetManager::default())),
            [parallel_recording_feature_descriptor()],
            parallel_recording_executor_registrations(Arc::clone(&trace)),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            TaskPool::new(
                TaskPoolDescriptor::compute()
                    .with_worker_threads(2)
                    .with_thread_name(PARALLEL_RECORDING_TEST_POOL_NAME),
            ),
        )
        .expect("parallel recording product framework");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(STANDARD_VIEWPORT_SIZE))
        .expect("parallel recording product viewport");
    let mut pipeline = RenderPipelineAsset::default_core2d();
    pipeline.handle = PARALLEL_RECORDING_TEST_PIPELINE_HANDLE;
    pipeline.name = "parallel-recording-product".to_string();
    pipeline.apply_plugin_render_features([parallel_recording_feature_descriptor()]);
    let pipeline = framework
        .register_pipeline_asset(pipeline)
        .expect("parallel recording product pipeline registration");
    framework
        .reload_pipeline(pipeline)
        .expect("parallel recording product pipeline reload");
    framework
        .set_pipeline_asset(viewport, pipeline)
        .expect("parallel recording product pipeline selection");
    let config = if parallel_recording {
        RenderSubmissionConfig::synchronous().with_parallel_recording(1)
    } else {
        RenderSubmissionConfig::synchronous()
    };
    framework
        .set_submission_config(config)
        .expect("parallel recording product configuration");

    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.view.camera.core_pipeline = CorePipelineKind::Core2d;
    extract.view.camera.projection_mode = ProjectionMode::Orthographic;
    extract.view.core_pipeline = CorePipelineKind::Core2d;
    for descriptor in &mut extract.view.cameras {
        descriptor.camera = extract.view.camera.clone();
    }
    extract.apply_viewport_size(STANDARD_VIEWPORT_SIZE);
    framework
        .submit_frame_extract(viewport, extract)
        .expect("parallel recording product submit");
    let captured = framework
        .capture_frame(viewport)
        .expect("parallel recording product capture request")
        .expect("parallel recording product frame should be capturable");

    ParallelRecordingProductFrame {
        stats: framework
            .query_stats()
            .expect("parallel recording product statistics"),
        captured,
        trace,
    }
}

fn parallel_recording_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "render_perf_parallel_recording",
        Vec::new(),
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque2d,
                "parallel-recording-producer",
                QueueLane::Graphics,
            )
            .with_executor_id("render-perf.parallel-recording-producer")
            .write_buffer_with_minimum_size("parallel-recording-packet", 4)
            .with_side_effects(),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque2d,
                "parallel-recording-consumer",
                QueueLane::Graphics,
            )
            .with_executor_id("render-perf.parallel-recording-consumer")
            .read_buffer("parallel-recording-packet")
            .with_side_effects(),
        ],
    )
}

fn parallel_recording_executor_registrations(
    trace: Arc<ParallelRecordingTrace>,
) -> Vec<RenderPassExecutorRegistration> {
    [
        "render-perf.parallel-recording-producer",
        "render-perf.parallel-recording-consumer",
    ]
    .into_iter()
    .map(|executor_id| {
        RenderPassExecutorRegistration::new_executor(
            executor_id,
            Arc::new(ParallelRecordingExecutor {
                trace: Arc::clone(&trace),
            }),
        )
    })
    .collect()
}

fn assert_parallel_recording_pass_order(stats: &RenderStats) {
    let producer = stats
        .last_graph_executed_passes
        .iter()
        .position(|pass| pass == "parallel-recording-producer")
        .expect("parallel recording producer should execute");
    let consumer = stats
        .last_graph_executed_passes
        .iter()
        .position(|pass| pass == "parallel-recording-consumer")
        .expect("parallel recording consumer should execute");
    assert!(producer < consumer);
    assert_eq!(
        stats.last_graph_executed_executor_ids[producer],
        "render-perf.parallel-recording-producer"
    );
    assert_eq!(
        stats.last_graph_executed_executor_ids[consumer],
        "render-perf.parallel-recording-consumer"
    );
}

fn render_two_standard_empty_scene_frames() -> (RenderStats, RenderStats) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(STANDARD_VIEWPORT_SIZE))
        .unwrap();
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.apply_viewport_size(STANDARD_VIEWPORT_SIZE);

    framework
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    let first = framework.query_stats().unwrap();
    framework.submit_frame_extract(viewport, extract).unwrap();
    let second = framework.query_stats().unwrap();
    (first, second)
}
