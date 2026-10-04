use super::*;
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    DrawInstanceSource, MeshDrawArgs, MeshGeometryHandle, MeshPassPipelineKind,
    MeshPipelineVariantId,
};

fn command(
    phase: RenderPhase,
    bucket_variant: u32,
    sort_key: u64,
    source_draw_index: usize,
) -> MeshDrawCommand {
    MeshDrawCommand::new(
        phase,
        MeshPassPipelineKind::Base,
        default_pipeline_key(),
        MeshPipelineVariantId::new(bucket_variant),
        sort_key,
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index: source_draw_index as u32,
            instance_count: 1,
        },
        MeshGeometryHandle::test(source_draw_index as u64 + 1),
        MeshDrawArgs::direct_indexed(0, 3),
    )
    .with_source_draw_index(source_draw_index)
}

#[test]
fn sealed_ranges_cover_each_command_once_in_bucket_order() {
    let mut arena = MeshPassCommandBuildArena::with_capacity(4);
    arena.push(command(RenderPhase::Transparent3d, 2, 20, 2));
    arena.push(command(RenderPhase::Opaque3d, 1, 30, 3));
    arena.push(command(RenderPhase::Shadow, 3, 10, 1));
    arena.push(command(RenderPhase::Prepass, 4, 5, 0));

    let sealed = arena.seal(true).expect("mesh command phases are supported");
    assert_eq!(sealed.commands().len(), 4);
    assert_eq!(sealed.ranges().total_len(), sealed.commands().len());
    let mut cursor = 0;
    for bucket in MeshPassCommandBucket::ALL {
        let range = sealed.ranges().range(bucket);
        assert_eq!(range.start(), cursor, "bucket ranges must be contiguous");
        cursor = range.end();
    }
    assert_eq!(cursor, sealed.commands().len());
    assert_eq!(sealed.bucket(MeshPassCommandBucket::DepthPrepass).len(), 1);
    assert_eq!(sealed.bucket(MeshPassCommandBucket::Shadow).len(), 1);
    assert_eq!(sealed.bucket(MeshPassCommandBucket::Opaque).len(), 1);
    assert_eq!(sealed.bucket(MeshPassCommandBucket::Transparent).len(), 1);
    assert_eq!(
        sealed
            .commands()
            .iter()
            .map(|command| command.phase)
            .collect::<Vec<_>>(),
        vec![
            RenderPhase::Prepass,
            RenderPhase::Shadow,
            RenderPhase::Opaque3d,
            RenderPhase::Transparent3d,
        ]
    );
}

#[test]
fn half_resolution_policy_is_decided_before_seal() {
    let command =
        command(RenderPhase::Transparent3d, 1, 0, 0).with_half_resolution_transparency(true);
    let mut dedicated = MeshPassCommandBuildArena::default();
    dedicated.push(command.clone());
    let dedicated = dedicated.seal(true).unwrap();
    assert_eq!(
        dedicated
            .bucket(MeshPassCommandBucket::HalfResolutionTransparent)
            .len(),
        1
    );

    let mut fallback = MeshPassCommandBuildArena::default();
    fallback.push(command);
    let fallback = fallback.seal(false).unwrap();
    assert_eq!(fallback.bucket(MeshPassCommandBucket::Transparent).len(), 1);
    assert!(fallback
        .bucket(MeshPassCommandBucket::HalfResolutionTransparent)
        .is_empty());
}

#[test]
fn checkpoints_and_cache_stats_survive_seal_without_recomputation() {
    let mut arena = MeshPassCommandBuildArena::default();
    arena.push(command(RenderPhase::Opaque3d, 1, 0, 0));
    let checkpoint = arena.checkpoint();
    arena.push(command(RenderPhase::Shadow, 2, 0, 1));
    arena
        .cache_stats_mut()
        .record_resolver_configuration_invalidation(5);
    arena.truncate_to_checkpoint(checkpoint);
    assert_eq!(
        arena
            .cache_stats_mut()
            .cache_invalidated_resolver_configuration_count,
        0
    );
    arena
        .cache_stats_mut()
        .record_resolver_configuration_invalidation(7);

    let sealed = arena.seal(true).unwrap();
    assert_eq!(sealed.commands().len(), 1);
    assert_eq!(
        sealed
            .cache_stats()
            .cache_invalidated_resolver_configuration_count,
        7
    );
}

#[test]
#[should_panic(expected = "mesh command arena checkpoint belongs to another arena")]
fn checkpoint_rejects_another_arena_owner() {
    let checkpoint = MeshPassCommandBuildArena::default().checkpoint();
    let mut another = MeshPassCommandBuildArena::default();
    another.truncate_to_checkpoint(checkpoint);
}

#[test]
fn unsupported_render_phases_fail_closed() {
    let mut arena = MeshPassCommandBuildArena::default();
    arena.push(command(RenderPhase::Ui, 1, 0, 0));

    let error = match arena.seal(true) {
        Ok(_) => panic!("UI commands do not belong to mesh arena"),
        Err(error) => error,
    };
    assert_eq!(error.unsupported_phase(), RenderPhase::Ui);
}

#[test]
fn sort_ties_preserve_producer_order_until_the_key_is_total() {
    let mut arena = MeshPassCommandBuildArena::default();
    arena.push(command(RenderPhase::Opaque3d, 2, 10, 2));
    arena.push(command(RenderPhase::Opaque3d, 1, 10, 1));
    arena.push(command(RenderPhase::Opaque3d, 1, 10, 0));

    let sealed = arena.seal(true).unwrap();
    assert_eq!(
        sealed
            .bucket(MeshPassCommandBucket::Opaque)
            .iter()
            .map(|command| (
                command.pipeline_variant_id.value(),
                command.source_draw_index
            ))
            .collect::<Vec<_>>(),
        vec![(2, 2), (1, 1), (1, 0)]
    );
}
