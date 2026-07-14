use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use zircon_runtime_interface::reflect::{ReflectError, ReflectTypeRegistration, ReflectedValue};

use crate::core::framework::scene::ComponentTypeDescriptor;

use super::declared_value_type::DeclaredValueType;
use super::vm_type_backing::VmTypeBacking;

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
        validate_registration(&registration.registration)?;
        let type_path = registration.registration.type_path.type_path.clone();
        if self.registrations.contains_key(&type_path) {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }

        let short_type_path = registration.registration.type_path.short_type_path.as_str();
        self.update_short_path_lookup(&type_path, short_type_path);
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

    pub fn register_vm_type(
        &mut self,
        registration: ReflectTypeRegistration,
        backing: VmTypeBacking,
    ) -> Result<(), ReflectError> {
        let type_path = registration.type_path.type_path.clone();
        if self.registrations.contains_key(&type_path) {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }
        let descriptor = Self::vm_component_descriptor(&registration, backing)?;
        match backing {
            VmTypeBacking::DynamicComponent => {
                let component =
                    super::dynamic_component::reflect_component_for_dynamic_descriptor(&descriptor);
                self.register(RuntimeTypeRegistration {
                    registration,
                    component: Some(component),
                    resource: None,
                })
            }
        }
    }

    pub(crate) fn upsert_vm_type(
        &mut self,
        registration: ReflectTypeRegistration,
        backing: VmTypeBacking,
    ) -> Result<(), ReflectError> {
        let descriptor = Self::vm_component_descriptor(&registration, backing)?;
        let type_path = registration.type_path.type_path.clone();
        let replacement = match backing {
            VmTypeBacking::DynamicComponent => RuntimeTypeRegistration {
                component: Some(
                    super::dynamic_component::reflect_component_for_dynamic_descriptor(&descriptor),
                ),
                registration,
                resource: None,
            },
        };
        let Some(existing) = self.registrations.get(&type_path) else {
            return self.register(replacement);
        };
        if !existing.registration.plugin_owned
            || existing.registration.plugin_id != replacement.registration.plugin_id
        {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }

        validate_registration(&replacement.registration)?;
        self.registrations.insert(type_path, replacement);
        self.rebuild_short_path_lookup();
        Ok(())
    }

    pub(crate) fn remove_vm_type(&mut self, type_path: &str) -> Result<(), ReflectError> {
        let Some(existing) = self.registrations.get(type_path) else {
            return Ok(());
        };
        if !existing.registration.plugin_owned || existing.component.is_none() {
            return Err(ReflectError::InvalidRegistration {
                type_path: type_path.to_string(),
                reason:
                    "only plugin-owned VM component registrations may be removed by the VM catalog"
                        .to_string(),
            });
        }
        self.registrations.remove(type_path);
        self.rebuild_short_path_lookup();
        Ok(())
    }

    pub(crate) fn vm_component_descriptor(
        registration: &ReflectTypeRegistration,
        backing: VmTypeBacking,
    ) -> Result<ComponentTypeDescriptor, ReflectError> {
        let plugin_id = validate_vm_registration(registration, backing)?.to_string();
        let mut descriptor = ComponentTypeDescriptor::new(
            registration.type_path.type_path.clone(),
            plugin_id,
            registration.display_name.clone(),
        );
        for field in &registration.type_info.fields {
            descriptor = descriptor.with_property(
                field.name.clone(),
                field.value_type_path.clone(),
                field.editable,
            );
        }
        Ok(descriptor)
    }

    pub fn registration(&self, type_path: &str) -> Result<&ReflectTypeRegistration, ReflectError> {
        Ok(&self.runtime_registration(type_path)?.registration)
    }

    pub fn runtime_registration(
        &self,
        type_path: &str,
    ) -> Result<&RuntimeTypeRegistration, ReflectError> {
        if let Some(registration) = self.registrations.get(type_path) {
            return Ok(registration);
        }

        if let Some(resolved) = self.short_paths.get(type_path) {
            if let Some(registration) = self.registrations.get(resolved) {
                return Ok(registration);
            }
            return Err(ReflectError::InvalidRegistration {
                type_path: resolved.clone(),
                reason: format!("short type path `{type_path}` points at a missing registration"),
            });
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
        self.registrations.contains_key(type_path) || self.short_paths.contains_key(type_path)
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

    fn rebuild_short_path_lookup(&mut self) {
        self.short_paths.clear();
        self.ambiguous_short_paths.clear();
        let paths = self
            .registrations
            .iter()
            .map(|(type_path, registration)| {
                (
                    type_path.clone(),
                    registration.registration.type_path.short_type_path.clone(),
                )
            })
            .collect::<Vec<_>>();
        for (type_path, short_type_path) in paths {
            self.update_short_path_lookup(&type_path, &short_type_path);
        }
    }
}

fn validate_vm_registration(
    registration: &ReflectTypeRegistration,
    backing: VmTypeBacking,
) -> Result<&str, ReflectError> {
    let type_path = registration.type_path.type_path.as_str();
    validate_canonical_text(type_path, "full type path", type_path)?;
    validate_canonical_text(
        type_path,
        "short type path",
        &registration.type_path.short_type_path,
    )?;
    validate_canonical_text(type_path, "display name", &registration.display_name)?;
    if !registration.plugin_owned {
        return Err(invalid_vm_registration(
            type_path,
            "VM types must be plugin-owned",
        ));
    }
    let Some(plugin_id) = registration.plugin_id.as_deref() else {
        return Err(invalid_vm_registration(
            type_path,
            "VM types must declare a plugin id",
        ));
    };
    validate_canonical_text(type_path, "plugin id", plugin_id)?;
    if registration.type_path.plugin_id.as_deref() != Some(plugin_id) {
        return Err(invalid_vm_registration(
            type_path,
            "VM type path and registration plugin ids must match",
        ));
    }
    let plugin_prefix = format!("{plugin_id}.");
    if !type_path.starts_with(&plugin_prefix) {
        return Err(invalid_vm_registration(
            type_path,
            &format!("VM full type path must begin with `{plugin_prefix}`"),
        ));
    }
    for field in &registration.type_info.fields {
        if let Err(reason) = DeclaredValueType::parse(&field.value_type_path) {
            return Err(invalid_vm_registration(
                type_path,
                &format!("reflected field `{}` {reason}", field.name),
            ));
        }
    }

    match backing {
        VmTypeBacking::DynamicComponent
            if !registration.is_component || registration.is_resource =>
        {
            Err(invalid_vm_registration(
                type_path,
                "dynamic VM backing requires a component-only registration",
            ))
        }
        VmTypeBacking::DynamicComponent => Ok(plugin_id),
    }
}

fn validate_registration(registration: &ReflectTypeRegistration) -> Result<(), ReflectError> {
    let type_path = registration.type_path.type_path.as_str();
    if registration.is_component && registration.is_resource {
        return Err(invalid_vm_registration(
            type_path,
            "a reflected type cannot be both a component and a resource",
        ));
    }

    let mut field_names = BTreeSet::new();
    for field in &registration.type_info.fields {
        if field.name.trim().is_empty() || field.name.trim() != field.name {
            return Err(invalid_vm_registration(
                type_path,
                "reflected field names must be non-empty and already trimmed",
            ));
        }
        if !field_names.insert(field.name.as_str()) {
            return Err(invalid_vm_registration(
                type_path,
                &format!("duplicate reflected field `{}`", field.name),
            ));
        }
        if field.value_type_path.trim().is_empty()
            || field.value_type_path.trim() != field.value_type_path
        {
            return Err(invalid_vm_registration(
                type_path,
                &format!(
                    "reflected field `{}` value type path must be non-empty and already trimmed",
                    field.name
                ),
            ));
        }
        if let Some(default_value) = &field.default_value {
            ensure_reflected_value_type(
                type_path,
                &field.name,
                &field.value_type_path,
                default_value,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn ensure_reflected_value_type(
    type_path: &str,
    field_name: &str,
    expected: &str,
    value: &ReflectedValue,
) -> Result<(), ReflectError> {
    if reflected_value_matches_type_path(expected, value) {
        return Ok(());
    }
    Err(ReflectError::TypeMismatch {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
        expected: expected.to_string(),
        actual: value.type_name().to_string(),
    })
}

fn reflected_value_matches_type_path(expected: &str, value: &ReflectedValue) -> bool {
    DeclaredValueType::parse(expected)
        .map(|declared| declared.matches_reflected(value))
        .unwrap_or(false)
}

fn validate_canonical_text(type_path: &str, label: &str, value: &str) -> Result<(), ReflectError> {
    if value.is_empty() || value.trim() != value {
        return Err(invalid_vm_registration(
            type_path,
            &format!("VM {label} must be non-empty and already trimmed"),
        ));
    }
    Ok(())
}

fn invalid_vm_registration(type_path: &str, reason: &str) -> ReflectError {
    ReflectError::InvalidRegistration {
        type_path: type_path.to_string(),
        reason: reason.to_string(),
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
