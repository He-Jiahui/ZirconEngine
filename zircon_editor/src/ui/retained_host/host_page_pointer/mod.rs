mod base_state;
mod build_host_page_pointer_layout;
mod constants;
mod dispatch_event;
mod error;
mod handle_click;
mod handle_overflow_click;
mod host_page_pointer_bridge;
mod host_page_pointer_dispatch;
mod host_page_pointer_item;
mod host_page_pointer_layout;
mod host_page_pointer_route;
mod new;
mod rebuild_surface;
mod register_handled_pointer_node;
mod root_frame;
mod sync;
mod tab_node_id;
mod tab_strip_geometry;

pub(crate) use build_host_page_pointer_layout::build_host_page_pointer_layout;
pub(crate) use constants::HOST_PAGE_OVERFLOW_POINTER_INDEX;
pub(crate) use error::HostPagePointerError;
pub(crate) use host_page_pointer_bridge::HostPagePointerBridge;
pub(crate) use host_page_pointer_dispatch::HostPagePointerDispatch;
pub(crate) use host_page_pointer_route::HostPagePointerRoute;

#[cfg(test)]
pub(crate) use host_page_pointer_item::HostPagePointerItem;
#[cfg(test)]
pub(crate) use host_page_pointer_layout::HostPagePointerLayout;
#[cfg(test)]
pub(crate) use tab_strip_geometry::{HostPageOverflowSlot, HostPageTabSlot};
