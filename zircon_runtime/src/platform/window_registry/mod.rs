mod primary_window_role;
mod registry;
mod registry_id_allocator;
mod relationship;
mod slot;
mod window_registry_error;

pub(crate) use primary_window_role::{PrimaryWindowRoleChange, WindowCloseBegin};
pub(crate) use registry::WindowRegistry;
pub(super) use registry_id_allocator::allocate_window_registry_id;
pub(crate) use relationship::WindowParentKind;
pub(crate) use window_registry_error::WindowRegistryError;

#[cfg(test)]
mod tests;
