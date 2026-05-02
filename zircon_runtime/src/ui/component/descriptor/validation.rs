use std::collections::BTreeSet;

use thiserror::Error;

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiHostCapability, UiPropSchema, UiRenderCapability, UiValue, UiValueKind,
};

#[derive(Clone, Debug, Error, PartialEq)]
pub enum UiComponentDescriptorError {
    #[error("component descriptor id must not be empty")]
    EmptyId,
    #[error("component descriptor `{component_id}` display name must not be empty")]
    EmptyDisplayName { component_id: String },
    #[error("component descriptor `{component_id}` role must not be empty")]
    EmptyRole { component_id: String },
    #[error("component descriptor `{component_id}` has duplicate {schema_kind} schema `{name}`")]
    DuplicateSchemaName {
        component_id: String,
        schema_kind: &'static str,
        name: String,
    },
    #[error("component descriptor `{component_id}` has duplicate slot schema `{name}`")]
    DuplicateSlotName { component_id: String, name: String },
    #[error("component descriptor `{component_id}` default prop `{name}` has no prop schema")]
    MissingDefaultPropSchema { component_id: String, name: String },
    #[error("component descriptor `{component_id}` schema `{name}` default uses {actual:?}, expected {expected:?}")]
    DefaultValueKindMismatch {
        component_id: String,
        name: String,
        expected: UiValueKind,
        actual: UiValueKind,
    },
    #[error("component descriptor `{component_id}` schema `{name}` has invalid range")]
    InvalidRange { component_id: String, name: String },
    #[error("component descriptor `{component_id}` schema `{name}` has invalid step")]
    InvalidStep { component_id: String, name: String },
    #[error("component descriptor `{component_id}` numeric value `{name}` must be finite")]
    NonFiniteNumber { component_id: String, name: String },
    #[error(
        "component descriptor `{component_id}` palette metadata is missing a default node template"
    )]
    MissingPaletteDefaultNode { component_id: String },
    #[error("component descriptor `{component_id}` palette sort key must not be empty")]
    EmptyPaletteSortKey { component_id: String },
    #[error(
        "component descriptor `{component_id}` default node slot `{slot_name}` is not declared"
    )]
    UnknownDefaultNodeSlot {
        component_id: String,
        slot_name: String,
    },
    #[error("component descriptor `{component_id}` virtualized render capability requires virtualized host capability")]
    MissingVirtualizedHostCapability { component_id: String },
}

