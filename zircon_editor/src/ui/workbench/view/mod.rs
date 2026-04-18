//! View descriptors, instances, and registry for the editor workbench.

mod dock_policy;
mod preferred_host;
mod preferred_host_to_view_host;
mod view_descriptor;
mod view_descriptor_builder;
mod view_descriptor_id;
mod view_host;
mod view_instance;
mod view_instance_id;
mod view_kind;
mod view_registry;
mod view_registry_descriptor_access;
mod view_registry_instance_access;
mod view_registry_instance_mutation;
mod view_registry_open_descriptor;
mod view_registry_register_view;
mod view_registry_restore_instance;

pub use dock_policy::DockPolicy;
pub use preferred_host::PreferredHost;
pub use view_descriptor::ViewDescriptor;
pub use view_descriptor_id::ViewDescriptorId;
pub use view_host::ViewHost;
pub use view_instance::ViewInstance;
pub use view_instance_id::ViewInstanceId;
pub use view_kind::ViewKind;
pub use view_registry::ViewRegistry;
