mod collect;
mod execution_owned_buffers;
mod execution_segments;
mod virtual_geometry_indirect_stats;

pub(in crate::virtual_geometry::renderer) use virtual_geometry_indirect_stats::VirtualGeometryIndirectStats;

pub(super) use collect::collect_virtual_geometry_indirect_stats;
