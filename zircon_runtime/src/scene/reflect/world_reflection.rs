use crate::scene::{
    reflect::{ReflectComponent, ReflectResource, TypeRegistry},
    World,
};
use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldId, ReflectFieldInfo, ReflectFieldValue, ReflectFieldsRequest,
    ReflectFieldsResponse, ReflectObjectAddress, ReflectReadRequest, ReflectReadResponse,
    ReflectSchemaFilter, ReflectSchemaRequest, ReflectSchemaResponse, ReflectTypeRegistration,
    ReflectWriteRequest, ReflectWriteResponse, ReflectedValue,
};

use super::validate_reflected_value;

pub struct WorldReflection;

impl WorldReflection {
    pub fn list_reflect_types(
        world: &World,
        request: ReflectSchemaRequest,
    ) -> Result<ReflectSchemaResponse, ReflectError> {
        let filter = request.filter;
        let schema_catalog = world.type_registry().schema_catalog();
        let registrations = if let Some(type_path) = filter.type_path.as_deref() {
            let registration = schema_catalog.registration(type_path)?;
            let mut registrations = Vec::with_capacity(1);
            if schema_filter_matches(registration, &filter) {
                registrations.push(registration.clone());
            }
            registrations
        } else {
            let registry_entries = schema_catalog.registrations();
            let mut registrations = Vec::with_capacity(registry_entries.size_hint().0);
            for registration in registry_entries {
                if schema_filter_matches(registration, &filter) {
                    registrations.push(registration.clone());
                }
            }
            registrations
        };

        Ok(ReflectSchemaResponse::new(
            schema_catalog.fingerprint(),
            registrations,
        ))
    }

    pub fn reflect_schema(
        world: &World,
        type_path: &str,
    ) -> Result<ReflectTypeRegistration, ReflectError> {
        Ok(world.type_registry().registration(type_path)?.clone())
    }

    pub fn reflect_fields(
        world: &World,
        request: ReflectFieldsRequest,
    ) -> Result<ReflectFieldsResponse, ReflectError> {
        let fields = match &request.address {
            ReflectObjectAddress::Component { entity, type_path } => {
                let registration = world.type_registry().registration(type_path)?;
                let adapter = component_adapter(world, type_path)?;
                Self::read_component_fields_by_slot(world, *entity, registration, adapter)?
            }
            ReflectObjectAddress::Resource { type_path } => {
                let registration = world.type_registry().registration(type_path)?;
                let adapter = resource_adapter_ref(world, type_path)?;
                Self::read_resource_fields_by_slot(world, registration, adapter)?
            }
        };

        Ok(ReflectFieldsResponse::new(request.address, fields))
    }

    pub fn reflect_read(
        world: &World,
        request: ReflectReadRequest,
    ) -> Result<ReflectReadResponse, ReflectError> {
        let type_path = request.address.type_path();
        let (field_slot, field_name) = field_access(world, type_path, request.field_id, false)?;
        let value = read_reflected_field_by_slot(world, &request.address, field_slot)?;
        validate_reflected_value(type_path, &field_name, &value)?;
        let field = ReflectFieldValue::new(request.field_id, field_name, value);

        Ok(ReflectReadResponse::new(request.address, field))
    }

    pub fn reflect_write(
        world: &mut World,
        request: ReflectWriteRequest,
    ) -> Result<ReflectWriteResponse, ReflectError> {
        let (changed, accepted_field) = match &request.address {
            ReflectObjectAddress::Component { entity, type_path } => {
                let adapter = component_adapter_for_write(world, type_path)?;
                let (field_slot, field_name) =
                    field_access(world, type_path, request.field_id, true)?;
                validate_reflected_value(type_path, &field_name, &request.value)?;
                // The response acknowledges the admitted request; observation remains an explicit read.
                let accepted_field =
                    ReflectFieldValue::new(request.field_id, field_name, request.value.clone());
                let changed =
                    adapter.write_field_by_slot(world, *entity, field_slot, request.value)?;
                (changed, accepted_field)
            }
            ReflectObjectAddress::Resource { type_path } => {
                let adapter = resource_adapter_for_write(world, type_path)?;
                let (field_slot, field_name) =
                    field_access(world, type_path, request.field_id, true)?;
                validate_reflected_value(type_path, &field_name, &request.value)?;
                let accepted_field =
                    ReflectFieldValue::new(request.field_id, field_name, request.value.clone());
                let changed = adapter.write_field_by_slot(world, field_slot, request.value)?;
                (changed, accepted_field)
            }
        };

        Ok(ReflectWriteResponse::new(
            request.address,
            accepted_field,
            changed,
        ))
    }
}

