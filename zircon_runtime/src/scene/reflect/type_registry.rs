use std::collections::{BTreeMap, HashSet};
use std::fmt;

use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldId, ReflectFieldInfo, ReflectSchemaCatalog,
    ReflectSchemaCatalogEntry, ReflectTypeRegistration, ReflectedValue,
};

use crate::core::framework::scene::ComponentTypeDescriptor;

use super::declared_value_type::DeclaredValueType;
use super::runtime_type_registration::RuntimeTypeRegistration;
use super::value_admission::validate_reflected_value_contract;
use super::vm_type_backing::VmTypeBacking;

const MAX_REFLECT_FIELDS_PER_TYPE: usize = 4_096;
const MAX_REFLECT_FIELD_DISPLAY_NAME_BYTES: usize = 256;
const MAX_REFLECT_ENUM_OPTIONS_PER_FIELD: usize = 4_096;
const MAX_REFLECT_ENUM_OPTIONS_PER_TYPE: usize = 16_384;
const MAX_REFLECT_ENUM_VALUE_BYTES: usize = 128;
const MAX_REFLECT_ENUM_DISPLAY_NAME_BYTES: usize = 256;

#[derive(Clone, Default)]
pub struct TypeRegistry {
    registrations: BTreeMap<String, RuntimeTypeRegistration>,
    schema_catalog: ReflectSchemaCatalog,
    schema_catalog_generation: u64,
}

impl TypeRegistry {
    pub fn register(&mut self, registration: RuntimeTypeRegistration) -> Result<(), ReflectError> {
        self.validate_new_registration(&registration)?;
        self.publish_prevalidated(registration);
        Ok(())
    }

    pub(crate) fn validate_new_registration(
        &self,
        registration: &RuntimeTypeRegistration,
    ) -> Result<(), ReflectError> {
        validate_registration(&registration.registration)?;
        self.schema_catalog
            .validate_insert(&ReflectSchemaCatalogEntry::new(
                registration.registration.clone(),
            ))
    }

    pub(crate) fn publish_prevalidated(&mut self, mut registration: RuntimeTypeRegistration) {
        debug_assert!(self.validate_new_registration(&registration).is_ok());
        let type_path = registration.registration.type_path.type_path().to_string();
        self.schema_catalog
            .try_insert(ReflectSchemaCatalogEntry::new(
                registration.registration.clone(),
            ))
            .expect("prevalidated reflection schema entry must publish");
        registration.registration = self
            .schema_catalog
            .registration(&type_path)
            .expect("published schema entry must resolve")
            .clone();
        let previous = self.registrations.insert(type_path, registration);
        debug_assert!(previous.is_none());
        self.advance_schema_catalog_generation();
    }

