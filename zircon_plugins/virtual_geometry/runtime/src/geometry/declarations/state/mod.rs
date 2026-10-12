#[path = "../../state/budget.rs"]
mod budget;
#[path = "../../state/hot_frontier.rs"]
mod hot_frontier;
#[path = "../../state/page_metadata.rs"]
mod page_metadata;
#[path = "../../state/request_state.rs"]
mod request_state;
#[path = "../../state/residency.rs"]
mod residency;
#[path = "../../state/runtime_state.rs"]
mod runtime_state;
#[path = "../../state/slot_allocator.rs"]
mod slot_allocator;

pub(in crate::virtual_geometry) use hot_frontier::HOT_FRONTIER_COOLING_FRAME_COUNT;
pub(crate) use runtime_state::VirtualGeometryRuntimeState;
