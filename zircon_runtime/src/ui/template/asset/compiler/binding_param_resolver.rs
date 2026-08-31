use std::{collections::BTreeMap, fmt::Write};

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiValue,
    template::{
        UiAssetError, UiBindingExpression, UiBindingRef, UiComponentParamSchema, UiNodeDefinition,
    },
};

use crate::ui::template::asset::binding::component_param_kind;

pub(crate) fn typed_component_params(
    schemas: &BTreeMap<String, UiComponentParamSchema>,
    params: &BTreeMap<String, Value>,
    asset_id: &str,
) -> Result<BTreeMap<String, UiValue>, UiAssetError> {
    params
        .iter()
        .map(|(name, value)| {
            let schema = schemas
                .get(name)
                .ok_or_else(|| UiAssetError::InvalidDocument {
                    asset_id: asset_id.to_string(),
                    detail: format!("resolved unknown component param {name}"),
                })?;
            let kind = component_param_kind(&schema.r#type).ok_or_else(|| {
                UiAssetError::InvalidDocument {
                    asset_id: asset_id.to_string(),
                    detail: format!(
                        "component param {name} has unsupported type {}",
                        schema.r#type
                    ),
                }
            })?;
            let value = UiValue::from_toml_with_kind(value, kind).ok_or_else(|| {
                UiAssetError::InvalidDocument {
                    asset_id: asset_id.to_string(),
                    detail: format!("component param {name} cannot be represented as {kind:?}"),
                }
            })?;
            Ok((name.clone(), value))
        })
        .collect()
}

pub(super) fn resolve_node_binding_params(
    node: &mut UiNodeDefinition,
    params: &BTreeMap<String, UiValue>,
    asset_id: &str,
) -> Result<(), UiAssetError> {
    node.bindings = resolve_binding_params(std::mem::take(&mut node.bindings), params, asset_id)?;
    for child in &mut node.children {
        resolve_node_binding_params(&mut child.node, params, asset_id)?;
    }
    Ok(())
}

pub(crate) fn resolve_binding_params(
    mut bindings: Vec<UiBindingRef>,
    params: &BTreeMap<String, UiValue>,
    asset_id: &str,
) -> Result<Vec<UiBindingRef>, UiAssetError> {
    for binding in &mut bindings {
        for assignment in &mut binding.targets {
            if !probe_param_reference(asset_id, &binding.id, &assignment.expression)? {
                continue;
            }
            let expression = parse_expression(asset_id, &binding.id, &assignment.expression)?;
            let (expression, changed) =
                substitute_params(expression, params, asset_id, &binding.id)?;
            if changed {
                assignment.expression = expression_source(&expression, asset_id, &binding.id)?;
            }
        }

        let Some(action) = binding.action.as_mut() else {
            continue;
        };
        for value in action.payload.values_mut() {
            let Value::String(expression_text) = value else {
                continue;
            };
            if !expression_text.trim_start().starts_with('=') {
                continue;
            }
            if !probe_param_reference(asset_id, &binding.id, expression_text)? {
                continue;
            }
            let expression = parse_expression(asset_id, &binding.id, expression_text)?;
            let (expression, changed) =
                substitute_params(expression, params, asset_id, &binding.id)?;
            if !changed {
                continue;
            }
            *value = match expression {
                UiBindingExpression::Literal(value) => {
                    payload_literal_value(&value, asset_id, &binding.id)?
                }
                expression => Value::String(format!(
                    "={}",
                    expression_source(&expression, asset_id, &binding.id)?
                )),
            };
        }
    }
    Ok(bindings)
}

fn probe_param_reference(
    asset_id: &str,
    binding_id: &str,
    expression: &str,
) -> Result<bool, UiAssetError> {
    UiBindingExpression::probe_param_reference(expression).map_err(|error| {
        UiAssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: format!("binding {binding_id} has invalid expression: {error}"),
        }
    })
}

pub(super) fn parse_expression(
    asset_id: &str,
    binding_id: &str,
    expression: &str,
) -> Result<UiBindingExpression, UiAssetError> {
    UiBindingExpression::parse(expression).map_err(|error| UiAssetError::InvalidDocument {
        asset_id: asset_id.to_string(),
        detail: format!("binding {binding_id} has invalid expression: {error}"),
    })
}

