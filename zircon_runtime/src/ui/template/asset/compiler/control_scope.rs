use std::collections::{BTreeMap, BTreeSet};

use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiAssetError, UiBindingExpression, UiBindingRef, UiTemplateNode,
};

use super::binding_param_resolver::{expression_source, parse_expression};

const COMPONENT_CONTROL_PREFIX: &str = "__zircon_component_instance_";

#[derive(Clone, Debug)]
pub(super) struct UiComponentControlScope {
    encoded_instance_path: String,
    root_local_control_id: Option<String>,
    root_alias: Option<String>,
}

impl UiComponentControlScope {
    pub(super) fn child(
        parent: Option<&Self>,
        instance_node_id: &str,
        root_local_control_id: Option<&str>,
        instance_control_id: Option<&str>,
    ) -> Self {
        let encoded_node_id = encode_identifier(instance_node_id);
        let encoded_instance_path = parent.map_or_else(
            || encoded_node_id.clone(),
            |parent| format!("{}_{encoded_node_id}", parent.encoded_instance_path),
        );
        Self {
            encoded_instance_path,
            root_local_control_id: root_local_control_id.map(str::to_string),
            root_alias: instance_control_id
                .map(|control_id| self::resolve_control_id(parent, control_id)),
        }
    }

    pub(super) fn resolve_control_id(&self, local_control_id: &str) -> String {
        if self.root_local_control_id.as_deref() == Some(local_control_id) {
            if let Some(alias) = &self.root_alias {
                return alias.clone();
            }
        }
        format!(
            "{COMPONENT_CONTROL_PREFIX}{}__{}",
            self.encoded_instance_path,
            encode_identifier(local_control_id)
        )
    }
}

pub(super) fn resolve_control_id(
    scope: Option<&UiComponentControlScope>,
    control_id: &str,
) -> String {
    scope.map_or_else(
        || control_id.to_string(),
        |scope| scope.resolve_control_id(control_id),
    )
}

pub(super) fn resolve_optional_control_id(
    scope: Option<&UiComponentControlScope>,
    control_id: Option<&str>,
) -> Option<String> {
    control_id.map(|control_id| resolve_control_id(scope, control_id))
}

pub(super) fn resolve_binding_control_scope(
    mut bindings: Vec<UiBindingRef>,
    scope: Option<&UiComponentControlScope>,
    asset_id: &str,
) -> Result<Vec<UiBindingRef>, UiAssetError> {
    let Some(scope) = scope else {
        return Ok(bindings);
    };
    for binding in &mut bindings {
        for assignment in &mut binding.targets {
            if !probe_control_reference(asset_id, &binding.id, &assignment.expression)? {
                continue;
            }
            let expression = parse_expression(asset_id, &binding.id, &assignment.expression)?;
            let expression = qualify_control_refs(expression, scope);
            assignment.expression = expression_source(&expression, asset_id, &binding.id)?;
        }

        let Some(action) = binding.action.as_mut() else {
            continue;
        };
        for value in action.payload.values_mut() {
            let Value::String(expression_text) = value else {
                continue;
            };
            if !expression_text.trim_start().starts_with('=')
                || !probe_control_reference(asset_id, &binding.id, expression_text)?
            {
                continue;
            }
            let expression = parse_expression(asset_id, &binding.id, expression_text)?;
            let expression = qualify_control_refs(expression, scope);
            *value = Value::String(format!(
                "={}",
                expression_source(&expression, asset_id, &binding.id)?
            ));
        }
    }
    Ok(bindings)
}

pub(super) fn retarget_expanded_root_control_id(
    root: &mut UiTemplateNode,
    control_id: String,
    asset_id: &str,
) -> Result<(), UiAssetError> {
    let previous_control_id = root.control_id.replace(control_id.clone());
    let Some(previous_control_id) = previous_control_id else {
        return Ok(());
    };
    if previous_control_id == control_id {
        return Ok(());
    }

    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        retarget_binding_control_id(
            &mut node.bindings,
            &previous_control_id,
            &control_id,
            asset_id,
        )?;
        stack.extend(node.children.iter_mut());
    }
    Ok(())
}

pub(super) fn validate_unique_control_ids(
    root: &UiTemplateNode,
    asset_id: &str,
) -> Result<(), UiAssetError> {
    let mut control_ids = BTreeSet::new();
    let mut duplicates = BTreeMap::<String, usize>::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(control_id) = node.control_id.as_deref() {
            if !control_ids.insert(control_id) {
                *duplicates.entry(control_id.to_string()).or_default() += 1;
            }
        }
        stack.extend(node.children.iter());
    }
    if duplicates.is_empty() {
        return Ok(());
    }
    Err(UiAssetError::InvalidDocument {
        asset_id: asset_id.to_string(),
        detail: format!(
            "compiled template contains duplicate control ids: {}",
            duplicates.keys().cloned().collect::<Vec<_>>().join(", ")
        ),
    })
}

