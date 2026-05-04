mod virtual_geometry_cull;
mod virtual_geometry_dto_conversions;
mod virtual_geometry_neutral_readback_outputs;
mod virtual_geometry_output_buffers;
mod virtual_geometry_plugin_renderer_outputs;
mod virtual_geometry_readback_outputs;
mod virtual_geometry_snapshot_rebuild;

pub(crate) use virtual_geometry_plugin_renderer_outputs::runtime_prepare_renderer_outputs;
pub(in crate::virtual_geometry::renderer) use virtual_geometry_plugin_renderer_outputs::{
    plugin_renderer_outputs_from_indirect_stats,
    plugin_renderer_outputs_from_node_cluster_cull_readback,
};
