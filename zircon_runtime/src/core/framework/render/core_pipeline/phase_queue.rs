use super::{
    CorePipelineKind, RenderPhase, RenderPhaseItem, RenderPhaseMeshSource,
    RenderPhaseSortComponents,
};
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
    pub depth_bias: f32,
    pub render_queue: i32,
    pub material_queue: i32,
    pub order_in_layer: i32,
    pub ui_z_index: i32,
}

impl<'a> MeshPhaseInput<'a> {
    pub const fn new(
        entity: EntityId,
        mesh_index: usize,
        material_alpha_mode: &'a RenderMaterialAlphaMode,
        depth: f32,
    ) -> Self {
        MeshPhaseInput {
            entity,
            mesh_index,
            material_alpha_mode,
            depth,
            depth_bias: 0.0,
            render_queue: 0,
            material_queue: 0,
            order_in_layer: 0,
            ui_z_index: 0,
        }
    }

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
        self
    }

    pub const fn with_render_queue(mut self, render_queue: i32) -> Self {
        self.render_queue = render_queue;
        self
    }

    pub const fn with_material_queue(mut self, material_queue: i32) -> Self {
        self.material_queue = material_queue;
        self
    }

    pub const fn with_order_in_layer(mut self, order_in_layer: i32) -> Self {
        self.order_in_layer = order_in_layer;
        self
    }

    pub const fn with_ui_z_index(mut self, ui_z_index: i32) -> Self {
        self.ui_z_index = ui_z_index;
        self
    }

    fn into_phase_item(self, pipeline: CorePipelineKind) -> RenderPhaseItem {
        let (alpha_mask, transparent) = match self.material_alpha_mode {
            RenderMaterialAlphaMode::Opaque => (false, false),
            RenderMaterialAlphaMode::Mask { .. } => (true, false),
            RenderMaterialAlphaMode::Blend => (false, true),
        };
        let phase = RenderPhase::mesh_phase(pipeline, alpha_mask, transparent);
        let sort_components = RenderPhaseSortComponents::new(self.depth, self.entity)
            .with_depth_bias(self.depth_bias)
            .with_render_queue(self.render_queue)
            .with_material_queue(self.material_queue)
            .with_order_in_layer(self.order_in_layer)
            .with_ui_z_index(self.ui_z_index);
        RenderPhaseItem {
            entity: self.entity,
            phase,
            sort_key: super::RenderPhaseSortKey::for_components(phase, sort_components),
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
    pub depth_bias: f32,
    pub render_queue: i32,
    pub material_queue: i32,
    pub ui_z_index: i32,
}

impl SpritePhaseInput {
    pub const fn new(
        entity: EntityId,
        sprite_index: usize,
        material_alpha_mode: RenderMaterialAlphaMode,
        z_order: i32,
        depth: f32,
    ) -> Self {
        Self {
            entity,
            sprite_index,
            material_alpha_mode,
            z_order,
            depth,
            depth_bias: 0.0,
            render_queue: 0,
            material_queue: 0,
            ui_z_index: 0,
        }
    }

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
        self
    }

    pub const fn with_render_queue(mut self, render_queue: i32) -> Self {
        self.render_queue = render_queue;
        self
    }

    pub const fn with_material_queue(mut self, material_queue: i32) -> Self {
        self.material_queue = material_queue;
        self
    }

    pub const fn with_ui_z_index(mut self, ui_z_index: i32) -> Self {
        self.ui_z_index = ui_z_index;
        self
    }

    fn into_phase_item(self, pipeline: CorePipelineKind) -> RenderPhaseItem {
        let (alpha_mask, transparent) = match self.material_alpha_mode {
            RenderMaterialAlphaMode::Opaque => (false, false),
            RenderMaterialAlphaMode::Mask { .. } => (true, false),
            RenderMaterialAlphaMode::Blend => (false, true),
        };
        let phase = RenderPhase::mesh_phase(pipeline, alpha_mask, transparent);
        let sort_components = RenderPhaseSortComponents::new(self.depth, self.entity)
            .with_depth_bias(self.depth_bias)
            .with_render_queue(self.render_queue)
            .with_material_queue(self.material_queue)
            .with_order_in_layer(self.z_order)
            .with_ui_z_index(self.ui_z_index);
        RenderPhaseItem {
            entity: self.entity,
            phase,
            sort_key: super::RenderPhaseSortKey::for_components(phase, sort_components),
            mesh_source: RenderPhaseMeshSource::SpriteIndex(self.sprite_index),
        }
    }
}
