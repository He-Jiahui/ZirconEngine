use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldValue, ReflectValueBudget, ReflectValueValidationError,
    ReflectedValue,
};

const RUNTIME_REFLECT_VALUE_MAX_DEPTH: usize = 128;
const RUNTIME_REFLECT_VALUE_MAX_NODES: usize = 16_384;
const RUNTIME_REFLECT_VALUE_MAX_STRING_BYTES: usize = 1024 * 1024;
const RUNTIME_REFLECT_VALUE_MAX_CONTAINER_ENTRIES: usize = 4_096;

pub(crate) const RUNTIME_REFLECT_VALUE_BUDGET: ReflectValueBudget = ReflectValueBudget::new(
    RUNTIME_REFLECT_VALUE_MAX_DEPTH,
    RUNTIME_REFLECT_VALUE_MAX_NODES,
    RUNTIME_REFLECT_VALUE_MAX_STRING_BYTES,
    RUNTIME_REFLECT_VALUE_MAX_CONTAINER_ENTRIES,
);

pub(crate) fn validate_reflected_value(
    type_path: &str,
    field_name: &str,
    value: &ReflectedValue,
) -> Result<(), ReflectError> {
    validate_reflected_value_contract(value)
        .map_err(|error| invalid_value(type_path, field_name, error))
}

fn validate_reflected_field_value(
    type_path: &str,
    field: &ReflectFieldValue,
) -> Result<(), ReflectError> {
    validate_reflected_value(type_path, &field.field_name, &field.value)
}

pub(crate) fn validate_reflected_field_values(
    type_path: &str,
    fields: &[ReflectFieldValue],
) -> Result<(), ReflectError> {
    for field in fields {
        validate_reflected_field_value(type_path, field)?;
    }
    Ok(())
}

pub(crate) fn validate_reflected_value_contract(
    value: &ReflectedValue,
) -> Result<(), ReflectValueValidationError> {
    value.validate_with_budget(RUNTIME_REFLECT_VALUE_BUDGET)
}

fn invalid_value(
    type_path: &str,
    field_name: &str,
    error: ReflectValueValidationError,
) -> ReflectError {
    ReflectError::InvalidValue {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
        reason: error.to_string(),
    }
}
