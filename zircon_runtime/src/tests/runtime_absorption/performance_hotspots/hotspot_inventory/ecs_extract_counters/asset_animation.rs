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

    for required_animation_scene_anchor in [
        "ANIMATION_SCENE_SCANNED_ENTITIES_DIAGNOSTIC",
        "animation.scene.scanned_entities",
        "animation.scene.sequence_samples",
        "animation.scene.output_poses",
        "animation.scene.applied_transforms",
        "animation.scene.published_events",
        "animation.scene.state_transitions",
        "pub(super) struct AnimationSceneFrameDiagnostics",
        "pub(super) fn from_scan(scan: &AnimationSceneScan) -> Self",
        "pub(super) fn record(self, core: &CoreHandle)",
        "scanned_entities: entity_ids.len()",
        "let event_count = events.len();",
        "let update_count = updates.len();",
        "AnimationSceneFrameDiagnostics::from_scan(&scan)",
        "frame_diagnostics.published_events += publish_events(level, graph_events);",
        "frame_diagnostics.applied_transforms =",
        "frame_diagnostics.state_transitions = transition_updates.len();",
        "AnimationSceneFrameDiagnostics::default().record(core);",
    ] {
        assert!(
            sources
                .animation_scene_diagnostics
                .contains(required_animation_scene_anchor)
                || sources
                    .animation_scene_events
                    .contains(required_animation_scene_anchor)
                || sources
                    .animation_scene_node_pose
                    .contains(required_animation_scene_anchor)
                || sources
                    .animation_scene_pending
                    .contains(required_animation_scene_anchor)
                || sources
                    .animation_scene_scan
                    .contains(required_animation_scene_anchor)
                || sources
                    .animation_scene_tick
                    .contains(required_animation_scene_anchor),
            "animation scene diagnostics should retain `{required_animation_scene_anchor}`"
        );
    }
}
