use zircon_runtime_interface::reflect::{ReflectScriptVisibility, ReflectTypeRegistration};

use crate::scene::{TypeRegistry, VmTypeBacking};
use crate::script::VmStateSchema;

use super::VmReflectionError;

/// Public VM-owned component registrations projected from one lifecycle state schema.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VmReflectionSchema {
    registrations: Vec<ReflectTypeRegistration>,
}

impl VmReflectionSchema {
    /// Projects the public component subset and validates it through the shared registry path.
    pub fn from_state_schema(schema: &VmStateSchema) -> Result<Self, VmReflectionError> {
        let mut registry = TypeRegistry::default();
        let mut registrations = Vec::new();
        for type_schema in &schema.types {
            let registration = &type_schema.registration;
            if registration.script_visibility != ReflectScriptVisibility::Public
                || !registration.is_component
                || registration.is_resource
            {
                continue;
            }
            registry.register_vm_type(registration.clone(), VmTypeBacking::DynamicComponent)?;
            registrations.push(registration.clone());
        }
        Ok(Self { registrations })
    }

    /// Returns public VM component registrations in the schema's stable declaration order.
    pub fn registrations(&self) -> &[ReflectTypeRegistration] {
        &self.registrations
    }

    /// Returns whether the VM generation exports no public reflected components.
    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }
}