    pub fn register_resource(
        &mut self,
        registration: ReflectTypeRegistration,
        adapter: crate::scene::reflect::ReflectResource,
    ) -> Result<(), ReflectError> {
        if !registration.is_resource() {
            return Err(ReflectError::InvalidRegistration {
                type_path: registration.type_path.type_path().to_string(),
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
        self.register_vm_type_with_descriptor(registration, backing)?;
        Ok(())
    }

    pub(crate) fn register_vm_type_with_descriptor(
        &mut self,
        registration: ReflectTypeRegistration,
        backing: VmTypeBacking,
    ) -> Result<ComponentTypeDescriptor, ReflectError> {
        let type_path = registration.type_path.type_path().to_string();
        if self.registrations.contains_key(&type_path) {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }
        let descriptor = Self::vm_component_descriptor(&registration, backing)?;
        match backing {
            VmTypeBacking::DynamicComponent => {
                let component =
                    super::dynamic_component::reflect_component_for_dynamic_descriptor(&descriptor);
                let runtime_registration = RuntimeTypeRegistration {
                    registration,
                    component: Some(component),
                    resource: None,
                };
                self.validate_new_registration(&runtime_registration)?;
                self.publish_prevalidated(runtime_registration);
            }
        }
        Ok(descriptor)
    }

    pub(crate) fn upsert_vm_type(
        &mut self,
        registration: ReflectTypeRegistration,
        backing: VmTypeBacking,
    ) -> Result<ComponentTypeDescriptor, ReflectError> {
        let descriptor = Self::vm_component_descriptor(&registration, backing)?;
        let type_path = registration.type_path.type_path().to_string();
        let mut replacement = match backing {
            VmTypeBacking::DynamicComponent => RuntimeTypeRegistration {
                component: Some(
                    super::dynamic_component::reflect_component_for_dynamic_descriptor(&descriptor),
                ),
                registration,
                resource: None,
            },
        };
        let Some(existing) = self.registrations.get(&type_path) else {
            self.validate_new_registration(&replacement)?;
            self.publish_prevalidated(replacement);
            return Ok(descriptor);
        };
        if !existing.registration.is_plugin_owned()
            || existing.registration.type_path.plugin_id()
                != replacement.registration.type_path.plugin_id()
        {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }
        if existing == &replacement {
            return Ok(descriptor);
        }

        self.schema_catalog
            .try_replace(ReflectSchemaCatalogEntry::new(
                replacement.registration.clone(),
            ))?;
        replacement.registration = self.schema_catalog.registration(&type_path)?.clone();
        self.registrations.insert(type_path.clone(), replacement);
        self.advance_schema_catalog_generation();
        Ok(descriptor)
    }

    pub(crate) fn remove_vm_type(&mut self, type_path: &str) -> Result<(), ReflectError> {
        let Some(existing) = self.registrations.get(type_path) else {
            return Ok(());
        };
        if !existing.registration.is_plugin_owned() || existing.component.is_none() {
            return Err(ReflectError::InvalidRegistration {
                type_path: type_path.to_string(),
                reason:
                    "only plugin-owned VM component registrations may be removed by the VM catalog"
                        .to_string(),
            });
        }
        self.schema_catalog.try_remove(type_path)?;
        self.registrations.remove(type_path);
        self.advance_schema_catalog_generation();
        Ok(())
    }

    pub(crate) fn vm_component_descriptor(
        registration: &ReflectTypeRegistration,
        backing: VmTypeBacking,
    ) -> Result<ComponentTypeDescriptor, ReflectError> {
        let plugin_id = validate_vm_registration(registration, backing)?.to_string();
        let mut descriptor = ComponentTypeDescriptor::new(
            registration.type_path.type_path().to_string(),
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
        self.schema_catalog.registration(type_path)
    }

    pub fn runtime_registration(
        &self,
        type_path: &str,
    ) -> Result<&RuntimeTypeRegistration, ReflectError> {
        if let Some(registration) = self.registrations.get(type_path) {
            return Ok(registration);
        }

        let resolved = self.schema_catalog.resolve_type_path(type_path)?;
        self.registrations
            .get(resolved)
            .ok_or_else(|| ReflectError::InvalidRegistration {
                type_path: resolved.to_string(),
                reason: "schema catalog entry is missing its runtime adapter projection"
                    .to_string(),
            })
    }

    pub fn resolve(&self, type_path: &str) -> Result<&str, ReflectError> {
        self.schema_catalog.resolve_type_path(type_path)
    }

    pub(crate) fn resolve_field_slot_by_id(
        &self,
        type_path: &str,
        field_id: ReflectFieldId,
    ) -> Result<u32, ReflectError> {
        self.schema_catalog.field_slot_by_id(type_path, field_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &RuntimeTypeRegistration> {
        self.registrations.values()
    }

    pub fn contains(&self, type_path: &str) -> bool {
        self.schema_catalog.contains(type_path)
    }

    pub fn contains_type_path(&self, type_path: &str) -> bool {
        self.schema_catalog.contains_type_path(type_path)
    }

    pub fn schema_catalog(&self) -> &ReflectSchemaCatalog {
        &self.schema_catalog
    }

    /// Returns the revision of the registered reflection catalog.
    ///
    /// Dynamic-scene spawn plans bind this value so a reflected registration
    /// change cannot reuse a plan compiled against an older schema catalog.
    pub fn schema_catalog_generation(&self) -> u64 {
        self.schema_catalog_generation
    }

    pub fn clear(&mut self) {
        let had_registrations = !self.registrations.is_empty();
        self.registrations.clear();
        self.schema_catalog.clear();
        if had_registrations {
            self.advance_schema_catalog_generation();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    fn advance_schema_catalog_generation(&mut self) {
        self.schema_catalog_generation = self.schema_catalog_generation.saturating_add(1);
    }
}

fn validate_vm_registration(
    registration: &ReflectTypeRegistration,
    backing: VmTypeBacking,
) -> Result<&str, ReflectError> {
    validate_registration(registration)?;
    let type_path = registration.type_path.type_path();
    validate_canonical_text(type_path, "display name", &registration.display_name)?;
    if !registration.is_plugin_owned() {
        return Err(invalid_vm_registration(
            type_path,
            "VM types must be plugin-owned",
        ));
    }
    let Some(plugin_id) = registration.type_path.plugin_id() else {
        return Err(invalid_vm_registration(
            type_path,
            "VM types must declare a plugin id",
        ));
    };
    let plugin_prefix = format!("{plugin_id}.");
    if !type_path.starts_with(&plugin_prefix) {
        return Err(invalid_vm_registration(
            type_path,
            &format!("VM full type path must begin with `{plugin_prefix}`"),
        ));
    }
    for field in &registration.type_info.fields {
        if let Err(reason) = DeclaredValueType::parse_vm(&field.value_type_path) {
            return Err(invalid_vm_registration(
                type_path,
                &format!("reflected field `{}` {reason}", field.name),
            ));
        }
    }

    match backing {
        VmTypeBacking::DynamicComponent if !registration.is_component() => {
            Err(invalid_vm_registration(
                type_path,
                "dynamic VM backing requires a component-only registration",
            ))
        }
        VmTypeBacking::DynamicComponent => Ok(plugin_id),
    }
}

fn validate_registration(registration: &ReflectTypeRegistration) -> Result<(), ReflectError> {
    let type_path = registration.type_path.type_path();
    if registration.type_info.fields.len() > MAX_REFLECT_FIELDS_PER_TYPE {
        return Err(ReflectError::InvalidRegistration {
            type_path: type_path.to_string(),
            reason: format!(
                "reflected types must not declare more than {MAX_REFLECT_FIELDS_PER_TYPE} fields"
            ),
        });
    }

    ReflectSchemaCatalog::validate_entry(&ReflectSchemaCatalogEntry::new(registration.clone()))?;

    let mut enum_option_count = 0_usize;
    for field in &registration.type_info.fields {
        validate_field_text(
            type_path,
            field,
            "display name",
            &field.display_name,
            MAX_REFLECT_FIELD_DISPLAY_NAME_BYTES,
        )?;
        let declared = DeclaredValueType::parse(&field.value_type_path)
            .map_err(|reason| invalid_field_registration(type_path, &field.name, &reason))?;
        validate_editor_hint(type_path, field, &declared)?;
        if let Some(default_value) = &field.default_value {
            validate_reflected_value_contract(default_value).map_err(|error| {
                invalid_field_registration(
                    type_path,
                    &field.name,
                    &format!("default value rejected: {error}"),
                )
            })?;
            if !declared.matches_reflected(default_value)
                && !(declared.is_named()
                    && editor_hint_matches_value(&field.editor_hint, default_value))
            {
                return Err(invalid_field_registration(
                    type_path,
                    &field.name,
                    &format!(
                        "default value type `{}` does not match declared value type `{}`",
                        default_value.type_name(),
                        field.value_type_path
                    ),
                ));
            }
        }
        validate_numeric_metadata(type_path, field, &declared)?;
        enum_option_count = enum_option_count
            .checked_add(field.enum_options.len())
            .ok_or_else(|| ReflectError::InvalidRegistration {
                type_path: type_path.to_string(),
                reason: "reflected enum option count overflowed".to_string(),
            })?;
        if enum_option_count > MAX_REFLECT_ENUM_OPTIONS_PER_TYPE {
            return Err(ReflectError::InvalidRegistration {
                type_path: type_path.to_string(),
                reason: format!(
                    "reflected types must not declare more than {MAX_REFLECT_ENUM_OPTIONS_PER_TYPE} total enum options"
                ),
            });
        }
        validate_enum_metadata(type_path, field, &declared)?;
    }
    Ok(())
}

fn validate_field_text(
    type_path: &str,
    field: &ReflectFieldInfo,
    label: &str,
    value: &str,
    max_bytes: usize,
) -> Result<(), ReflectError> {
    if value.is_empty() || value.trim() != value {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("field {label} must be non-empty and already trimmed"),
        ));
    }
    if value.len() > max_bytes {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("field {label} must not exceed {max_bytes} bytes"),
        ));
    }
    Ok(())
}

fn validate_editor_hint(
    type_path: &str,
    field: &ReflectFieldInfo,
    declared: &DeclaredValueType,
) -> Result<(), ReflectError> {
    let compatible = match &field.editor_hint {
        ReflectEditorHint::None | ReflectEditorHint::Json => true,
        ReflectEditorHint::String | ReflectEditorHint::MultilineString => {
            matches!(declared, DeclaredValueType::String) || declared.is_named()
        }
        ReflectEditorHint::Bool => {
            matches!(declared, DeclaredValueType::Bool) || declared.is_named()
        }
        ReflectEditorHint::Integer => {
            matches!(declared, DeclaredValueType::Integer) || declared.is_named()
        }
        ReflectEditorHint::Unsigned => {
            matches!(declared, DeclaredValueType::Unsigned) || declared.is_named()
        }
        ReflectEditorHint::Scalar => {
            matches!(declared, DeclaredValueType::Scalar) || declared.is_named()
        }
        ReflectEditorHint::Vec2 => {
            matches!(declared, DeclaredValueType::Vec2) || declared.is_named()
        }
        ReflectEditorHint::Vec3 => {
            matches!(declared, DeclaredValueType::Vec3) || declared.is_named()
        }
        ReflectEditorHint::Vec4 => {
            matches!(
                declared,
                DeclaredValueType::Vec4 | DeclaredValueType::Quaternion
            ) || declared.is_named()
        }
        ReflectEditorHint::Enum => declared.supports_enum_metadata() || declared.is_named(),
        ReflectEditorHint::Entity => {
            matches!(declared, DeclaredValueType::Entity) || declared.is_named()
        }
        ReflectEditorHint::Resource => {
            matches!(declared, DeclaredValueType::Resource) || declared.is_named()
        }
        ReflectEditorHint::Color => {
            matches!(declared, DeclaredValueType::Vec3 | DeclaredValueType::Vec4)
                || declared.is_named()
        }
    };
    if compatible {
        return Ok(());
    }
    Err(invalid_field_registration(
        type_path,
        &field.name,
        &format!(
            "editor hint `{:?}` is incompatible with declared value type `{}`",
            field.editor_hint, field.value_type_path
        ),
    ))
}

fn validate_numeric_metadata(
    type_path: &str,
    field: &ReflectFieldInfo,
    declared: &DeclaredValueType,
) -> Result<(), ReflectError> {
    if field.numeric_range.is_none()
        || declared.supports_numeric_metadata()
        || (declared.is_named()
            && matches!(
                &field.editor_hint,
                ReflectEditorHint::Integer
                    | ReflectEditorHint::Unsigned
                    | ReflectEditorHint::Scalar
            ))
    {
        return Ok(());
    }
    Err(invalid_field_registration(
        type_path,
        &field.name,
        "numeric range metadata requires an integer, unsigned, or scalar field",
    ))
}

fn validate_enum_metadata(
    type_path: &str,
    field: &ReflectFieldInfo,
    declared: &DeclaredValueType,
) -> Result<(), ReflectError> {
    if field.enum_options.len() > MAX_REFLECT_ENUM_OPTIONS_PER_FIELD {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!(
                "enum fields must not declare more than {MAX_REFLECT_ENUM_OPTIONS_PER_FIELD} options"
            ),
        ));
    }
    if field.enum_options.is_empty() {
        return Ok(());
    }
    if !declared.supports_enum_metadata()
        && !(declared.is_named() && field.editor_hint == ReflectEditorHint::Enum)
    {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            "enum options require an enum field",
        ));
    }

