#[path = "despawned.rs"]
mod despawned;
#[path = "error.rs"]
mod error;
#[path = "internal.rs"]
mod internal;
#[path = "location.rs"]
mod location;
#[path = "registry.rs"]
mod registry;
#[path = "slot.rs"]
mod slot;
#[path = "stable_location.rs"]
mod stable_location;

pub use error::EntityRegistryError;
pub(crate) use internal::InternalEntity;
pub use location::EntityLocation;
pub(crate) use registry::EntityRegistry;
pub use stable_location::StableEntityLocation;
