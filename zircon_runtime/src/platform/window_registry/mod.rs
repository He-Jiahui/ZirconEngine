#[path = "primary_window_role.rs"]
mod primary_window_role;
#[path = "registry.rs"]
mod registry;
#[path = "registry_id_allocator.rs"]
mod registry_id_allocator;
#[path = "relationship.rs"]
mod relationship;
#[path = "slot.rs"]
mod slot;
#[path = "window_registry_error.rs"]
mod window_registry_error;

pub(crate) use primary_window_role::{PrimaryWindowRoleChange, WindowCloseBegin};
pub(crate) use registry::WindowRegistry;
pub(super) use registry_id_allocator::allocate_window_registry_id;
pub(crate) use relationship::WindowParentKind;
pub(crate) use window_registry_error::WindowRegistryError;
