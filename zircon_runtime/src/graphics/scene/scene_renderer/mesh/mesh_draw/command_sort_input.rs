use crate::core::framework::render::RenderPhaseSortComponents;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeshCommandSortInput {
    pub(crate) depth: f32,
    pub(crate) depth_bias: f32,
    pub(crate) render_queue: i32,
    pub(crate) material_queue: i32,
    pub(crate) order_in_layer: i32,
    pub(crate) ui_z_index: i32,
    pub(crate) tie_breaker: u64,
}

impl MeshCommandSortInput {
    pub(crate) const fn new(depth: f32, tie_breaker: u64) -> Self {
        Self {
            depth,
            depth_bias: 0.0,
            render_queue: 0,
            material_queue: 0,
            order_in_layer: 0,
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
            .with_render_queue(self.render_queue)
            .with_material_queue(self.material_queue)
            .with_order_in_layer(self.order_in_layer)
            .with_ui_z_index(self.ui_z_index)
    }
}
