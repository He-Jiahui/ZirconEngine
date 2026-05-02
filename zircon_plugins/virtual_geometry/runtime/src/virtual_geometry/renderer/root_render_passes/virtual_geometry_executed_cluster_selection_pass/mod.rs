mod buffer;
mod execute;
mod output;
mod seed_backed_execution_selection;
mod selection_collection;
mod selection_filter;

pub(in crate::virtual_geometry::renderer::root_render_passes) use output::VirtualGeometryExecutedClusterSelectionPassOutput;

pub(super) use execute::execute_virtual_geometry_executed_cluster_selection_pass;

#[cfg(test)]
mod tests;