fn substitute_params(
    expression: UiBindingExpression,
    params: &BTreeMap<String, UiValue>,
    asset_id: &str,
    binding_id: &str,
) -> Result<(UiBindingExpression, bool), UiAssetError> {
    match expression {
        UiBindingExpression::ParamRef(name) => params
            .get(&name)
            .map(|value| (UiBindingExpression::Literal(value.clone()), true))
            .ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: asset_id.to_string(),
                detail: format!("binding {binding_id} references missing component param {name}"),
            }),
        UiBindingExpression::Equals(lhs, rhs) => {
            substitute_binary(*lhs, *rhs, params, asset_id, binding_id, BinaryOp::Equals)
        }
        UiBindingExpression::NotEquals(lhs, rhs) => substitute_binary(
            *lhs,
            *rhs,
            params,
            asset_id,
            binding_id,
            BinaryOp::NotEquals,
        ),
        UiBindingExpression::And(lhs, rhs) => {
            substitute_binary(*lhs, *rhs, params, asset_id, binding_id, BinaryOp::And)
        }
        UiBindingExpression::Or(lhs, rhs) => {
            substitute_binary(*lhs, *rhs, params, asset_id, binding_id, BinaryOp::Or)
        }
        UiBindingExpression::Not(value) => {
            let (value, changed) = substitute_params(*value, params, asset_id, binding_id)?;
            let expression = match value {
                UiBindingExpression::Literal(UiValue::Bool(value)) => {
                    UiBindingExpression::Literal(UiValue::Bool(!value))
                }
                value => UiBindingExpression::Not(Box::new(value)),
            };
            Ok((expression, changed))
        }
        expression => Ok((expression, false)),
    }
}

#[derive(Clone, Copy)]
enum BinaryOp {
    Equals,
    NotEquals,
    And,
    Or,
}

fn substitute_binary(
    lhs: UiBindingExpression,
    rhs: UiBindingExpression,
    params: &BTreeMap<String, UiValue>,
    asset_id: &str,
    binding_id: &str,
    op: BinaryOp,
) -> Result<(UiBindingExpression, bool), UiAssetError> {
    let (lhs, lhs_changed) = substitute_params(lhs, params, asset_id, binding_id)?;
    let (rhs, rhs_changed) = substitute_params(rhs, params, asset_id, binding_id)?;
    let changed = lhs_changed || rhs_changed;
    let expression = match (&lhs, &rhs, op) {
        (
            UiBindingExpression::Literal(lhs),
            UiBindingExpression::Literal(rhs),
            BinaryOp::Equals,
        ) => UiBindingExpression::Literal(UiValue::Bool(lhs == rhs)),
        (
            UiBindingExpression::Literal(lhs),
            UiBindingExpression::Literal(rhs),
            BinaryOp::NotEquals,
        ) => UiBindingExpression::Literal(UiValue::Bool(lhs != rhs)),
        (
            UiBindingExpression::Literal(UiValue::Bool(lhs)),
            UiBindingExpression::Literal(UiValue::Bool(rhs)),
            BinaryOp::And,
        ) => UiBindingExpression::Literal(UiValue::Bool(*lhs && *rhs)),
        (
            UiBindingExpression::Literal(UiValue::Bool(lhs)),
            UiBindingExpression::Literal(UiValue::Bool(rhs)),
            BinaryOp::Or,
        ) => UiBindingExpression::Literal(UiValue::Bool(*lhs || *rhs)),
        (_, _, BinaryOp::Equals) => UiBindingExpression::Equals(Box::new(lhs), Box::new(rhs)),
        (_, _, BinaryOp::NotEquals) => UiBindingExpression::NotEquals(Box::new(lhs), Box::new(rhs)),
        (_, _, BinaryOp::And) => UiBindingExpression::And(Box::new(lhs), Box::new(rhs)),
        (_, _, BinaryOp::Or) => UiBindingExpression::Or(Box::new(lhs), Box::new(rhs)),
    };
    Ok((expression, changed))
}

