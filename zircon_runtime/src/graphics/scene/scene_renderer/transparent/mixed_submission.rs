use crate::core::framework::render::{RenderPhase, RenderPhaseMeshSource, RenderPhaseQueue};
use crate::core::framework::scene::EntityId;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawCommand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TransparentSubmissionSource {
    Mesh { command_index: usize },
    Sprite { sprite_index: usize },
}

impl TransparentSubmissionSource {
    const fn order(self) -> u8 {
        match self {
            Self::Mesh { .. } => 0,
            Self::Sprite { .. } => 1,
        }
    }

    const fn stable_index(self) -> usize {
        match self {
            Self::Mesh { command_index } => command_index,
            Self::Sprite { sprite_index } => sprite_index,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TransparentSubmissionItem {
    pub(crate) source: TransparentSubmissionSource,
    pub(crate) sort_key: u64,
    pub(crate) entity: EntityId,
}

impl TransparentSubmissionItem {
    const fn ordering_key(self) -> (u64, EntityId, u8, usize) {
        (
            self.sort_key,
            self.entity,
            self.source.order(),
            self.source.stable_index(),
        )
    }
}

pub(crate) fn has_transparent_sprite_submissions(sprite_phase_queue: &RenderPhaseQueue) -> bool {
    sprite_phase_queue
        .items_for_phase(RenderPhase::Transparent3d)
        .any(|item| matches!(item.mesh_source, RenderPhaseMeshSource::SpriteIndex(_)))
}

pub(crate) fn build_transparent_submission_order(
    mesh_commands: &[MeshDrawCommand],
    sprite_phase_queue: &RenderPhaseQueue,
) -> Vec<TransparentSubmissionItem> {
    let mut items = Vec::new();
    items.extend(
        mesh_commands
            .iter()
            .enumerate()
            .map(|(command_index, command)| TransparentSubmissionItem {
                source: TransparentSubmissionSource::Mesh { command_index },
                sort_key: command.sort_key,
                entity: command.source_entity,
            }),
    );
    items.extend(
        sprite_phase_queue
            .items_for_phase(RenderPhase::Transparent3d)
            .filter_map(|phase_item| match phase_item.mesh_source {
                RenderPhaseMeshSource::SpriteIndex(sprite_index) => {
                    Some(TransparentSubmissionItem {
                        source: TransparentSubmissionSource::Sprite { sprite_index },
                        sort_key: phase_item.sort_key.raw(),
                        entity: phase_item.entity,
                    })
                }
                RenderPhaseMeshSource::MeshIndex(_) => None,
            }),
    );
    items.sort_by_key(|item| item.ordering_key());
    items
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderPhase, RenderPhaseItem, RenderPhaseMeshSource, RenderPhaseQueue, RenderPhaseSortKey,
    };
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        DrawInstanceSource, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle,
        MeshPassPipelineKind, MeshPipelineVariantId,
    };

    use super::{
        build_transparent_submission_order, has_transparent_sprite_submissions,
        TransparentSubmissionSource,
    };

    #[test]
    fn transparent_submission_order_interleaves_meshes_and_sprites_by_sort_key() {
        let mesh_commands = vec![
            mesh_command(100, 100, 0),
            mesh_command(300, 300, 1),
            mesh_command(500, 500, 2),
        ];
        let sprite_phase_queue = RenderPhaseQueue::new(vec![
            sprite_phase_item(200, 200, 20, RenderPhase::Transparent3d),
            sprite_phase_item(400, 400, 40, RenderPhase::Transparent3d),
        ]);

        let order = build_transparent_submission_order(&mesh_commands, &sprite_phase_queue)
            .into_iter()
            .map(|item| item.source)
            .collect::<Vec<_>>();

        assert_eq!(
            order,
            vec![
                TransparentSubmissionSource::Mesh { command_index: 0 },
                TransparentSubmissionSource::Sprite { sprite_index: 20 },
                TransparentSubmissionSource::Mesh { command_index: 1 },
                TransparentSubmissionSource::Sprite { sprite_index: 40 },
                TransparentSubmissionSource::Mesh { command_index: 2 },
            ]
        );
    }

    #[test]
    fn transparent_submission_order_ignores_non_transparent3d_sprites() {
        let mesh_commands = vec![mesh_command(100, 100, 0)];
        let sprite_phase_queue = RenderPhaseQueue::new(vec![
            sprite_phase_item(50, 50, 5, RenderPhase::Transparent2d),
            sprite_phase_item(150, 150, 15, RenderPhase::Transparent3d),
        ]);

        assert!(has_transparent_sprite_submissions(&sprite_phase_queue));
        let order = build_transparent_submission_order(&mesh_commands, &sprite_phase_queue)
            .into_iter()
            .map(|item| item.source)
            .collect::<Vec<_>>();

        assert_eq!(
            order,
            vec![
                TransparentSubmissionSource::Mesh { command_index: 0 },
                TransparentSubmissionSource::Sprite { sprite_index: 15 },
            ]
        );
    }

    #[test]
    fn transparent_sprite_submission_detection_ignores_mesh_phase_items() {
        let sprite_phase_queue = RenderPhaseQueue::new(vec![RenderPhaseItem {
            entity: 7,
            phase: RenderPhase::Transparent3d,
            sort_key: RenderPhaseSortKey::new(7),
            mesh_source: RenderPhaseMeshSource::MeshIndex(3),
        }]);

        assert!(!has_transparent_sprite_submissions(&sprite_phase_queue));
    }

    fn mesh_command(sort_key: u64, entity: u64, source_draw_index: usize) -> MeshDrawCommand {
        MeshDrawCommand::new(
            RenderPhase::Transparent3d,
            MeshPassPipelineKind::Base,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
            sort_key,
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: 0,
                instance_count: 1,
            },
            MeshGeometryHandle::test(1),
            MeshDrawArgs::direct_indexed(0, 3),
        )
        .with_source_entity(entity)
        .with_source_draw_index(source_draw_index)
    }

    fn sprite_phase_item(
        sort_key: u64,
        entity: u64,
        sprite_index: usize,
        phase: RenderPhase,
    ) -> RenderPhaseItem {
        RenderPhaseItem {
            entity,
            phase,
            sort_key: RenderPhaseSortKey::new(sort_key),
            mesh_source: RenderPhaseMeshSource::SpriteIndex(sprite_index),
        }
    }
}
