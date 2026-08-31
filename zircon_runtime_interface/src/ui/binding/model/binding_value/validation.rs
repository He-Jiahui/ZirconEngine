use super::{
    types::validate_identity, UiBindingMapKey, UiBindingValue, UiBindingValueBudget,
    UiBindingValueIdentityKind, UiBindingValueValidationError,
};

impl UiBindingValue {
    pub fn validate(&self) -> Result<(), UiBindingValueValidationError> {
        self.validate_with_budget(UiBindingValueBudget::STANDARD)
    }

    pub fn validate_with_budget(
        &self,
        budget: UiBindingValueBudget,
    ) -> Result<(), UiBindingValueValidationError> {
        let mut state = UiBindingValueValidationState::default();
        self.validate_at_depth(budget, &mut state, 1)
    }

    fn validate_at_depth(
        &self,
        budget: UiBindingValueBudget,
        state: &mut UiBindingValueValidationState,
        depth: usize,
    ) -> Result<(), UiBindingValueValidationError> {
        if depth > budget.max_depth {
            return Err(UiBindingValueValidationError::DepthExceeded {
                actual: depth,
                maximum: budget.max_depth,
            });
        }
        state.add_node(budget)?;
        match self {
            Self::String(value) => state.add_string(budget, value),
            Self::Float(value) if !value.is_finite() => {
                Err(UiBindingValueValidationError::NonFiniteFloat)
            }
            Self::Array(values) => {
                validate_collection_len(values.len(), budget)?;
                for value in values {
                    value.validate_at_depth(budget, state, depth + 1)?;
                }
                Ok(())
            }
            Self::Record(fields) => {
                validate_collection_len(fields.len(), budget)?;
                for (field, value) in fields {
                    validate_identity(UiBindingValueIdentityKind::RecordField, field)?;
                    state.add_string(budget, field)?;
                    value.validate_at_depth(budget, state, depth + 1)?;
                }
                Ok(())
            }
            Self::Map(values) => {
                validate_collection_len(values.len(), budget)?;
                for (key, value) in values.iter() {
                    state.add_node(budget)?;
                    if let UiBindingMapKey::String(key) = key {
                        state.add_string(budget, key)?;
                    }
                    value.validate_at_depth(budget, state, depth + 1)?;
                }
                Ok(())
            }
            Self::Enum(value) => {
                validate_identity(UiBindingValueIdentityKind::EnumType, value.type_id())?;
                validate_identity(UiBindingValueIdentityKind::EnumVariant, value.variant())?;
                state.add_string(budget, value.type_id())?;
                state.add_string(budget, value.variant())?;
                if let Some(payload) = value.payload() {
                    payload.validate_at_depth(budget, state, depth + 1)?;
                }
                Ok(())
            }
            Self::Asset(value) => {
                validate_identity(UiBindingValueIdentityKind::AssetLocator, value.locator())?;
                state.add_string(budget, value.locator())
            }
            Self::Entity(value) => {
                if value.generation() == 0 {
                    Err(UiBindingValueValidationError::ZeroGeneration {
                        kind: UiBindingValueIdentityKind::EntityGeneration,
                    })
                } else {
                    Ok(())
                }
            }
            Self::Optional(value) => {
                if let Some(value) = value.as_deref() {
                    value.validate_at_depth(budget, state, depth + 1)?;
                }
                Ok(())
            }
            Self::CollectionView(value) => {
                value.validate_window()?;
                state.add_string(budget, value.provider().id.as_str())?;
                state.add_string(budget, value.item_schema().id.as_str())
            }
            Self::Unsigned(_) | Self::Signed(_) | Self::Float(_) | Self::Bool(_) | Self::Null => {
                Ok(())
            }
        }
    }
}

#[derive(Default)]
struct UiBindingValueValidationState {
    nodes: usize,
    string_bytes: usize,
}

impl UiBindingValueValidationState {
    fn add_node(
        &mut self,
        budget: UiBindingValueBudget,
    ) -> Result<(), UiBindingValueValidationError> {
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > budget.max_nodes {
            Err(UiBindingValueValidationError::NodeBudgetExceeded {
                actual: self.nodes,
                maximum: budget.max_nodes,
            })
        } else {
            Ok(())
        }
    }

    fn add_string(
        &mut self,
        budget: UiBindingValueBudget,
        value: &str,
    ) -> Result<(), UiBindingValueValidationError> {
        self.string_bytes = self.string_bytes.saturating_add(value.len());
        if self.string_bytes > budget.max_string_bytes {
            Err(UiBindingValueValidationError::StringBudgetExceeded {
                actual: self.string_bytes,
                maximum: budget.max_string_bytes,
            })
        } else {
            Ok(())
        }
    }
}

fn validate_collection_len(
    actual: usize,
    budget: UiBindingValueBudget,
) -> Result<(), UiBindingValueValidationError> {
    if actual > budget.max_collection_entries {
        Err(UiBindingValueValidationError::CollectionEntriesExceeded {
            actual,
            maximum: budget.max_collection_entries,
        })
    } else {
        Ok(())
    }
}
