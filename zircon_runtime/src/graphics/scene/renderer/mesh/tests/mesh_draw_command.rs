use std::sync::Arc;

use crate::core::framework::render::{packed_sort_key_u64, RenderPhase, RenderPhaseSortComponents};
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    DrawInstanceSource, MeshDrawArgs, MeshGeometryHandle, MeshPassPipelineKind,
    MeshPipelineVariantId,
};

use super::{next_mesh_bind_handle_id, MeshBindHandle, MeshDrawCommand};

#[test]
fn mesh_bind_handle_ids_are_nonzero_unique_and_stable_across_clone() {
    let first = next_mesh_bind_handle_id();
    let second = next_mesh_bind_handle_id();
    let handle = MeshBindHandle::test(first);

    assert_ne!(first, 0);
    assert_ne!(second, 0);
    assert_ne!(first, second);
    assert_eq!(handle.id(), handle.clone().id());
}

#[test]
fn velocity_geometry_bind_key_includes_previous_geometry_slot() {
    let velocity = command(MeshPassPipelineKind::Velocity)
        .with_previous_velocity_geometry(MeshGeometryHandle::test(20));
    let base = command(MeshPassPipelineKind::Base)
        .with_previous_velocity_geometry(MeshGeometryHandle::test(20));

    assert_eq!(velocity.geometry_bind_key(), (10, 20));
    assert_eq!(base.geometry_bind_key(), (10, 0));
}

#[test]
fn uncached_indirect_command_keeps_submission_payload_inline() {
    let command = MeshDrawCommand::new(
        RenderPhase::Opaque3d,
        MeshPassPipelineKind::Base,
        default_pipeline_key(),
        MeshPipelineVariantId::new(1),
        0,
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index: 0,
            instance_count: 1,
        },
        MeshGeometryHandle::test(10),
        MeshDrawArgs::test_indexed_indirect(91, 0),
    );

    assert!(!command.payload_is_shared());
}

#[test]
fn cached_visible_projection_refreshes_sort_and_gpu_scene_span() {
    let sort_components = RenderPhaseSortComponents::new(0.75, 33);
    let cached = command(MeshPassPipelineKind::Base)
        .with_source_entity(1)
        .with_source_draw_index(2)
        .with_material(MeshBindHandle::test(0x1_0003));
    let projected = MeshDrawCommand::from_cached_payload(
        cached.static_payload(),
        9,
        11,
        sort_components,
        (27, 4),
        None,
    );

    assert_eq!(projected.source_entity, 9);
    assert_eq!(projected.source_draw_index, 11);
    assert_eq!(
        projected.sort_key,
        packed_sort_key_u64(
            RenderPhase::PostProcess,
            sort_components,
            MeshPipelineVariantId::new(1).value(),
            3,
        )
    );
    match projected.instance_source {
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index,
            instance_count,
        } => {
            assert_eq!(first_instance_index, 27);
            assert_eq!(instance_count, 4);
        }
    }
    match projected.draw_args {
        MeshDrawArgs::DirectIndexed {
            first_instance,
            instance_count,
            ..
        } => {
            assert_eq!(first_instance, 27);
            assert_eq!(instance_count, 4);
        }
        MeshDrawArgs::IndexedIndirect { .. } => panic!("test command must be direct"),
    }
}

#[test]
fn cached_payload_reprojects_only_current_frame_visible_state() {
    let cached = command(MeshPassPipelineKind::Base)
        .with_material(MeshBindHandle::test(0x1_0003))
        .with_gpu_scene_bind_group(MeshBindHandle::test(41));
    let payload = cached.static_payload();
    let sort_components = RenderPhaseSortComponents::new(0.75, 33);

    let projected = MeshDrawCommand::from_cached_payload(
        payload.clone(),
        9,
        11,
        sort_components,
        (27, 4),
        Some(MeshBindHandle::test(99)),
    );

    assert!(Arc::ptr_eq(&payload, &projected.static_payload()));
    assert_eq!(projected.source_entity, 9);
    assert_eq!(projected.source_draw_index, 11);
    assert_eq!(
        projected.sort_key,
        packed_sort_key_u64(
            RenderPhase::PostProcess,
            sort_components,
            MeshPipelineVariantId::new(1).value(),
            3,
        )
    );
    assert_eq!(
        projected
            .gpu_scene_bind_group
            .as_ref()
            .map(MeshBindHandle::id),
        Some(99)
    );
    assert_eq!(projected.material.as_ref().map(MeshBindHandle::id), Some(3));
    match projected.instance_source {
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index,
            instance_count,
        } => {
            assert_eq!(first_instance_index, 27);
            assert_eq!(instance_count, 4);
        }
    }
    match projected.draw_args {
        MeshDrawArgs::DirectIndexed {
            first_instance,
            instance_count,
            ..
        } => {
            assert_eq!(first_instance, 27);
            assert_eq!(instance_count, 4);
        }
        MeshDrawArgs::IndexedIndirect { .. } => panic!("test command must be direct"),
    }
}

fn command(kind: MeshPassPipelineKind) -> MeshDrawCommand {
    MeshDrawCommand::new(
        RenderPhase::PostProcess,
        kind,
        default_pipeline_key(),
        MeshPipelineVariantId::new(1),
        0,
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index: 0,
            instance_count: 1,
        },
        MeshGeometryHandle::test(10),
        MeshDrawArgs::direct_indexed(0, 3),
    )
}
