mod collect;
mod execution_owned_buffers;
mod execution_segments;
mod virtual_geometry_indirect_stats;

pub(in crate::graphics::scene::scene_renderer::core) use virtual_geometry_indirect_stats::VirtualGeometryIndirectStats;

pub(super) use collect::collect_virtual_geometry_indirect_stats;
