mod cull_output_update;
mod indirect_output_update;
mod last_output_update;
mod render_path_output_update;

pub(in crate::graphics::scene::scene_renderer::core) use cull_output_update::VirtualGeometryCullOutputUpdate;
pub(in crate::graphics::scene::scene_renderer::core) use indirect_output_update::VirtualGeometryIndirectOutputUpdate;
pub(in crate::graphics::scene::scene_renderer::core) use last_output_update::VirtualGeometryLastOutputUpdate;
pub(in crate::graphics::scene::scene_renderer::core) use render_path_output_update::VirtualGeometryRenderPathOutputUpdate;
