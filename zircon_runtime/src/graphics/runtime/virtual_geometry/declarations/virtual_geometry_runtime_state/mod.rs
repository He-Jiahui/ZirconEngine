mod budget;
mod hot_frontier;
mod page_metadata;
mod request_state;
mod residency;
mod runtime_state;
mod slot_allocator;

pub(in crate::graphics::runtime::virtual_geometry) use hot_frontier::HOT_FRONTIER_COOLING_FRAME_COUNT;
pub(crate) use runtime_state::VirtualGeometryRuntimeState;
