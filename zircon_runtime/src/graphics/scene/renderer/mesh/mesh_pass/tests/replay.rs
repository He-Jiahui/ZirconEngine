use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawArgs, MeshPassPipelineKind, MeshPipelineVariantId,
};

use super::{
    MeshDrawCommandReplayer, MeshDrawReplayStats, MeshDrawReplayStatsAccumulator,
    FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT, GPU_SCENE_BIND_GROUP_SLOT, MATERIAL_BIND_GROUP_SLOT,
};

#[test]
fn replay_stats_accumulator_is_sync_and_saturates() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<MeshDrawReplayStatsAccumulator>();

    let stats = MeshDrawReplayStatsAccumulator::default();
    stats.record(MeshDrawReplayStats {
        draw_call_count: u32::MAX,
        indirect_count_draw_call_count: u32::MAX,
        fixed_multi_draw_call_count: u32::MAX,
        per_draw_indirect_draw_call_count: u32::MAX,
        direct_draw_call_count: u32::MAX,
        state_change_count: 2,
        bind_skip_count: 3,
        material_bind_group_set_count: 4,
        material_bind_group_skip_count: 5,
    });
    stats.record(MeshDrawReplayStats {
        draw_call_count: 1,
        indirect_count_draw_call_count: 1,
        fixed_multi_draw_call_count: 1,
        per_draw_indirect_draw_call_count: 1,
        direct_draw_call_count: 1,
        state_change_count: u32::MAX,
        bind_skip_count: u32::MAX,
        material_bind_group_set_count: u32::MAX,
        material_bind_group_skip_count: u32::MAX,
    });

    assert_eq!(
        stats.stats(),
        MeshDrawReplayStats {
            draw_call_count: u32::MAX,
            indirect_count_draw_call_count: u32::MAX,
            fixed_multi_draw_call_count: u32::MAX,
            per_draw_indirect_draw_call_count: u32::MAX,
            direct_draw_call_count: u32::MAX,
            state_change_count: u32::MAX,
            bind_skip_count: u32::MAX,
            material_bind_group_set_count: u32::MAX,
            material_bind_group_skip_count: u32::MAX,
        }
    );
}

#[test]
fn mesh_draw_command_replayer_rebinds_after_external_pipeline() {
    let mut replayer = MeshDrawCommandReplayer::default();
    let variant = MeshPipelineVariantId::new(1);

    assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, variant));
    assert!(!replayer.should_set_pipeline(MeshPassPipelineKind::Base, variant));

    replayer.invalidate_state_after_external_pipeline();

    assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, variant));
}

#[test]
fn mesh_draw_command_replayer_counts_pipeline_changes() {
    let mut replayer = MeshDrawCommandReplayer::default();
    let base_variant = MeshPipelineVariantId::new(1);
    let shadow_variant = MeshPipelineVariantId::new(2);

    assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, base_variant));
    assert!(!replayer.should_set_pipeline(MeshPassPipelineKind::Base, base_variant));
    assert!(replayer.should_set_pipeline(MeshPassPipelineKind::ShadowDepth, shadow_variant));

    assert_eq!(replayer.stats().draw_call_count, 0);
    assert_eq!(replayer.stats().indirect_count_draw_call_count, 0);
    assert_eq!(replayer.stats().fixed_multi_draw_call_count, 0);
    assert_eq!(replayer.stats().per_draw_indirect_draw_call_count, 0);
    assert_eq!(replayer.stats().direct_draw_call_count, 0);
    assert_eq!(replayer.stats().state_change_count, 2);
    assert_eq!(replayer.stats().bind_skip_count, 0);
}

#[test]
fn mesh_draw_command_replayer_classifies_unbatched_direct_and_indirect_draws() {
    let mut replayer = MeshDrawCommandReplayer::default();
    let direct = MeshDrawArgs::direct_indexed(0, 3);
    let indirect = MeshDrawArgs::test_indexed_indirect(7, 0);

    replayer.record_unbatched_draw_call(&direct);
    replayer.record_unbatched_draw_call(&indirect);

    assert_eq!(replayer.stats().draw_call_count, 2);
    assert_eq!(replayer.stats().direct_draw_call_count, 1);
    assert_eq!(replayer.stats().per_draw_indirect_draw_call_count, 1);
    assert_eq!(replayer.stats().fixed_multi_draw_call_count, 0);
    assert_eq!(replayer.stats().indirect_count_draw_call_count, 0);
}

