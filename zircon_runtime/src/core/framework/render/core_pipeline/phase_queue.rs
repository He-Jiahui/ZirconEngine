use super::{
    CorePipelineKind, RenderPhase, RenderPhaseItem, RenderPhaseMeshSource, RenderPhaseQueueSummary,
    RenderPhaseSortComponents, RenderQueueValue,
};
use crate::core::framework::scene::EntityId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 提取层交给绘制层的有序队列；阶段查询和诊断跨度都依赖构造时的顺序。
pub struct RenderPhaseQueue {
    // TODO: [CR-RENDER-CORE-0002] 确定是否封装可写 items；外部重排后阶段二分查询会漏项，当前调用方只读。
    pub items: Vec<RenderPhaseItem>,
}

impl RenderPhaseQueue {
    pub fn new(mut items: Vec<RenderPhaseItem>) -> Self {
        sort_phase_queue_items(&mut items);
        Self { items }
    }

    /// 仅查询已排序队列；共享优先级下的 2D/3D 阶段还须按精确阶段筛选。
    pub fn items_for_phase(&self, phase: RenderPhase) -> impl Iterator<Item = &RenderPhaseItem> {
        let phase_order = phase.queue_order();
        let start = self
            .items
            .partition_point(|item| item.phase.queue_order() < phase_order);
        let end = self
            .items
            .partition_point(|item| item.phase.queue_order() <= phase_order);
        self.items[start..end]
            .iter()
            .filter(move |item| item.phase == phase)
    }

    pub fn summary(&self) -> RenderPhaseQueueSummary {
        RenderPhaseQueueSummary::from_sorted_items(&self.items)
    }
}

// 间接排序保留相同键的提取顺序，同时避免比较时反复搬动完整队列项。
fn sort_phase_queue_items(items: &mut [RenderPhaseItem]) {
    if items.len() < 2 {
        return;
    }

    let mut ordered = Vec::with_capacity(items.len());
    ordered.extend(
        items
            .iter()
            .enumerate()
            .map(|(source_index, item)| (item.ordering_key(), source_index)),
    );
    ordered.sort_unstable();

    let mut destination_for_source = vec![0_usize; items.len()];
    for (destination, (_, source)) in ordered.into_iter().enumerate() {
        destination_for_source[source] = destination;
    }
    for source in 0..items.len() {
        while destination_for_source[source] != source {
            let destination = destination_for_source[source];
            items.swap(source, destination);
            destination_for_source.swap(source, destination);
        }
    }
}

pub fn build_mesh_phase_queue(
    pipeline: CorePipelineKind,
    meshes: impl IntoIterator<Item = MeshPhaseInput>,
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
pub struct MeshPhaseInput {
    pub entity: EntityId,
    pub mesh_index: usize,
    pub queue: RenderQueueValue,
    pub depth: f32,
    pub depth_bias: f32,
    pub camera_order: i32,
    pub sorting_layer: i32,
    pub order_in_layer: i32,
    pub y_sort: Option<f32>,
    pub ui_z_index: i32,
}

impl MeshPhaseInput {
    pub const fn new(
        entity: EntityId,
        mesh_index: usize,
        queue: RenderQueueValue,
        depth: f32,
    ) -> Self {
        MeshPhaseInput {
            entity,
            mesh_index,
            queue,
            depth,
            depth_bias: 0.0,
            camera_order: 0,
            sorting_layer: 0,
            order_in_layer: 0,
            y_sort: None,
            ui_z_index: 0,
        }
    }

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
        self
    }

    pub const fn with_camera_order(mut self, camera_order: i32) -> Self {
        self.camera_order = camera_order;
        self
    }

    pub const fn with_queue(mut self, queue: RenderQueueValue) -> Self {
        self.queue = queue;
        self
    }

    pub fn with_queue_offset(mut self, offset: i32) -> Self {
        self.queue = self.queue.with_material_offset_i32(offset);
        self
    }

    pub const fn with_sorting_layer(mut self, sorting_layer: i32) -> Self {
        self.sorting_layer = sorting_layer;
        self
    }

    pub const fn with_order_in_layer(mut self, order_in_layer: i32) -> Self {
        self.order_in_layer = order_in_layer;
        self
    }

    pub const fn with_y_sort(mut self, y_sort: Option<f32>) -> Self {
        self.y_sort = y_sort;
        self
    }

    pub const fn with_ui_z_index(mut self, ui_z_index: i32) -> Self {
        self.ui_z_index = ui_z_index;
        self
    }

    fn into_phase_item(self, pipeline: CorePipelineKind) -> RenderPhaseItem {
        let phase = self.queue.phase(pipeline);
        let sort_components = RenderPhaseSortComponents::new(self.depth, self.entity)
            .with_depth_bias(self.depth_bias)
            .with_camera_order(self.camera_order)
            .with_queue(self.queue)
            .with_sorting_layer(self.sorting_layer)
            .with_order_in_layer(self.order_in_layer)
            .with_y_sort(self.y_sort)
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
    pub queue: RenderQueueValue,
    pub z_order: i32,
    pub depth: f32,
    pub depth_bias: f32,
    pub camera_order: i32,
    pub sorting_layer: i32,
    pub y_sort: Option<f32>,
    pub ui_z_index: i32,
}

impl SpritePhaseInput {
    pub const fn new(
        entity: EntityId,
        sprite_index: usize,
        queue: RenderQueueValue,
        z_order: i32,
        depth: f32,
    ) -> Self {
        Self {
            entity,
            sprite_index,
            queue,
            z_order,
            depth,
            depth_bias: 0.0,
            camera_order: 0,
            sorting_layer: 0,
            y_sort: None,
            ui_z_index: 0,
        }
    }

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
        self
    }

    pub const fn with_camera_order(mut self, camera_order: i32) -> Self {
        self.camera_order = camera_order;
        self
    }

    pub const fn with_queue(mut self, queue: RenderQueueValue) -> Self {
        self.queue = queue;
        self
    }

    pub fn with_queue_offset(mut self, offset: i32) -> Self {
        self.queue = self.queue.with_material_offset_i32(offset);
        self
    }

    pub const fn with_sorting_layer(mut self, sorting_layer: i32) -> Self {
        self.sorting_layer = sorting_layer;
        self
    }

    pub const fn with_y_sort(mut self, y_sort: Option<f32>) -> Self {
        self.y_sort = y_sort;
        self
    }

    pub const fn with_ui_z_index(mut self, ui_z_index: i32) -> Self {
        self.ui_z_index = ui_z_index;
        self
    }

    fn into_phase_item(self, pipeline: CorePipelineKind) -> RenderPhaseItem {
        let phase = self.queue.phase(pipeline);
        let sort_components = RenderPhaseSortComponents::new(self.depth, self.entity)
            .with_depth_bias(self.depth_bias)
            .with_camera_order(self.camera_order)
            .with_queue(self.queue)
            .with_sorting_layer(self.sorting_layer)
            .with_order_in_layer(self.z_order)
            .with_y_sort(self.y_sort)
            .with_ui_z_index(self.ui_z_index);
        RenderPhaseItem {
            entity: self.entity,
            phase,
            sort_key: super::RenderPhaseSortKey::for_components(phase, sort_components),
            mesh_source: RenderPhaseMeshSource::SpriteIndex(self.sprite_index),
        }
    }
}

#[cfg(test)]
#[path = "tests/phase_queue.rs"]
mod tests;
