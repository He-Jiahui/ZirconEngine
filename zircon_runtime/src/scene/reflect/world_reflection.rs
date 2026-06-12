use crate::scene::{
    reflect::{ReflectComponent, ReflectResource, TypeRegistry},
    World,
};
use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldValue, ReflectFieldsRequest, ReflectFieldsResponse,
    ReflectObjectAddress, ReflectReadRequest, ReflectReadResponse, ReflectSchemaFilter,
    ReflectSchemaRequest, ReflectSchemaResponse, ReflectTypeRegistration, ReflectWriteRequest,
    ReflectWriteResponse, ReflectedValue,
};

pub struct WorldReflection;

impl WorldReflection {
    pub fn list_reflect_types(
        world: &World,
        request: ReflectSchemaRequest,
    ) -> Result<ReflectSchemaResponse, ReflectError> {
        let filter = request.filter;
        let registrations = if let Some(type_path) = filter.type_path.as_deref() {
            let registration = world.type_registry().runtime_registration(type_path)?;
            let mut registrations = Vec::with_capacity(1);
            if schema_filter_matches(&registration.registration, &filter) {
                registrations.push(registration.registration.clone());
            }
            registrations
        } else {
            let registry_entries = world.type_registry().iter();
            let mut registrations = Vec::with_capacity(registry_entries.size_hint().0);
            for registration in registry_entries {
                if schema_filter_matches(&registration.registration, &filter) {
                    registrations.push(registration.registration.clone());
                }
            }
            registrations
        };

        Ok(ReflectSchemaResponse::new(registrations))
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
                let adapter = component_adapter(world, type_path)?;
                adapter.read_fields(world, *entity)?
            }
            ReflectObjectAddress::Resource { type_path } => {
                let adapter = resource_adapter_ref(world, type_path)?;
                adapter.read_fields(world)?
            }
        };

        Ok(ReflectFieldsResponse::new(request.address, fields))
    }

    pub fn reflect_read(
        world: &World,
        request: ReflectReadRequest,
    ) -> Result<ReflectReadResponse, ReflectError> {
        let value = read_reflected_field(world, &request.address, &request.field_name)?;
        let field = ReflectFieldValue::new(request.field_name, value);

        Ok(ReflectReadResponse::new(request.address, field))
    }

    pub fn reflect_write(
        world: &mut World,
        request: ReflectWriteRequest,
    ) -> Result<ReflectWriteResponse, ReflectError> {
        let changed = match &request.address {
            ReflectObjectAddress::Component { entity, type_path } => {
                let adapter = component_adapter_for_write(world, type_path)?;
                adapter.write_field(world, *entity, &request.field_name, request.value)?
            }
            ReflectObjectAddress::Resource { type_path } => {
                let adapter = resource_adapter_for_write(world, type_path)?;
                adapter.write_field(world, &request.field_name, request.value)?
            }
        };

        let value = read_reflected_field(world, &request.address, &request.field_name)?;
        let field = ReflectFieldValue::new(request.field_name, value);

        Ok(ReflectWriteResponse::new(request.address, field, changed))
    }
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
        && !((filter.include_components && registration.is_component)
            || (filter.include_resources && registration.is_resource))
    {
        return false;
    }

    if filter.editor_visible && !registration.editor_visible {
        return false;
    }

    if filter.remote_visible && !registration.remote_visible {
        return false;
    }

    if !filter.include_plugin_owned && registration.plugin_owned {
        return false;
    }

    true
}

fn component_adapter<'a>(
    world: &'a World,
    type_path: &str,
) -> Result<&'a ReflectComponent, ReflectError> {
    let registration = world.type_registry().runtime_registration(type_path)?;
    if !registration.registration.is_component {
        return Err(ReflectError::AddressKindMismatch {
            expected: format!(
                "component `{}`",
                registration.registration.type_path.type_path
            ),
            actual: format!(
                "non-component `{}`",
                registration.registration.type_path.type_path
            ),
        });
    }

    let Some(adapter) = registration.component.as_ref() else {
        return Err(ReflectError::NoComponentAdapter {
            type_path: registration.registration.type_path.type_path.clone(),
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
    if !registration.registration.is_resource {
        return Err(ReflectError::AddressKindMismatch {
            expected: format!(
                "resource `{}`",
                registration.registration.type_path.type_path
            ),
            actual: format!(
                "non-resource `{}`",
                registration.registration.type_path.type_path
            ),
        });
    }

    let Some(adapter) = registration.resource.as_ref() else {
        return Err(ReflectError::NoResourceAdapter {
            type_path: registration.registration.type_path.type_path.clone(),
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

fn read_reflected_field(
    world: &World,
    address: &ReflectObjectAddress,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    match address {
        ReflectObjectAddress::Component { entity, type_path } => {
            let adapter = component_adapter(world, type_path)?;
            adapter.read_field(world, *entity, field_name)
        }
        ReflectObjectAddress::Resource { type_path } => {
            let adapter = resource_adapter_ref(world, type_path)?;
            adapter.read_field(world, field_name)
        }
    }
}