#[test]
fn mesh_draw_command_replayer_skips_redundant_tracked_bind_groups() {
    let mut replayer = MeshDrawCommandReplayer::default();

    assert!(replayer.should_bind_raw_group(2, 10));
    assert!(!replayer.should_bind_raw_group(2, 10));
    assert!(replayer.should_bind_raw_group(2, 11));
    assert!(replayer.should_bind_raw_group(6, 10));
    assert!(replayer.should_bind_raw_group(6, 10));

    assert_eq!(replayer.stats().bind_skip_count, 1);
}

#[test]
fn mesh_draw_command_replayer_does_not_alias_owned_and_borrowed_bind_ids() {
    let mut replayer = MeshDrawCommandReplayer::default();

    assert!(replayer.should_bind_mesh_group(GPU_SCENE_BIND_GROUP_SLOT, 10));
    assert!(replayer.should_bind_raw_group(GPU_SCENE_BIND_GROUP_SLOT, 10));
    assert!(!replayer.should_bind_raw_group(GPU_SCENE_BIND_GROUP_SLOT, 10));

    assert_eq!(replayer.stats().bind_skip_count, 1);
}

#[test]
fn mesh_draw_command_replayer_separates_material_bind_group_sets_and_skips() {
    let mut replayer = MeshDrawCommandReplayer::default();

    assert!(replayer.should_bind_raw_group(MATERIAL_BIND_GROUP_SLOT, 10));
    assert!(!replayer.should_bind_raw_group(MATERIAL_BIND_GROUP_SLOT, 10));
    assert!(replayer.should_bind_raw_group(MATERIAL_BIND_GROUP_SLOT, 11));
    assert!(replayer.should_bind_raw_group(FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT, 10));
    assert!(!replayer.should_bind_raw_group(FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT, 10));

    assert_eq!(replayer.stats().material_bind_group_set_count, 2);
    assert_eq!(replayer.stats().material_bind_group_skip_count, 1);
    assert_eq!(replayer.stats().bind_skip_count, 2);
}

#[test]
fn mesh_draw_command_replayer_resets_bind_tracking_on_pipeline_change() {
    let mut replayer = MeshDrawCommandReplayer::default();
    let base_variant = MeshPipelineVariantId::new(1);
    let shadow_variant = MeshPipelineVariantId::new(2);

    assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, base_variant));
    assert!(replayer.should_bind_raw_group(2, 10));
    assert!(!replayer.should_bind_raw_group(2, 10));
    assert!(replayer.should_set_pipeline(MeshPassPipelineKind::ShadowDepth, shadow_variant));
    assert!(replayer.should_bind_raw_group(2, 10));

    assert_eq!(replayer.stats().state_change_count, 2);
    assert_eq!(replayer.stats().bind_skip_count, 1);
}

#[test]
fn mesh_draw_command_replayer_tracks_forward_shadow_receiver_slot() {
    let mut replayer = MeshDrawCommandReplayer::default();
    let base_variant = MeshPipelineVariantId::new(1);

    assert!(replayer.should_bind_raw_group(FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT, 10));
    assert!(!replayer.should_bind_raw_group(FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT, 10));
    assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, base_variant));
    assert!(replayer.should_bind_raw_group(FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT, 10));
}

#[test]
fn mesh_draw_command_replayer_skips_redundant_geometry_until_pipeline_changes() {
    let mut replayer = MeshDrawCommandReplayer::default();
    let base_variant = MeshPipelineVariantId::new(1);
    let shadow_variant = MeshPipelineVariantId::new(2);

    assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, base_variant));
    assert!(replayer.should_bind_geometry((10, 20)));
    assert!(!replayer.should_bind_geometry((10, 20)));
    assert!(replayer.should_bind_geometry((10, 21)));
    assert!(replayer.should_set_pipeline(MeshPassPipelineKind::ShadowDepth, shadow_variant));
    assert!(replayer.should_bind_geometry((10, 21)));
}

#[test]
fn mesh_draw_command_replayer_records_multi_draw_indexed_indirect_batches() {
    let source = include_str!("../replay.rs");

    assert!(source.contains("pass.multi_draw_indexed_indirect"));
    assert!(source.contains("pass.multi_draw_indexed_indirect_count"));
    assert!(source.contains("execution.indirect_count_supported()"));
    assert!(source.contains("execution.multi_draw_indirect_supported()"));
    assert!(source.contains("record_indirect_count_draw_call"));
    assert!(source.contains("record_fixed_multi_draw_call"));
    assert!(source.contains("per_draw_indirect_draw_call_count"));
    assert!(source.contains("direct_draw_call_count"));
    assert!(source.contains("fn draw_indexed_indirect_range"));
    assert!(source.contains("compaction_ready_for_replay"));
    assert!(source.contains("visible_remap_scene_bind_group"));
    assert!(source.contains("batch.first_args"));
    assert!(source.contains("batch.args_count"));
    assert!(source.contains("batch.draw_count_index"));
}
