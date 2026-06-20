use crate::core::framework::render::{RenderPhaseSortComponents, RenderQueueValue};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeshCommandSortInput {
    pub(crate) depth: f32,
    pub(crate) depth_bias: f32,
    pub(crate) queue: RenderQueueValue,
    pub(crate) camera_order: i32,
    pub(crate) sorting_layer: i32,
    pub(crate) order_in_layer: i32,
    pub(crate) y_sort: Option<f32>,
    pub(crate) ui_z_index: i32,
    pub(crate) tie_breaker: u64,
}

impl MeshCommandSortInput {
    pub(crate) const fn new(depth: f32, tie_breaker: u64) -> Self {
        Self {
            depth,
            depth_bias: 0.0,
            queue: RenderQueueValue::GEOMETRY,
            camera_order: 0,
            sorting_layer: 0,
            order_in_layer: 0,
            y_sort: None,
            ui_z_index: 0,
            tie_breaker,
        }
    }

    pub(crate) const fn with_tie_breaker(mut self, tie_breaker: u64) -> Self {
        self.tie_breaker = tie_breaker;
        self
    }

    pub(crate) fn components(self) -> RenderPhaseSortComponents {
        RenderPhaseSortComponents::new(self.depth, self.tie_breaker)
            .with_depth_bias(self.depth_bias)
            .with_camera_order(self.camera_order)
            .with_queue(self.queue)
            .with_sorting_layer(self.sorting_layer)
            .with_order_in_layer(self.order_in_layer)
            .with_y_sort(self.y_sort)
            .with_ui_z_index(self.ui_z_index)
    }
}
