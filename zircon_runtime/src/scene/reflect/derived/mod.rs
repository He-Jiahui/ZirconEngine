mod component_adapter;

pub use component_adapter::{
    derived_component_registration, derived_component_registration_with_adapter,
};
pub use zircon_runtime_interface::reflect::{ZrReflect, ZrReflectValue};

#[cfg(test)]
mod tests;
