#[path = "component_descriptor.rs"]
mod component_descriptor;
#[path = "component_model.rs"]
mod component_model;
#[path = "default_node_template.rs"]
mod default_node_template;
#[path = "fallback_policy.rs"]
mod fallback_policy;
#[path = "host_capability.rs"]
mod host_capability;
#[path = "option_descriptor.rs"]
mod option_descriptor;
#[path = "palette_metadata.rs"]
mod palette_metadata;
#[path = "prop_schema.rs"]
mod prop_schema;
#[path = "render_capability.rs"]
mod render_capability;
#[path = "slot_schema.rs"]
mod slot_schema;

pub use component_descriptor::UiComponentDescriptor;
pub use component_model::{UiComponentDescriptorKind, UiComponentLayoutRole};
pub use default_node_template::UiDefaultNodeTemplate;
pub use fallback_policy::{
    UiWidgetEditorFallback, UiWidgetFallbackPolicy, UiWidgetRuntimeFallback,
};
pub use host_capability::{UiHostCapability, UiHostCapabilitySet};
pub use option_descriptor::UiOptionDescriptor;
pub use palette_metadata::UiPaletteMetadata;
pub use prop_schema::UiPropSchema;
pub use render_capability::UiRenderCapability;
pub use slot_schema::UiSlotSchema;