fn probe_control_reference(
    asset_id: &str,
    binding_id: &str,
    expression: &str,
) -> Result<bool, UiAssetError> {
    UiBindingExpression::probe_control_reference(expression).map_err(|error| {
        UiAssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: format!("binding {binding_id} has invalid expression: {error}"),
        }
    })
}

fn qualify_control_refs(
    expression: UiBindingExpression,
    scope: &UiComponentControlScope,
) -> UiBindingExpression {
    match expression {
        UiBindingExpression::ControlPropRef {
            control_id,
            property,
        } => UiBindingExpression::ControlPropRef {
            control_id: scope.resolve_control_id(&control_id),
            property,
        },
        UiBindingExpression::Equals(lhs, rhs) => UiBindingExpression::Equals(
            Box::new(qualify_control_refs(*lhs, scope)),
            Box::new(qualify_control_refs(*rhs, scope)),
        ),
        UiBindingExpression::NotEquals(lhs, rhs) => UiBindingExpression::NotEquals(
            Box::new(qualify_control_refs(*lhs, scope)),
            Box::new(qualify_control_refs(*rhs, scope)),
        ),
        UiBindingExpression::And(lhs, rhs) => UiBindingExpression::And(
            Box::new(qualify_control_refs(*lhs, scope)),
            Box::new(qualify_control_refs(*rhs, scope)),
        ),
        UiBindingExpression::Or(lhs, rhs) => UiBindingExpression::Or(
            Box::new(qualify_control_refs(*lhs, scope)),
            Box::new(qualify_control_refs(*rhs, scope)),
        ),
        UiBindingExpression::Not(value) => {
            UiBindingExpression::Not(Box::new(qualify_control_refs(*value, scope)))
        }
        expression => expression,
    }
}

fn retarget_binding_control_id(
    bindings: &mut [UiBindingRef],
    previous_control_id: &str,
    control_id: &str,
    asset_id: &str,
) -> Result<(), UiAssetError> {
    for binding in bindings {
        for assignment in &mut binding.targets {
            if !probe_control_reference(asset_id, &binding.id, &assignment.expression)? {
                continue;
            }
            let expression = parse_expression(asset_id, &binding.id, &assignment.expression)?;
            let (expression, changed) =
                retarget_control_refs(expression, previous_control_id, control_id);
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
            if !expression_text.trim_start().starts_with('=')
                || !probe_control_reference(asset_id, &binding.id, expression_text)?
            {
                continue;
            }
            let expression = parse_expression(asset_id, &binding.id, expression_text)?;
            let (expression, changed) =
                retarget_control_refs(expression, previous_control_id, control_id);
            if changed {
                *value = Value::String(format!(
                    "={}",
                    expression_source(&expression, asset_id, &binding.id)?
                ));
            }
        }
    }
    Ok(())
}

fn retarget_control_refs(
    expression: UiBindingExpression,
    previous_control_id: &str,
    control_id: &str,
) -> (UiBindingExpression, bool) {
    match expression {
        UiBindingExpression::ControlPropRef {
            control_id: candidate,
            property,
        } if candidate == previous_control_id => (
            UiBindingExpression::ControlPropRef {
                control_id: control_id.to_string(),
                property,
            },
            true,
        ),
        UiBindingExpression::Equals(lhs, rhs) => {
            let (lhs, lhs_changed) = retarget_control_refs(*lhs, previous_control_id, control_id);
            let (rhs, rhs_changed) = retarget_control_refs(*rhs, previous_control_id, control_id);
            (
                UiBindingExpression::Equals(Box::new(lhs), Box::new(rhs)),
                lhs_changed || rhs_changed,
            )
        }
        UiBindingExpression::NotEquals(lhs, rhs) => {
            let (lhs, lhs_changed) = retarget_control_refs(*lhs, previous_control_id, control_id);
            let (rhs, rhs_changed) = retarget_control_refs(*rhs, previous_control_id, control_id);
            (
                UiBindingExpression::NotEquals(Box::new(lhs), Box::new(rhs)),
                lhs_changed || rhs_changed,
            )
        }
        UiBindingExpression::And(lhs, rhs) => {
            let (lhs, lhs_changed) = retarget_control_refs(*lhs, previous_control_id, control_id);
            let (rhs, rhs_changed) = retarget_control_refs(*rhs, previous_control_id, control_id);
            (
                UiBindingExpression::And(Box::new(lhs), Box::new(rhs)),
                lhs_changed || rhs_changed,
            )
        }
        UiBindingExpression::Or(lhs, rhs) => {
            let (lhs, lhs_changed) = retarget_control_refs(*lhs, previous_control_id, control_id);
            let (rhs, rhs_changed) = retarget_control_refs(*rhs, previous_control_id, control_id);
            (
                UiBindingExpression::Or(Box::new(lhs), Box::new(rhs)),
                lhs_changed || rhs_changed,
            )
        }
        UiBindingExpression::Not(value) => {
            let (value, changed) = retarget_control_refs(*value, previous_control_id, control_id);
            (UiBindingExpression::Not(Box::new(value)), changed)
        }
        expression => (expression, false),
    }
}

fn encode_identifier(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in value.as_bytes() {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}