pub(super) fn expression_source(
    expression: &UiBindingExpression,
    asset_id: &str,
    binding_id: &str,
) -> Result<String, UiAssetError> {
    match expression {
        UiBindingExpression::Literal(value) => {
            literal_source(value).ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: asset_id.to_string(),
                detail: format!(
                    "binding {binding_id} cannot encode resolved {:?} param in an expression",
                    value.kind()
                ),
            })
        }
        UiBindingExpression::ParamRef(name) => Ok(format!("param.{name}")),
        UiBindingExpression::PropRef(name) => Ok(format!("prop.{name}")),
        UiBindingExpression::ControlPropRef {
            control_id,
            property,
        } => Ok(format!("control.{control_id}.prop.{property}")),
        UiBindingExpression::Equals(lhs, rhs) => {
            binary_source(lhs, rhs, "==", asset_id, binding_id)
        }
        UiBindingExpression::NotEquals(lhs, rhs) => {
            binary_source(lhs, rhs, "!=", asset_id, binding_id)
        }
        UiBindingExpression::And(lhs, rhs) => binary_source(lhs, rhs, "&&", asset_id, binding_id),
        UiBindingExpression::Or(lhs, rhs) => binary_source(lhs, rhs, "||", asset_id, binding_id),
        UiBindingExpression::Not(value) => Ok(format!(
            "!({})",
            expression_source(value, asset_id, binding_id)?
        )),
    }
}

fn binary_source(
    lhs: &UiBindingExpression,
    rhs: &UiBindingExpression,
    operator: &str,
    asset_id: &str,
    binding_id: &str,
) -> Result<String, UiAssetError> {
    Ok(format!(
        "({} {operator} {})",
        expression_source(lhs, asset_id, binding_id)?,
        expression_source(rhs, asset_id, binding_id)?
    ))
}

fn literal_source(value: &UiValue) -> Option<String> {
    match value {
        UiValue::Bool(value) => Some(value.to_string()),
        UiValue::Int(value) => Some(value.to_string()),
        UiValue::Float(value) => float_source(*value),
        UiValue::String(value) => Some(string_source(value)),
        UiValue::Color(value) => Some(typed_string_source("color", value)),
        UiValue::AssetRef(value) => Some(typed_string_source("asset_ref", value)),
        UiValue::InstanceRef(value) => Some(typed_string_source("instance_ref", value)),
        UiValue::Enum(value) => Some(typed_string_source("enum", value)),
        UiValue::Vec2(values) => vector_source("vec2", values),
        UiValue::Vec3(values) => vector_source("vec3", values),
        UiValue::Vec4(values) => vector_source("vec4", values),
        UiValue::Flags(values) => Some(format!(
            "flags({})",
            values
                .iter()
                .map(|value| string_source(value))
                .collect::<Vec<_>>()
                .join(", ")
        )),
        UiValue::Null => Some("null".to_string()),
        UiValue::Array(_) | UiValue::Map(_) => None,
    }
}

fn payload_literal_value(
    value: &UiValue,
    asset_id: &str,
    binding_id: &str,
) -> Result<Value, UiAssetError> {
    match value {
        UiValue::Bool(_)
        | UiValue::Int(_)
        | UiValue::Float(_)
        | UiValue::String(_)
        | UiValue::Array(_)
        | UiValue::Map(_) => Ok(value.to_toml()),
        value => expression_source(
            &UiBindingExpression::Literal(value.clone()),
            asset_id,
            binding_id,
        )
        .map(|source| Value::String(format!("={source}"))),
    }
}

fn typed_string_source(constructor: &str, value: &str) -> String {
    format!("{constructor}({})", string_source(value))
}

fn vector_source<const N: usize>(constructor: &str, values: &[f64; N]) -> Option<String> {
    let values = values
        .iter()
        .map(|value| float_source(*value))
        .collect::<Option<Vec<_>>>()?;
    Some(format!("{constructor}({})", values.join(", ")))
}

fn string_source(value: &str) -> String {
    let mut source = String::with_capacity(value.len() + 2);
    source.push('"');
    for ch in value.chars() {
        match ch {
            '"' => source.push_str("\\\""),
            '\\' => source.push_str("\\\\"),
            '\n' => source.push_str("\\n"),
            '\r' => source.push_str("\\r"),
            '\t' => source.push_str("\\t"),
            '\u{0008}' => source.push_str("\\b"),
            '\u{000c}' => source.push_str("\\f"),
            ch if ch.is_control() => {
                write!(&mut source, "\\u{:04x}", ch as u32)
                    .expect("writing to a String cannot fail");
            }
            ch => source.push(ch),
        }
    }
    source.push('"');
    source
}

fn float_source(value: f64) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    let mut source = value.to_string();
    if source.contains('e') || source.contains('E') {
        return None;
    }
    if !source.contains('.') {
        source.push_str(".0");
    }
    Some(source)
}