pub fn validate_component_descriptor(
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentDescriptorError> {
    if descriptor.id.trim().is_empty() {
        return Err(UiComponentDescriptorError::EmptyId);
    }
    if descriptor.display_name.trim().is_empty() {
        return Err(UiComponentDescriptorError::EmptyDisplayName {
            component_id: descriptor.id.clone(),
        });
    }
    if descriptor.role.trim().is_empty() {
        return Err(UiComponentDescriptorError::EmptyRole {
            component_id: descriptor.id.clone(),
        });
    }

    validate_schema_names(descriptor, "prop", &descriptor.prop_schema)?;
    validate_schema_names(descriptor, "state", &descriptor.state_schema)?;
    validate_slot_names(descriptor)?;
    validate_schema_defaults(descriptor)?;
    validate_default_props(descriptor)?;
    validate_palette(descriptor)?;
    validate_capabilities(descriptor)?;
    Ok(())
}

fn validate_schema_names(
    descriptor: &UiComponentDescriptor,
    schema_kind: &'static str,
    schemas: &[UiPropSchema],
) -> Result<(), UiComponentDescriptorError> {
    let mut names = BTreeSet::new();
    for schema in schemas {
        if schema.name.trim().is_empty() || !names.insert(schema.name.as_str()) {
            return Err(UiComponentDescriptorError::DuplicateSchemaName {
                component_id: descriptor.id.clone(),
                schema_kind,
                name: schema.name.clone(),
            });
        }

        let mut option_ids = BTreeSet::new();
        for option in &schema.options {
            if !option_ids.insert(option.id.as_str()) {
                return Err(UiComponentDescriptorError::DuplicateSchemaName {
                    component_id: descriptor.id.clone(),
                    schema_kind: "option",
                    name: option.id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn validate_slot_names(
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentDescriptorError> {
    let mut names = BTreeSet::new();
    for slot in &descriptor.slot_schema {
        if slot.name.trim().is_empty() || !names.insert(slot.name.as_str()) {
            return Err(UiComponentDescriptorError::DuplicateSlotName {
                component_id: descriptor.id.clone(),
                name: slot.name.clone(),
            });
        }
    }
    Ok(())
}

fn validate_schema_defaults(
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentDescriptorError> {
    for schema in descriptor
        .prop_schema
        .iter()
        .chain(descriptor.state_schema.iter())
    {
        if let Some(default_value) = &schema.default_value {
            validate_value_kind(
                descriptor,
                &schema.name,
                schema.value_kind,
                default_value.kind(),
            )?;
            validate_finite_value(descriptor, &schema.name, default_value)?;
        }
        if let (Some(min), Some(max)) = (schema.min, schema.max) {
            if !min.is_finite() || !max.is_finite() || min > max {
                return Err(UiComponentDescriptorError::InvalidRange {
                    component_id: descriptor.id.clone(),
                    name: schema.name.clone(),
                });
            }
        }
        if schema
            .step
            .is_some_and(|step| !step.is_finite() || step <= 0.0)
        {
            return Err(UiComponentDescriptorError::InvalidStep {
                component_id: descriptor.id.clone(),
                name: schema.name.clone(),
            });
        }
    }
    Ok(())
}

fn validate_default_props(
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentDescriptorError> {
    for (name, value) in &descriptor.default_props {
        let Some(schema) = descriptor.prop(name) else {
            return Err(UiComponentDescriptorError::MissingDefaultPropSchema {
                component_id: descriptor.id.clone(),
                name: name.clone(),
            });
        };
        validate_value_kind(descriptor, name, schema.value_kind, value.kind())?;
        validate_finite_value(descriptor, name, value)?;
    }
    Ok(())
}

fn validate_finite_value(
    descriptor: &UiComponentDescriptor,
    name: &str,
    value: &UiValue,
) -> Result<(), UiComponentDescriptorError> {
    match value {
        UiValue::Float(value) => validate_finite_number(descriptor, name, *value),
        UiValue::Vec2(values) => values
            .iter()
            .try_for_each(|value| validate_finite_number(descriptor, name, *value)),
        UiValue::Vec3(values) => values
            .iter()
            .try_for_each(|value| validate_finite_number(descriptor, name, *value)),
        UiValue::Vec4(values) => values
            .iter()
            .try_for_each(|value| validate_finite_number(descriptor, name, *value)),
        UiValue::Array(values) => values
            .iter()
            .try_for_each(|value| validate_finite_value(descriptor, name, value)),
        UiValue::Map(values) => values
            .values()
            .try_for_each(|value| validate_finite_value(descriptor, name, value)),
        _ => Ok(()),
    }
}

fn validate_finite_number(
    descriptor: &UiComponentDescriptor,
    name: &str,
    value: f64,
) -> Result<(), UiComponentDescriptorError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(UiComponentDescriptorError::NonFiniteNumber {
            component_id: descriptor.id.clone(),
            name: name.to_string(),
        })
    }
}

fn validate_palette(descriptor: &UiComponentDescriptor) -> Result<(), UiComponentDescriptorError> {
    let Some(metadata) = &descriptor.palette else {
        return Ok(());
    };
    if metadata.sort_key.trim().is_empty() {
        return Err(UiComponentDescriptorError::EmptyPaletteSortKey {
            component_id: descriptor.id.clone(),
        });
    }
    if metadata.default_node.is_empty() {
        return Err(UiComponentDescriptorError::MissingPaletteDefaultNode {
            component_id: descriptor.id.clone(),
        });
    }
    if let Some(slot_name) = metadata.default_node.slot_name.as_deref() {
        if descriptor.slot_schema(slot_name).is_none() {
            return Err(UiComponentDescriptorError::UnknownDefaultNodeSlot {
                component_id: descriptor.id.clone(),
                slot_name: slot_name.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_capabilities(
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentDescriptorError> {
    if descriptor
        .required_render_capabilities
        .contains(&UiRenderCapability::VirtualizedLayout)
        && !descriptor
            .required_host_capabilities
            .contains(&UiHostCapability::VirtualizedLayout)
    {
        return Err(
            UiComponentDescriptorError::MissingVirtualizedHostCapability {
                component_id: descriptor.id.clone(),
            },
        );
    }
    Ok(())
}

fn validate_value_kind(
    descriptor: &UiComponentDescriptor,
    name: &str,
    expected: UiValueKind,
    actual: UiValueKind,
) -> Result<(), UiComponentDescriptorError> {
    if expected == UiValueKind::Any || expected == actual {
        return Ok(());
    }
    Err(UiComponentDescriptorError::DefaultValueKindMismatch {
        component_id: descriptor.id.clone(),
        name: name.to_string(),
        expected,
        actual,
    })
}
