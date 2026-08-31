use serde_json::Value;

use super::{
    ReflectValueBudget, ReflectValueBudgetDimension, ReflectValueFloatKind,
    ReflectValueValidationError, ReflectedValue,
};

enum PendingValue<'a> {
    Reflected {
        value: &'a ReflectedValue,
        depth: usize,
    },
    Json {
        value: &'a Value,
        depth: usize,
    },
}

impl ReflectedValue {
    pub fn validate_with_budget(
        &self,
        budget: ReflectValueBudget,
    ) -> Result<(), ReflectValueValidationError> {
        let mut state = ValidationState::default();
        let mut pending = vec![PendingValue::Reflected {
            value: self,
            depth: 1,
        }];

        while let Some(value) = pending.pop() {
            match value {
                PendingValue::Reflected { value, depth } => {
                    state.visit(depth, budget)?;
                    validate_reflected(value, depth, budget, &mut state, &mut pending)?;
                }
                PendingValue::Json { value, depth } => {
                    state.visit(depth, budget)?;
                    validate_json(value, depth, budget, &mut state, &mut pending)?;
                }
            }
        }
        Ok(())
    }
}

fn validate_reflected<'a>(
    value: &'a ReflectedValue,
    depth: usize,
    budget: ReflectValueBudget,
    state: &mut ValidationState,
    pending: &mut Vec<PendingValue<'a>>,
) -> Result<(), ReflectValueValidationError> {
    match value {
        ReflectedValue::Scalar(value) => {
            validate_finite(ReflectValueFloatKind::Scalar, std::slice::from_ref(value))
        }
        ReflectedValue::Vec2(values) => validate_finite(ReflectValueFloatKind::Vec2, values),
        ReflectedValue::Vec3(values) => validate_finite(ReflectValueFloatKind::Vec3, values),
        ReflectedValue::Vec4(values) => validate_finite(ReflectValueFloatKind::Vec4, values),
        ReflectedValue::Quaternion(values) => {
            validate_finite(ReflectValueFloatKind::Quaternion, values)
        }
        ReflectedValue::String(value)
        | ReflectedValue::Enum(value)
        | ReflectedValue::Resource(value) => state.add_string(value, budget),
        ReflectedValue::List(values) => {
            validate_container(values.len(), budget)?;
            if !values.is_empty() {
                let depth = child_depth(depth, budget)?;
                pending.extend(
                    values
                        .iter()
                        .rev()
                        .map(|value| PendingValue::Reflected { value, depth }),
                );
            }
            Ok(())
        }
        ReflectedValue::Map(values) => {
            validate_container(values.len(), budget)?;
            for key in values.keys() {
                state.add_string(key, budget)?;
            }
            if !values.is_empty() {
                let depth = child_depth(depth, budget)?;
                pending.extend(
                    values
                        .values()
                        .rev()
                        .map(|value| PendingValue::Reflected { value, depth }),
                );
            }
            Ok(())
        }
        ReflectedValue::Json(value) => {
            pending.push(PendingValue::Json {
                value,
                depth: child_depth(depth, budget)?,
            });
            Ok(())
        }
        ReflectedValue::Null
        | ReflectedValue::Bool(_)
        | ReflectedValue::Integer(_)
        | ReflectedValue::Unsigned(_)
        | ReflectedValue::Entity(_) => Ok(()),
    }
}

fn validate_json<'a>(
    value: &'a Value,
    depth: usize,
    budget: ReflectValueBudget,
    state: &mut ValidationState,
    pending: &mut Vec<PendingValue<'a>>,
) -> Result<(), ReflectValueValidationError> {
    match value {
        Value::String(value) => state.add_string(value, budget),
        Value::Array(values) => {
            validate_container(values.len(), budget)?;
            if !values.is_empty() {
                let depth = child_depth(depth, budget)?;
                pending.extend(
                    values
                        .iter()
                        .rev()
                        .map(|value| PendingValue::Json { value, depth }),
                );
            }
            Ok(())
        }
        Value::Object(values) => {
            validate_container(values.len(), budget)?;
            for key in values.keys() {
                state.add_string(key, budget)?;
            }
            if !values.is_empty() {
                let depth = child_depth(depth, budget)?;
                pending.extend(
                    values
                        .values()
                        .rev()
                        .map(|value| PendingValue::Json { value, depth }),
                );
            }
            Ok(())
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(()),
    }
}

#[derive(Default)]
struct ValidationState {
    nodes: usize,
    string_bytes: usize,
}

impl ValidationState {
    fn visit(
        &mut self,
        depth: usize,
        budget: ReflectValueBudget,
    ) -> Result<(), ReflectValueValidationError> {
        if depth > budget.max_depth() {
            return Err(ReflectValueValidationError::DepthExceeded {
                actual: depth,
                maximum: budget.max_depth(),
            });
        }
        let actual = self.nodes.checked_add(1).ok_or(
            ReflectValueValidationError::BudgetArithmeticOverflow {
                dimension: ReflectValueBudgetDimension::Nodes,
            },
        )?;
        if actual > budget.max_nodes() {
            return Err(ReflectValueValidationError::NodeBudgetExceeded {
                actual,
                maximum: budget.max_nodes(),
            });
        }
        self.nodes = actual;
        Ok(())
    }

    fn add_string(
        &mut self,
        value: &str,
        budget: ReflectValueBudget,
    ) -> Result<(), ReflectValueValidationError> {
        let actual = self.string_bytes.checked_add(value.len()).ok_or(
            ReflectValueValidationError::BudgetArithmeticOverflow {
                dimension: ReflectValueBudgetDimension::StringBytes,
            },
        )?;
        if actual > budget.max_string_bytes() {
            return Err(ReflectValueValidationError::StringBudgetExceeded {
                actual,
                maximum: budget.max_string_bytes(),
            });
        }
        self.string_bytes = actual;
        Ok(())
    }
}

fn validate_container(
    actual: usize,
    budget: ReflectValueBudget,
) -> Result<(), ReflectValueValidationError> {
    if actual > budget.max_container_entries() {
        Err(ReflectValueValidationError::ContainerEntriesExceeded {
            actual,
            maximum: budget.max_container_entries(),
        })
    } else {
        Ok(())
    }
}

fn child_depth(
    depth: usize,
    budget: ReflectValueBudget,
) -> Result<usize, ReflectValueValidationError> {
    let actual =
        depth
            .checked_add(1)
            .ok_or(ReflectValueValidationError::BudgetArithmeticOverflow {
                dimension: ReflectValueBudgetDimension::Depth,
            })?;
    if actual > budget.max_depth() {
        Err(ReflectValueValidationError::DepthExceeded {
            actual,
            maximum: budget.max_depth(),
        })
    } else {
        Ok(actual)
    }
}

fn validate_finite(
    kind: ReflectValueFloatKind,
    values: &[f32],
) -> Result<(), ReflectValueValidationError> {
    if let Some((component, _)) = values
        .iter()
        .enumerate()
        .find(|(_, value)| !value.is_finite())
    {
        Err(ReflectValueValidationError::NonFiniteFloat { kind, component })
    } else {
        Ok(())
    }
}
