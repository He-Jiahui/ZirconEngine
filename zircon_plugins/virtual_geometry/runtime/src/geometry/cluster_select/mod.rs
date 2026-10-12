#[path = "buffer.rs"]
mod buffer;
#[path = "execute.rs"]
mod execute;
#[path = "output.rs"]
mod output;
#[path = "../passes/seeds/mod.rs"]
mod seed_backed_execution_selection;
#[path = "../selection_collection.rs"]
mod selection_collection;
#[path = "../selection_filter.rs"]
mod selection_filter;

pub(in crate::virtual_geometry::renderer::root_render_passes) use output::VirtualGeometryExecutedClusterSelectionPassOutput;

pub(super) use execute::execute_virtual_geometry_executed_cluster_selection_pass;

#[cfg(test)]
#[path = "../passes/tests/mod.rs"]
mod tests;