    let mut values = HashSet::with_capacity(field.enum_options.len());
    for option in &field.enum_options {
        validate_enum_option_text(
            type_path,
            field,
            "value",
            &option.value,
            MAX_REFLECT_ENUM_VALUE_BYTES,
        )?;
        if !valid_enum_value(&option.value) {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                "enum option values must use ASCII letters, digits, `_`, `-`, `.`, or `:`",
            ));
        }
        validate_enum_option_text(
            type_path,
            field,
            "display name",
            &option.display_name,
            MAX_REFLECT_ENUM_DISPLAY_NAME_BYTES,
        )?;
        if !values.insert(option.value.as_str()) {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!("duplicate enum option value `{}`", option.value),
            ));
        }
    }
    if let Some(ReflectedValue::Enum(default)) = &field.default_value {
        if !values.contains(default.as_str()) {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!("enum default `{default}` is not present in enum options"),
            ));
        }
    }
    Ok(())
}

fn validate_enum_option_text(
    type_path: &str,
    field: &ReflectFieldInfo,
    label: &str,
    value: &str,
    max_bytes: usize,
) -> Result<(), ReflectError> {
    if value.is_empty() || value.trim() != value {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("enum option {label} must be non-empty and already trimmed"),
        ));
    }
    if value.len() > max_bytes {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("enum option {label} must not exceed {max_bytes} bytes"),
        ));
    }
    Ok(())
}

