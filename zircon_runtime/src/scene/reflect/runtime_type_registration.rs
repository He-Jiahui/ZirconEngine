use std::fmt;

use zircon_runtime_interface::reflect::ReflectTypeRegistration;

use super::{ReflectComponent, ReflectResource};

#[derive(Clone)]
pub struct RuntimeTypeRegistration {
    pub registration: ReflectTypeRegistration,
    pub component: Option<ReflectComponent>,
    pub resource: Option<ReflectResource>,
}

impl RuntimeTypeRegistration {
    pub fn metadata(registration: ReflectTypeRegistration) -> Self {
        Self {
            registration,
            component: None,
            resource: None,
        }
    }
}

impl fmt::Debug for RuntimeTypeRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeTypeRegistration")
            .field("registration", &self.registration)
            .field("has_component_adapter", &self.component.is_some())
            .field("has_resource_adapter", &self.resource.is_some())
            .finish()
    }
}

impl PartialEq for RuntimeTypeRegistration {
    fn eq(&self, other: &Self) -> bool {
        self.registration == other.registration
            && self.component.is_some() == other.component.is_some()
            && self.resource.is_some() == other.resource.is_some()
    }
}
