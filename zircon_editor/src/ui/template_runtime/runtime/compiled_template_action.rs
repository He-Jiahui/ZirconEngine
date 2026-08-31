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
mod tests {
    use super::*;

    #[test]
    fn compiled_action_resolves_without_retaining_authoring_payload_text() {
        let mut action = UiActionRef {
            route: Some("plugin.operation".to_string()),
            action: None,
            payload: BTreeMap::from([(
                "entity".to_string(),
                Value::String("=control.RowList.prop.selected_row_identity".to_string()),
            )]),
            payload_missing_policy: Default::default(),
        };
        let compiled = CompiledTemplateAction::compile(&action).unwrap();
        action.payload.insert(
            "entity".to_string(),
            Value::String("tampered-after-compile".to_string()),
        );
        let controls = BTreeMap::from([(
            "RowList".to_string(),
            BTreeMap::from([("selected_row_identity".to_string(), Value::Integer(73))]),
        )]);

        assert_eq!(
            compiled.resolve(&BTreeMap::new(), &controls),
            Some(UiTemplateActionInvocation::route(
                "plugin.operation",
                BTreeMap::from([("entity".to_string(), UiValue::Int(73))]),
            ))
        );
    }

    #[test]
    fn preview_only_payload_expression_fails_closed_after_compilation() {
        let action = UiActionRef {
            route: Some("plugin.operation".to_string()),
            action: None,
            payload: BTreeMap::from([(
                "label".to_string(),
                Value::String("=concat(self.text, \"!\")".to_string()),
            )]),
            payload_missing_policy: Default::default(),
        };
        let compiled = CompiledTemplateAction::compile(&action).unwrap();

        assert!(compiled
            .resolve(&BTreeMap::new(), &BTreeMap::new())
            .is_none());
    }

    #[test]
    fn compiled_action_missing_value_policy_distinguishes_omit_substitute_and_reject() {
        let mut action = UiActionRef {
            route: Some("plugin.operation".to_string()),
            action: None,
            payload: BTreeMap::from([(
                "entity".to_string(),
                Value::String("=prop.missing".to_string()),
            )]),
            payload_missing_policy: UiBindingMissingValuePolicy::Optional,
        };
        let optional = CompiledTemplateAction::compile(&action).unwrap();
        assert!(optional
            .resolve(&BTreeMap::new(), &BTreeMap::new())
            .is_some_and(|invocation| invocation.payload.is_empty()));

        action.payload_missing_policy = UiBindingMissingValuePolicy::Default {
            value: UiValue::Int(73),
        };
        let defaulted = CompiledTemplateAction::compile(&action).unwrap();
        assert_eq!(
            defaulted
                .resolve(&BTreeMap::new(), &BTreeMap::new())
                .and_then(|invocation| invocation.payload.get("entity").cloned()),
            Some(UiValue::Int(73))
        );

        action.payload_missing_policy = UiBindingMissingValuePolicy::Error;
        let rejected = CompiledTemplateAction::compile(&action).unwrap();
        assert!(rejected
            .resolve(&BTreeMap::new(), &BTreeMap::new())
            .is_none());
    }
}
