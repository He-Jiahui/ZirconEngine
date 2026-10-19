//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。保存同组守卫使用的局部数据或辅助变换。
use super::super::sources::HotspotInventorySources;

pub(super) fn assert_asset_and_animation_evidence(sources: &HotspotInventorySources) {
    for (owner, source, anchors) in [
        ("worker facade", sources.asset_worker_source, &["mod diagnostics;", "AssetWorkerPoolFrameSampler"][..]),
        ("worker frame diagnostics", sources.asset_worker_frame_diagnostics, &[
            "ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC",
            "ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC",
            "pub struct AssetWorkerPoolFrameSampler",
            "pub completed_delta: u64",
            "pub fn sample(&mut self, pool: &AssetWorkerPool) -> AssetWorkerPoolFrameDiagnostics",
            "asset.worker.frame_completed",
        ][..]),
        ("worker manager", sources.asset_worker_manager, &[
            "pub fn spawn_worker_pool_with_frame_sampler(",
            "AssetWorkerPoolFrameSampler::from_pool(&pool)",
        ][..]),
        ("worker test mount", sources.asset_worker_tests, &["mod diagnostics;"][..]),
        ("worker frame regression", sources.asset_worker_frame_tests, &[
            "fn worker_pool_frame_sampler_records_per_job_completion_deltas()",
            "assert_eq!(first_frame.completed_delta, 2);",
            "assert_eq!(first_frame.failed_delta, 1);",
            "assert_eq!(second_frame.completed_delta, 0);",
            "assert_eq!(second_frame.failed_delta, 0);",
            "vec![2.0, 0.0]",
            "vec![1.0, 0.0]",
        ][..]),
    ] {
        for anchor in anchors {
            assert!(source.contains(anchor), "{owner} missing {anchor}");
        }
    }

    for required_animation_evaluation_anchor in [
        "pub const ANIMATION_EVALUATE_SYSTEM: &str = \"animation.evaluate\";",
        "runtime_scene_system(",
        "pub(crate) fn tick_animation_world(",
        "publish_clip_events(asset_manager, level, replacement_epoch)",
        "ProjectAnimationClipEventSampler::new(asset_manager)",
        "level.drain_animation_clip_events(replacement_epoch, &sampler)",
        "fn sample_clip_events_budgeted(",
        "AnimationClipEventSamplingCursor",
        "const ANIMATION_CLIP_EVENT_MAX_DRAIN_SAMPLES: usize = 32;",
        "const ANIMATION_CLIP_EVENT_MAX_PENDING_SAMPLES: usize =",
        "with_world_mut_if_replacement_epoch",
        "pub fn enqueue_animation_clip_event_range_batches(",
        "pub fn drain_animation_clip_events(",
    ] {
        assert!(
            sources
                .animation_plugin_runtime_system
                .contains(required_animation_evaluation_anchor)
                || sources
                    .animation_plugin_tick
                    .contains(required_animation_evaluation_anchor)
                || sources
                    .animation_plugin_events
                    .contains(required_animation_evaluation_anchor)
                || sources
                    .animation_clip_events
                    .contains(required_animation_evaluation_anchor)
                || sources
                    .animation_level_system
                    .contains(required_animation_evaluation_anchor)
                || sources
                    .animation_level_runtime
                    .contains(required_animation_evaluation_anchor),
            "animation evaluation should retain `{required_animation_evaluation_anchor}`"
        );
    }
    let tick_tokens = sources
        .animation_plugin_tick
        .split_whitespace()
        .collect::<String>();
    for pending in ["scan.clip_event_samples", "pending_clip_event_samples"] {
        let call = format!(
            "enqueue_clip_event_samples(level,replacement_epoch,clip_event_admission_cursor,{pending},)"
        );
        assert!(
            tick_tokens.contains(&call),
            "animation cursor admission missing {pending}"
        );
    }
    for anchor in [
        "pub(super) struct AnimationSceneFrameDiagnostics",
        "pub(super) fn from_scan(scan: &AnimationSceneScan) -> Self",
        "pub(super) fn record(self, core: &CoreHandle)",
        "animation.scene.scanned_entities",
        "animation.scene.sequence_samples",
        "animation.scene.clip_pose_samples",
        "animation.scene.clip_event_samples",
        "animation.scene.graph_pose_samples",
        "animation.scene.state_machine_pose_samples",
        "animation.scene.output_poses",
        "animation.scene.applied_transforms",
        "animation.scene.published_events",
        "animation.scene.state_transitions",
    ] {
        assert!(
            sources.animation_plugin_diagnostics.contains(anchor),
            "animation counters missing {anchor}"
        );
    }
    for anchor in [
        "AnimationSceneFrameDiagnostics::from_scan(&scan)",
        "frame_diagnostics.record(core)",
    ] {
        assert!(
            sources.animation_plugin_tick.contains(anchor),
            "animation counter publication missing {anchor}"
        );
    }
}
