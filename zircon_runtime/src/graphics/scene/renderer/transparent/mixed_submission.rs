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

/// 合并调用方已筛出的 Transparent3d mesh 命令与本函数筛出的 SpriteIndex 项，再按相同排序键交错提交。
/// 返回索引仍指向原命令或精灵列表；同键时用实体、来源和原索引确定顺序。
pub(crate) fn build_transparent_submission_order(
    mesh_commands: &[MeshDrawCommand],
    sprite_phase_queue: &RenderPhaseQueue,
) -> Vec<TransparentSubmissionItem> {
    let sprite_items = sprite_phase_queue
        .items_for_phase(RenderPhase::Transparent3d)
        .filter_map(|phase_item| match phase_item.mesh_source {
            RenderPhaseMeshSource::SpriteIndex(sprite_index) => Some(TransparentSubmissionItem {
                source: TransparentSubmissionSource::Sprite { sprite_index },
                sort_key: phase_item.sort_key.raw(),
                entity: phase_item.entity,
            }),
            RenderPhaseMeshSource::MeshIndex(_) => None,
        });
    let sprite_capacity = sprite_items.size_hint().1.unwrap_or(0);
    let mut items = Vec::with_capacity(mesh_commands.len().saturating_add(sprite_capacity));
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
    items.extend(sprite_items);
    items.sort_unstable_by_key(|item| item.ordering_key());
    items
}

#[cfg(test)]
#[path = "tests/mixed_submission.rs"]
mod tests;
