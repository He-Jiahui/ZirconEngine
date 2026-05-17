use super::{CorePipelineKind, RenderPhase, RenderPhaseItem, RenderPhaseMeshSource};
use crate::core::framework::render::RenderMaterialAlphaMode;
use crate::core::framework::scene::EntityId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderPhaseQueue {
    pub items: Vec<RenderPhaseItem>,
}

impl RenderPhaseQueue {
    pub fn new(mut items: Vec<RenderPhaseItem>) -> Self {
        items.sort_by_key(|item| (item.phase_order(), item.sort_key, item.entity));
        Self { items }
    }

    pub fn items_for_phase(&self, phase: RenderPhase) -> impl Iterator<Item = &RenderPhaseItem> {
        self.items.iter().filter(move |item| item.phase == phase)
    }
}

impl RenderPhaseItem {
    fn phase_order(self) -> u8 {
        match self.phase {
            RenderPhase::Prepass => 0,
            RenderPhase::Shadow => 1,
            RenderPhase::Opaque2d | RenderPhase::Opaque3d => 2,
            RenderPhase::AlphaMask2d | RenderPhase::AlphaMask3d => 3,
            RenderPhase::Deferred => 4,
            RenderPhase::Transparent2d | RenderPhase::Transparent3d => 5,
            RenderPhase::PostProcess => 6,
            RenderPhase::Ui => 7,
            RenderPhase::Overlay => 8,
            RenderPhase::Debug => 9,
        }
    }
}

pub fn build_mesh_phase_queue<'a>(
    pipeline: CorePipelineKind,
    meshes: impl IntoIterator<Item = MeshPhaseInput<'a>>,
) -> RenderPhaseQueue {
    RenderPhaseQueue::new(
        meshes
            .into_iter()
            .map(|mesh| mesh.into_phase_item(pipeline))
            .collect(),
    )
}

pub fn build_sprite_phase_queue(
    pipeline: CorePipelineKind,
    sprites: impl IntoIterator<Item = SpritePhaseInput>,
) -> RenderPhaseQueue {
    RenderPhaseQueue::new(
        sprites
            .into_iter()
            .map(|sprite| sprite.into_phase_item(pipeline))
            .collect(),
    )
}

#[derive(Clone, Copy, Debug)]
pub struct MeshPhaseInput<'a> {
    pub entity: EntityId,
    pub mesh_index: usize,
    pub material_alpha_mode: &'a RenderMaterialAlphaMode,
    pub depth: f32,
}

impl MeshPhaseInput<'_> {
    fn into_phase_item(self, pipeline: CorePipelineKind) -> RenderPhaseItem {
        let (alpha_mask, transparent) = match self.material_alpha_mode {
            RenderMaterialAlphaMode::Opaque => (false, false),
            RenderMaterialAlphaMode::Mask { .. } => (true, false),
            RenderMaterialAlphaMode::Blend => (false, true),
        };
        let phase = RenderPhase::mesh_phase(pipeline, alpha_mask, transparent);
        RenderPhaseItem {
            entity: self.entity,
            phase,
            sort_key: super::RenderPhaseSortKey::for_mesh(phase, self.depth, self.entity),
            mesh_source: RenderPhaseMeshSource::MeshIndex(self.mesh_index),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpritePhaseInput {
    pub entity: EntityId,
    pub sprite_index: usize,
    pub material_alpha_mode: RenderMaterialAlphaMode,
    pub z_order: i32,
    pub depth: f32,
}

impl SpritePhaseInput {
    fn into_phase_item(self, pipeline: CorePipelineKind) -> RenderPhaseItem {
        let (alpha_mask, transparent) = match self.material_alpha_mode {
            RenderMaterialAlphaMode::Opaque => (false, false),
            RenderMaterialAlphaMode::Mask { .. } => (true, false),
            RenderMaterialAlphaMode::Blend => (false, true),
        };
        let phase = RenderPhase::mesh_phase(pipeline, alpha_mask, transparent);
        RenderPhaseItem {
            entity: self.entity,
            phase,
            sort_key: super::RenderPhaseSortKey::for_sprite(
                phase,
                self.z_order,
                self.depth,
                self.entity,
            ),
            mesh_source: RenderPhaseMeshSource::SpriteIndex(self.sprite_index),
        }
    }
}
