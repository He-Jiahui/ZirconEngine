use super::*;

fn route_binding(route: &str) -> UiBindingRef {
    UiBindingRef {
        component_event: None,
        id: "Button/onClick".to_string(),
        event: UiEventKind::Click,
        mode: Default::default(),
        route: Some(route.to_string()),
        action: None,
        targets: Vec::new(),
    }
}

#[test]
fn dotted_form_route_uses_form_value_changed_payload_suggestions() {
    let suggestions = binding_root_payload_suggestions(&route_binding("route.form.value_changed"));
    assert_eq!(
        suggestions,
        vec![
            ("value".to_string(), Value::String("preview".to_string())),
            ("committed".to_string(), Value::Boolean(true)),
            (
                "fields".to_string(),
                Value::Array(vec![Value::String("title".to_string())])
            ),
            (
                "context".to_string(),
                Value::Table(
                    [
                        ("source".to_string(), Value::String("ui.click".to_string())),
                        ("subject".to_string(), Value::String("field".to_string())),
                    ]
                    .into_iter()
                    .collect()
                )
            ),
        ]
    );
}

#[test]
fn dotted_selection_route_uses_selection_payload_suggestions() {
    let suggestions = binding_root_payload_suggestions(&route_binding("route.selection.changed"));
    assert_eq!(
        suggestions,
        vec![
            (
                "primary".to_string(),
                Value::String("SelectedNode".to_string())
            ),
            (
                "selection_ids".to_string(),
                Value::Array(vec![Value::String("SelectedNode".to_string())])
            ),
            (
                "context".to_string(),
                Value::Table(
                    [
                        ("additive".to_string(), Value::Boolean(false)),
                        ("source".to_string(), Value::String("hierarchy".to_string())),
                    ]
                    .into_iter()
                    .collect()
                )
            ),
        ]
    );
}

#[test]
fn editor_normalization_uses_the_shared_binding_name_schema() {
    assert_eq!(
        normalized_binding_target(" workbench.asset.open ", UiBindingSchemaNameKind::Route),
        Some("workbench.asset.open".to_string())
    );
    assert_eq!(
        normalized_binding_target("workbench..open", UiBindingSchemaNameKind::Route),
        None
    );
    assert_eq!(
        normalized_binding_target("view/console", UiBindingSchemaNameKind::Action),
        None
    );
    assert_eq!(
        normalized_payload_key("context.source"),
        Some("context.source".to_string())
    );
    assert_eq!(normalized_payload_key("context.not valid"), None);
}
