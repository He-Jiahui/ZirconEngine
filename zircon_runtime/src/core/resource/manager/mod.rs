mod events;
mod lazy_registration;
mod lease_ops;
mod management_projection;
mod payload_ops;
mod readiness_projection;
mod registry_export;
mod registry_ops;
mod resource_manager;
mod revision;
mod runtime_slot;

pub use resource_manager::{ResourceManager, ResourceRegistryReadGuard};
