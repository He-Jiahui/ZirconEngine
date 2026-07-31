use super::super::sources::HotspotInventorySources;

pub(super) fn assert_asset_and_animation_evidence(sources: &HotspotInventorySources) {
    for required_asset_worker_anchor in [
        "ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC",
        "ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC",
        "pub struct AssetWorkerPoolFrameSampler",
        "pub fn spawn_worker_pool_with_frame_sampler(",
        "AssetWorkerPoolFrameSampler::from_pool(&pool)",
        "pub completed_delta: u64",
        "pub fn sample(&mut self, pool: &AssetWorkerPool) -> AssetWorkerPoolFrameDiagnostics",
        "worker_pool_frame_sampler_records_per_frame_completion_deltas",
        "asset.worker.frame_completed",
    ] {
        assert!(
            sources
                .asset_worker_source
                .contains(required_asset_worker_anchor)
                || sources
                    .asset_worker_manager
                    .contains(required_asset_worker_anchor)
                || sources
                    .asset_worker_tests
                    .contains(required_asset_worker_anchor),
            "asset worker frame diagnostics should retain `{required_asset_worker_anchor}`"
        );
    }

    for required_animation_evaluation_anchor in [
        "pub const ANIMATION_EVALUATE_SYSTEM: &str = \"animation.evaluate\";",
        "runtime_scene_system(",
        "pub(crate) fn tick_animation_world(",
        "enqueue_clip_event_samples(level, scan.clip_event_samples);",
        "enqueue_clip_event_samples(level, graph_event_samples);",
        "publish_clip_events(asset_manager, level);",
        "level.drain_animation_clip_events(asset_manager)",
        "pub(crate) fn sample_clip_events_budgeted(",
        "AnimationClipEventSamplingCursor",
        "const ANIMATION_CLIP_EVENT_MAX_DRAIN_SAMPLES: usize = 32;",
        "pub fn enqueue_animation_clip_event_range(",
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
                    .contains(required_animation_evaluation_anchor),
            "animation evaluation should retain `{required_animation_evaluation_anchor}`"
        );
    }
}