impl WorldReflection {
    pub(crate) fn read_component_fields_by_slot(
        world: &World,
        entity: crate::scene::EntityId,
        registration: &ReflectTypeRegistration,
        adapter: &ReflectComponent,
    ) -> Result<Vec<ReflectFieldValue>, ReflectError> {
        let type_path = registration.type_path.type_path();
        if !adapter.contains(world, entity) {
            return Err(ReflectError::MissingComponent {
                entity,
                type_path: type_path.to_string(),
            });
        }
        read_schema_fields_by_slot(type_path, &registration.type_info.fields, |slot| {
            adapter.read_field_by_slot(world, entity, slot)
        })
    }

    pub(crate) fn read_resource_fields_by_slot(
        world: &World,
        registration: &ReflectTypeRegistration,
        adapter: &ReflectResource,
    ) -> Result<Vec<ReflectFieldValue>, ReflectError> {
        let type_path = registration.type_path.type_path();
        if !adapter.contains(world) {
            return Err(ReflectError::MissingResource {
                type_path: type_path.to_string(),
            });
        }
        read_schema_fields_by_slot(type_path, &registration.type_info.fields, |slot| {
            adapter.read_field_by_slot(world, slot)
        })
    }
}

fn read_schema_fields_by_slot(
    type_path: &str,
    fields: &[ReflectFieldInfo],
    mut read: impl FnMut(u32) -> Result<ReflectedValue, ReflectError>,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    let mut values = Vec::with_capacity(fields.len());
    for (slot, field) in fields.iter().enumerate() {
        let slot = u32::try_from(slot).map_err(|_| ReflectError::InvalidRegistration {
            type_path: type_path.to_string(),
            reason: "reflection schema has more than u32::MAX fields".to_string(),
        })?;
        let value = read(slot)?;
        validate_reflected_value(type_path, &field.name, &value)?;
        values.push(ReflectFieldValue::new(field.id, field.name.clone(), value));
    }
    Ok(values)
}

impl World {
    pub fn type_registry(&self) -> &TypeRegistry {
        self.type_registry_for_reflection()
    }

    pub fn list_reflect_types(
        &self,
        request: ReflectSchemaRequest,
    ) -> Result<ReflectSchemaResponse, ReflectError> {
        WorldReflection::list_reflect_types(self, request)
    }

    pub fn reflect_schema(&self, type_path: &str) -> Result<ReflectTypeRegistration, ReflectError> {
        WorldReflection::reflect_schema(self, type_path)
    }

    pub fn reflect_fields(
        &self,
        request: ReflectFieldsRequest,
    ) -> Result<ReflectFieldsResponse, ReflectError> {
        WorldReflection::reflect_fields(self, request)
    }

    pub fn reflect_read(
        &self,
        request: ReflectReadRequest,
    ) -> Result<ReflectReadResponse, ReflectError> {
        WorldReflection::reflect_read(self, request)
    }

    pub fn reflect_write(
        &mut self,
        request: ReflectWriteRequest,
    ) -> Result<ReflectWriteResponse, ReflectError> {
        WorldReflection::reflect_write(self, request)
    }

    #[cfg(test)]
    pub(crate) fn type_registry_mut_for_tests(&mut self) -> &mut TypeRegistry {
        self.type_registry_mut_for_reflection()
    }
}

