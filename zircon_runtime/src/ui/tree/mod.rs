//! 为接口层的数据树补充运行时算法，表面将布局、输入路由和命中索引连接为同一帧的几何消费者。

mod hit_test;
mod node;

pub(crate) use hit_test::{
    bounded_cells_for_frame, bounded_hit_grid_dimensions, find_bubble_route_value,
    frame_is_finite_positive, hit_grid_capacity_bounds,
};
pub use hit_test::{UiHitTestIndex, UiHitTestResult};
pub use node::{
    UiRuntimeTreeFocusExt, UiRuntimeTreeInteractionExt, UiRuntimeTreeLayoutExt,
    UiRuntimeTreeRenderOrderExt, UiRuntimeTreeRoutingExt, UiRuntimeTreeScrollExt,
};
