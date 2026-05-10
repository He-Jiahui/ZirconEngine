use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use zircon_runtime_interface::reflect::{ReflectError, ReflectTypeRegistration};

#[derive(Clone)]
pub struct RuntimeTypeRegistration {
    pub registration: ReflectTypeRegistration,
    pub component: Option<crate::scene::reflect::ReflectComponent>,
    pub resource: Option<crate::scene::reflect::ReflectResource>,
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

#[derive(Clone, Default)]
pub struct TypeRegistry {
    registrations: BTreeMap<String, RuntimeTypeRegistration>,
    short_paths: BTreeMap<String, String>,
    ambiguous_short_paths: BTreeSet<String>,
}

impl TypeRegistry {
    pub fn register(&mut self, registration: RuntimeTypeRegistration) -> Result<(), ReflectError> {
        let type_path = registration.registration.type_path.type_path.clone();
        if self.registrations.contains_key(&type_path) {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }

        let short_type_path = registration.registration.type_path.short_type_path.clone();
        self.update_short_path_lookup(&type_path, &short_type_path);
        self.registrations.insert(type_path, registration);
        Ok(())
    }

    pub fn register_resource(
        &mut self,
        registration: ReflectTypeRegistration,
        adapter: crate::scene::reflect::ReflectResource,
    ) -> Result<(), ReflectError> {
        if !registration.is_resource || registration.is_component {
            return Err(ReflectError::InvalidRegistration {
                type_path: registration.type_path.type_path.clone(),
                reason: "resource adapters require resource-only registrations".to_string(),
            });
        }

        self.register(RuntimeTypeRegistration {
            registration,
            component: None,
            resource: Some(adapter),
        })
    }

    pub fn registration(&self, type_path: &str) -> Result<&ReflectTypeRegistration, ReflectError> {
        self.runtime_registration(type_path)
            .map(|registration| &registration.registration)
    }

    pub fn runtime_registration(
        &self,
        type_path: &str,
    ) -> Result<&RuntimeTypeRegistration, ReflectError> {
        let resolved = self.resolve(type_path)?;
        Ok(self
            .registrations
            .get(resolved)
            .expect("resolved reflected type path must exist in registry"))
    }

    pub fn resolve(&self, type_path: &str) -> Result<&str, ReflectError> {
        if let Some((canonical_type_path, _)) = self.registrations.get_key_value(type_path) {
            return Ok(canonical_type_path.as_str());
        }

        if let Some(resolved) = self.short_paths.get(type_path) {
            return Ok(resolved.as_str());
        }

        if self.ambiguous_short_paths.contains(type_path) {
            return Err(ReflectError::AmbiguousShortTypePath {
                short_type_path: type_path.to_string(),
            });
        }

        Err(ReflectError::UnknownType {
            type_path: type_path.to_string(),
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &RuntimeTypeRegistration> {
        self.registrations.values()
    }

    pub fn contains(&self, type_path: &str) -> bool {
        self.resolve(type_path).is_ok()
    }

    pub fn contains_type_path(&self, type_path: &str) -> bool {
        self.registrations.contains_key(type_path)
    }

    pub fn clear(&mut self) {
        self.registrations.clear();
        self.short_paths.clear();
        self.ambiguous_short_paths.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    fn update_short_path_lookup(&mut self, type_path: &str, short_type_path: &str) {
        if self.ambiguous_short_paths.contains(short_type_path) {
            return;
        }

        match self.short_paths.get(short_type_path) {
            None => {
                self.short_paths
                    .insert(short_type_path.to_string(), type_path.to_string());
            }
            Some(existing) if existing == type_path => {}
            Some(_) => {
                self.short_paths.remove(short_type_path);
                self.ambiguous_short_paths
                    .insert(short_type_path.to_string());
            }
        }
    }
}

impl fmt::Debug for TypeRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TypeRegistry")
            .field("registrations", &self.registrations)
            .field("short_paths", &self.short_paths)
            .field("ambiguous_short_paths", &self.ambiguous_short_paths)
            .finish()
    }
}

impl PartialEq for TypeRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.registrations == other.registrations
            && self.short_paths == other.short_paths
            && self.ambiguous_short_paths == other.ambiguous_short_paths
    }
}