fn schema_filter_matches(
    registration: &ReflectTypeRegistration,
    filter: &ReflectSchemaFilter,
) -> bool {
    let category_filter_active = filter.include_components || filter.include_resources;
    if category_filter_active
        && !((filter.include_components && registration.is_component())
            || (filter.include_resources && registration.is_resource()))
    {
        return false;
    }

    if filter.editor_visible && !registration.editor_visible {
        return false;
    }

    if filter.remote_visible && !registration.remote_visible {
        return false;
    }

    if !filter.include_plugin_owned && registration.is_plugin_owned() {
        return false;
    }

    true
}

fn component_adapter<'a>(
    world: &'a World,
    type_path: &str,
) -> Result<&'a ReflectComponent, ReflectError> {
    let registration = world.type_registry().runtime_registration(type_path)?;
    if !registration.registration.is_component() {
        return Err(ReflectError::AddressKindMismatch {
            expected: format!(
                "component `{}`",
                registration.registration.type_path.type_path()
            ),
            actual: format!(
                "non-component `{}`",
                registration.registration.type_path.type_path()
            ),
        });
    }

    let Some(adapter) = registration.component.as_ref() else {
        return Err(ReflectError::NoComponentAdapter {
            type_path: registration.registration.type_path.type_path().to_string(),
        });
    };

    Ok(adapter)
}

fn component_adapter_for_write(
    world: &World,
    type_path: &str,
) -> Result<ReflectComponent, ReflectError> {
    component_adapter(world, type_path).cloned()
}

fn resource_adapter_ref<'a>(
    world: &'a World,
    type_path: &str,
) -> Result<&'a ReflectResource, ReflectError> {
    let registration = world.type_registry().runtime_registration(type_path)?;
    if !registration.registration.is_resource() {
        return Err(ReflectError::AddressKindMismatch {
            expected: format!(
                "resource `{}`",
                registration.registration.type_path.type_path()
            ),
            actual: format!(
                "non-resource `{}`",
                registration.registration.type_path.type_path()
            ),
        });
    }

    let Some(adapter) = registration.resource.as_ref() else {
        return Err(ReflectError::NoResourceAdapter {
            type_path: registration.registration.type_path.type_path().to_string(),
        });
    };

    Ok(adapter)
}

fn resource_adapter_for_write(
    world: &World,
    type_path: &str,
) -> Result<ReflectResource, ReflectError> {
    resource_adapter_ref(world, type_path).copied()
}

fn field_access(
    world: &World,
    type_path: &str,
    field_id: ReflectFieldId,
    require_editable: bool,
) -> Result<(u32, String), ReflectError> {
    let registry = world.type_registry();
    let registration = registry.runtime_registration(type_path)?;
    let field_slot = registry.resolve_field_slot_by_id(type_path, field_id)?;
    let Some(field) = registration
        .registration
        .type_info
        .fields
        .get(field_slot as usize)
        .filter(|field| field.id == field_id)
    else {
        return Err(ReflectError::InvalidRegistration {
            type_path: registration.registration.type_path.type_path().to_string(),
            reason: format!(
                "catalog slot {field_slot} does not project stable field ID `{field_id}`"
            ),
        });
    };
    if require_editable && !field.editable {
        return Err(ReflectError::NonEditableField {
            type_path: registration.registration.type_path.type_path().to_string(),
            field_name: field.name.clone(),
        });
    }
    Ok((field_slot, field.name.clone()))
}

fn read_reflected_field_by_slot(
    world: &World,
    address: &ReflectObjectAddress,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    match address {
        ReflectObjectAddress::Component { entity, type_path } => {
            let adapter = component_adapter(world, type_path)?;
            adapter.read_field_by_slot(world, *entity, field_slot)
        }
        ReflectObjectAddress::Resource { type_path } => {
            let adapter = resource_adapter_ref(world, type_path)?;
            adapter.read_field_by_slot(world, field_slot)
        }
    }
}