fn editor_hint_matches_value(hint: &ReflectEditorHint, value: &ReflectedValue) -> bool {
    match (hint, value) {
        (
            ReflectEditorHint::String | ReflectEditorHint::MultilineString,
            ReflectedValue::String(_),
        )
        | (ReflectEditorHint::Bool, ReflectedValue::Bool(_))
        | (ReflectEditorHint::Integer, ReflectedValue::Integer(_))
        | (ReflectEditorHint::Unsigned, ReflectedValue::Unsigned(_))
        | (ReflectEditorHint::Enum, ReflectedValue::Enum(_))
        | (ReflectEditorHint::Resource, ReflectedValue::Resource(_))
        | (ReflectEditorHint::Json, ReflectedValue::Json(_)) => true,
        (ReflectEditorHint::Scalar, ReflectedValue::Scalar(value)) => value.is_finite(),
        (ReflectEditorHint::Vec2, ReflectedValue::Vec2(values)) => finite(values),
        (ReflectEditorHint::Vec3 | ReflectEditorHint::Color, ReflectedValue::Vec3(values)) => {
            finite(values)
        }
        (
            ReflectEditorHint::Vec4 | ReflectEditorHint::Color,
            ReflectedValue::Vec4(values) | ReflectedValue::Quaternion(values),
        ) => finite(values),
        (ReflectEditorHint::Entity, ReflectedValue::Entity(_) | ReflectedValue::Null) => true,
        _ => false,
    }
}

fn finite(values: &[f32]) -> bool {
    values.iter().all(|value| value.is_finite())
}

fn valid_enum_value(value: &str) -> bool {
    value.bytes().all(|byte| {
        byte == b'_' || byte == b'-' || byte == b'.' || byte == b':' || byte.is_ascii_alphanumeric()
    })
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

fn invalid_field_registration(type_path: &str, field_name: &str, reason: &str) -> ReflectError {
    ReflectError::InvalidFieldRegistration {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
        reason: reason.to_string(),
    }
}

impl fmt::Debug for TypeRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TypeRegistry")
            .field("registrations", &self.registrations)
            .field("schema_catalog", &self.schema_catalog)
            .field("schema_catalog_generation", &self.schema_catalog_generation)
            .finish()
    }
}

impl PartialEq for TypeRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.registrations == other.registrations && self.schema_catalog == other.schema_catalog
    }
}
