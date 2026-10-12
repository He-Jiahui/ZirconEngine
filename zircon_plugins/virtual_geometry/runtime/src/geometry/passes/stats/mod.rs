#[path = "collect.rs"]
mod collect;
#[path = "../../execution_owned_buffers.rs"]
mod execution_owned_buffers;
#[path = "../../stats/execution_segments.rs"]
mod execution_segments;
#[path = "stats.rs"]
mod virtual_geometry_indirect_stats;

pub(in crate::virtual_geometry::renderer) use virtual_geometry_indirect_stats::VirtualGeometryIndirectStats;

pub(super) use collect::collect_virtual_geometry_indirect_stats;
