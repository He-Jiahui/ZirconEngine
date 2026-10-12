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
