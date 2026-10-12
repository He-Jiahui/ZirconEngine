use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::UiTemplateActionInvocation,
    template::{UiActionRef, UiBindingMissingValuePolicy},
};

use super::resolve_template_action;

#[test]
fn resolves_typed_action_payload_from_a_control_property_snapshot() {
    let action = UiActionRef {
        route: Some("plugin.operation".to_string()),
        action: None,
        payload: BTreeMap::from([(
            "entity".to_string(),
            Value::String("=control.RowList.prop.selected_row_identity".to_string()),
        )]),
        payload_missing_policy: Default::default(),
    };
    let control_attributes = BTreeMap::from([(
        "RowList".to_string(),
        BTreeMap::from([("selected_row_identity".to_string(), Value::Integer(73))]),
    )]);

    assert_eq!(
        resolve_template_action(&action, &BTreeMap::new(), &control_attributes),
        Some(UiTemplateActionInvocation::route(
            "plugin.operation",
            BTreeMap::from([("entity".to_string(), UiValue::Int(73))]),
        ))
    );
}

#[test]
fn authored_editor_action_keeps_action_identity_without_a_route_alias() {
    let action = UiActionRef {
        route: None,
        action: Some("view.console.clear".to_string()),
        payload: BTreeMap::new(),
        payload_missing_policy: Default::default(),
    };

    assert_eq!(
        resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()),
        Some(UiTemplateActionInvocation::action("view.console.clear"))
    );
}

#[test]
fn authored_action_and_route_aliases_are_rejected_as_ambiguous() {
    let action = UiActionRef {
        route: Some("view.console.clear".to_string()),
        action: Some("view.console.clear".to_string()),
        payload: BTreeMap::new(),
        payload_missing_policy: Default::default(),
    };

    assert_eq!(
        resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()),
        None
    );
}

#[test]
fn authored_editor_action_with_route_payload_is_rejected() {
    let action = UiActionRef {
        route: None,
        action: Some("view.console.clear".to_string()),
        payload: BTreeMap::from([(
            "legacy_route_argument".to_string(),
            toml::Value::Boolean(true),
        )]),
        payload_missing_policy: Default::default(),
    };

    assert_eq!(
        resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()),
        None
    );
}

#[test]
fn source_action_missing_value_policy_distinguishes_omit_substitute_and_reject() {
    let mut action = UiActionRef {
        route: Some("plugin.operation".to_string()),
        action: None,
        payload: BTreeMap::from([(
            "entity".to_string(),
            Value::String("=prop.missing".to_string()),
        )]),
        payload_missing_policy: UiBindingMissingValuePolicy::Optional,
    };

    let optional = resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new())
        .expect("optional missing payload should preserve its route");
    assert!(optional.payload.is_empty());

    action.payload_missing_policy = UiBindingMissingValuePolicy::Fallback {
        value: UiValue::Int(73),
    };
    assert_eq!(
        resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new())
            .and_then(|invocation| invocation.payload.get("entity").cloned()),
        Some(UiValue::Int(73))
    );

    action.payload_missing_policy = UiBindingMissingValuePolicy::Error;
    assert!(resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()).is_none());
}

#[test]
fn console_editor_commands_are_authored_as_actions_not_routes() {
    let source = include_str!("../../../../../assets/ui/editor/host/console_body.zui");

    for command_id in [
        "view.console.filter.all",
        "view.console.filter.error",
        "view.console.filter.warning",
        "view.console.filter.info",
        "view.console.source.all",
        "view.console.source.editor",
        "view.console.source.runtime",
        "view.console.source.play",
        "view.console.source.plugin",
        "view.console.source.import",
        "view.console.source.script_build",
        "view.console.clear",
    ] {
        assert!(
            source.contains(&format!("action = {{ action = \"{command_id}\" }}")),
            "{command_id} must use UiActionRef.action"
        );
        assert!(
            !source.contains(&format!("route = \"{command_id}\"")),
            "{command_id} must not retain a route alias"
        );
    }
}

#[test]
fn host_projection_indexes_bindings_by_reference() {
    let source = include_str!("../projection.rs");
    let builders = source
        .split("pub(super) fn build_host_model")
        .nth(1)
        .expect("host model builders")
        .split("fn merge_projection_only_host_nodes")
        .next()
        .expect("host model builder bodies");
    let cloned_rows = [".cloned", "()"].concat();

    assert!(!builders.contains(&cloned_rows));
}
