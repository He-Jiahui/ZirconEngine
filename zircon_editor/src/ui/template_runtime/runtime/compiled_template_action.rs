use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::UiTemplateActionInvocation,
    template::{
        UiActionRef, UiBindingExpression, UiBindingMissingValuePolicy,
        UiBindingMissingValueResolution,
    },
};

#[derive(Clone, Debug)]
pub(super) enum CompiledTemplateAction {
    Action(String),
    Route {
        route: String,
        payload: Vec<CompiledTemplateActionPayloadField>,
        payload_missing_policy: UiBindingMissingValuePolicy,
    },
}

#[derive(Clone, Debug)]
pub(super) struct CompiledTemplateActionPayloadField {
    name: String,
    value: CompiledTemplateActionPayloadValue,
}

#[derive(Clone, Debug)]
enum CompiledTemplateActionPayloadValue {
    Literal(UiValue),
    Expression(UiBindingExpression),
    Unavailable,
}

impl CompiledTemplateAction {
    pub(super) fn compile(action: &UiActionRef) -> Option<Self> {
        let route = action
            .route
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty());
        let action_id = action
            .action
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty());
        match (route, action_id) {
            (None, Some(action_id)) if action.payload.is_empty() => {
                Some(Self::Action(action_id.to_string()))
            }
            (Some(route), None) => Some(Self::Route {
                route: route.to_string(),
                payload: action
                    .payload
                    .iter()
                    .map(|(name, value)| CompiledTemplateActionPayloadField {
                        name: name.clone(),
                        value: CompiledTemplateActionPayloadValue::compile(value),
                    })
                    .collect(),
                payload_missing_policy: action.payload_missing_policy.clone(),
            }),
            _ => None,
        }
    }

    pub(super) fn resolve(
        &self,
        source_attributes: &BTreeMap<String, Value>,
        attributes_by_control: &BTreeMap<String, BTreeMap<String, Value>>,
    ) -> Option<UiTemplateActionInvocation> {
        match self {
            Self::Action(action_id) => Some(UiTemplateActionInvocation::action(action_id.clone())),
            Self::Route {
                route,
                payload,
                payload_missing_policy,
            } => {
                let mut resolved_payload = BTreeMap::new();
                for field in payload {
                    match payload_missing_policy.resolve(
                        field
                            .value
                            .resolve(source_attributes, attributes_by_control),
                    ) {
                        UiBindingMissingValueResolution::Value(value) => {
                            resolved_payload.insert(field.name.clone(), value);
                        }
                        UiBindingMissingValueResolution::Omitted => {}
                        UiBindingMissingValueResolution::RequiredMissing
                        | UiBindingMissingValueResolution::ExplicitError => return None,
                    }
                }
                Some(UiTemplateActionInvocation::route(
                    route.clone(),
                    resolved_payload,
                ))
            }
        }
    }
}

impl CompiledTemplateActionPayloadValue {
    fn compile(value: &Value) -> Self {
        let Value::String(expression_text) = value else {
            return Self::Literal(UiValue::from_toml(value));
        };
        if !expression_text.trim_start().starts_with('=') {
            return Self::Literal(UiValue::String(expression_text.clone()));
        }
        UiBindingExpression::parse(expression_text)
            .map(Self::Expression)
            .unwrap_or(Self::Unavailable)
    }

    fn resolve(
        &self,
        source_attributes: &BTreeMap<String, Value>,
        attributes_by_control: &BTreeMap<String, BTreeMap<String, Value>>,
    ) -> Option<UiValue> {
        match self {
            Self::Literal(value) => Some(value.clone()),
            Self::Expression(expression) => expression
                .evaluate_with(
                    |_| None,
                    |property| source_attributes.get(property).map(UiValue::from_toml),
                    |control_id, property| {
                        attributes_by_control
                            .get(control_id)
                            .and_then(|attributes| attributes.get(property))
                            .map(UiValue::from_toml)
                    },
                )
                .ok(),
            Self::Unavailable => None,
        }
    }
}

#[cfg(test)]
#[path = "tests/compiled_template_action.rs"]
mod tests;
