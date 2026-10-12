//! 场景绘制的共享 GPU 顶点、统一参数与编辑器线条构建入口。
//! overlay 消费网格、选中和 gizmo 几何；场景、环境捕获与阴影 pass 共用 SceneUniform 布局。
#[path = "buffers/mod.rs"]
mod buffers;
#[path = "fallback/mod.rs"]
mod fallback;
#[path = "grid/mod.rs"]
mod grid;
#[path = "handles/mod.rs"]
mod handles;
#[path = "icon_vertex/mod.rs"]
mod icon_vertex;
#[path = "icons/mod.rs"]
mod icons;
#[path = "line_geometry/mod.rs"]
mod line_geometry;
#[path = "line_vertex/mod.rs"]
mod line_vertex;
#[path = "scene_environment_sh9.rs"]
mod scene_environment_sh9;
#[path = "scene_gizmo/mod.rs"]
mod scene_gizmo;
#[path = "scene_uniform/mod.rs"]
mod scene_uniform;
#[path = "selection/mod.rs"]
mod selection;
#[path = "wireframe/mod.rs"]
mod wireframe;

pub(crate) use buffers::{build_icon_buffer, build_line_buffer};
pub(crate) use fallback::{render_mat4_or, render_vec4_or};
pub(crate) use grid::build_grid_vertices;
pub(crate) use handles::build_handle_vertices;
pub(crate) use icon_vertex::IconVertex;
pub(crate) use icons::build_icon_quad_vertices;
pub(crate) use line_vertex::LineVertex;
pub(crate) use scene_environment_sh9::SceneEnvironmentSh9;
pub(crate) use scene_gizmo::build_scene_gizmo_line_vertices;
pub(crate) use scene_uniform::SceneUniform;
pub(crate) use selection::build_selection_vertices;
pub(crate) use wireframe::build_wireframe_vertices;
