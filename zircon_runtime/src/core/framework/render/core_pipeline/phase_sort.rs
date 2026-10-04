use serde::{Deserialize, Serialize};

use super::packed_sort_key::{depth_sort_key, ordered_depth_key, packed_sort_key_u64};
use super::{RenderPhase, RenderQueueValue};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
/// 队列比较用的压缩键；业务端应由阶段及排序分量构造，不得假定内部位段稳定。
pub struct RenderPhaseSortKey(u64);

#[derive(Clone, Copy, Debug, PartialEq)]
/// 提取侧保留的排序意图；绘制侧按阶段压缩，诊断侧据此解释各排序域。
pub struct RenderPhaseSortComponents {
    pub camera_order: i32,
    pub queue: RenderQueueValue,
    pub sorting_layer: i32,
    pub order_in_layer: i32,
    pub y_sort: Option<f32>,
    pub depth: f32,
    pub depth_bias: f32,
    pub ui_z_index: i32,
    pub entity_tie_breaker: u64,
}

impl RenderPhaseSortComponents {
    pub const fn new(depth: f32, entity_tie_breaker: u64) -> Self {
        Self {
            camera_order: 0,
            queue: RenderQueueValue::GEOMETRY,
            sorting_layer: 0,
            order_in_layer: 0,
            y_sort: None,
            depth,
            depth_bias: 0.0,
            ui_z_index: 0,
            entity_tie_breaker,
        }
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

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
        self
    }

    pub const fn with_ui_z_index(mut self, ui_z_index: i32) -> Self {
        self.ui_z_index = ui_z_index;
        self
    }

    pub fn effective_depth(self) -> f32 {
        self.depth + self.depth_bias
    }

    pub fn depth_key(self) -> i64 {
        depth_sort_key(self.effective_depth())
    }

    pub fn ordered_depth_key(self, phase: RenderPhase) -> i64 {
        ordered_depth_key(phase, self.depth_key())
    }
}

impl RenderPhaseSortKey {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn for_components(phase: RenderPhase, components: RenderPhaseSortComponents) -> Self {
        Self(packed_sort_key_u64(phase, components, 0, 0))
    }

    pub fn for_mesh(phase: RenderPhase, depth: f32, tie_breaker: u64) -> Self {
        Self::for_components(
            phase,
            RenderPhaseSortComponents::new(depth, tie_breaker)
                .with_queue(default_queue_for_phase(phase)),
        )
    }

    pub fn for_sprite(phase: RenderPhase, z_order: i32, depth: f32, tie_breaker: u64) -> Self {
        Self::for_components(
            phase,
            RenderPhaseSortComponents::new(depth, tie_breaker)
                .with_queue(default_queue_for_phase(phase))
                .with_order_in_layer(z_order),
        )
    }
}

fn default_queue_for_phase(phase: RenderPhase) -> RenderQueueValue {
    match phase {
        RenderPhase::AlphaMask2d | RenderPhase::AlphaMask3d => RenderQueueValue::ALPHA_TEST,
        RenderPhase::Transparent2d | RenderPhase::Transparent3d => RenderQueueValue::TRANSPARENT,
        RenderPhase::Ui | RenderPhase::Overlay | RenderPhase::Debug => RenderQueueValue::OVERLAY,
        _ => RenderQueueValue::GEOMETRY,
    }
}

#[cfg(test)]
#[path = "tests/phase_sort.rs"]
mod tests;
