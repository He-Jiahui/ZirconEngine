#[path = "../../../scene/component_type_desc/component_property_desc.rs"]
mod component_property_descriptor;
#[path = "../component_type_desc/desc.rs"]
mod component_type_descriptor;
#[path = "../component_type_desc/constructors.rs"]
mod constructors;

pub use component_property_descriptor::ComponentPropertyDescriptor;
pub use component_type_descriptor::ComponentTypeDescriptor;
