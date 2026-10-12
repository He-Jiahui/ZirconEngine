//! View descriptors, instances, and registry for the editor workbench.

#[path = "activity_window_template_spec.rs"]
mod activity_window_template_spec;
#[path = "dock_policy.rs"]
mod dock_policy;
#[path = "pane_interaction_mode.rs"]
mod pane_interaction_mode;
#[path = "pane_payload_kind.rs"]
mod pane_payload_kind;
#[path = "pane_route_namespace.rs"]
mod pane_route_namespace;
#[path = "pane_template_spec.rs"]
mod pane_template_spec;
#[path = "view_descriptor.rs"]
mod view_descriptor;
#[path = "view_descriptor_builder.rs"]
mod view_descriptor_builder;
#[path = "view_descriptor_id.rs"]
mod view_descriptor_id;
#[path = "view_host.rs"]
mod view_host;
#[path = "view_instance.rs"]
mod view_instance;
#[path = "view_instance_id.rs"]
mod view_instance_id;
#[path = "view_kind.rs"]
mod view_kind;
#[path = "view_registry.rs"]
mod view_registry;
#[path = "view_registry_descriptor_access.rs"]
mod view_registry_descriptor_access;
#[path = "view_registry_descriptor_mutation.rs"]
mod view_registry_descriptor_mutation;
#[path = "view_registry_instance_access.rs"]
mod view_registry_instance_access;
#[path = "view_registry_instance_mutation.rs"]
mod view_registry_instance_mutation;
#[path = "view_registry_open_descriptor.rs"]
mod view_registry_open_descriptor;
#[path = "view_registry_register_view.rs"]
mod view_registry_register_view;
#[path = "view_registry_restore_instance.rs"]
mod view_registry_restore_instance;
#[path = "workbench_slot_to_view_host.rs"]
mod workbench_slot_to_view_host;

pub use crate::core::extension::WorkbenchSlot;
pub use activity_window_template_spec::ActivityWindowTemplateSpec;
pub use dock_policy::DockPolicy;
pub use pane_interaction_mode::PaneInteractionMode;
pub use pane_payload_kind::PanePayloadKind;
pub use pane_payload_kind::PanePayloadKind as ViewPayloadKind;
pub use pane_route_namespace::PaneRouteNamespace;
pub use pane_template_spec::{PaneBodySpec, PaneShellSpec, PaneTemplateSpec};
pub use view_descriptor::ViewDescriptor;
pub use view_descriptor_id::ViewDescriptorId;
pub use view_host::ViewHost;
pub use view_instance::ViewInstance;
pub use view_instance_id::ViewInstanceId;
pub use view_kind::ViewKind;
pub use view_registry::ViewRegistry;
