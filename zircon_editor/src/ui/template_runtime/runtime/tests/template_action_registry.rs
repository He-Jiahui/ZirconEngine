use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiActionRef;

use super::*;

fn action() -> UiActionRef {
    UiActionRef {
        route: Some("plugin.operation".to_string()),
        action: None,
        payload: BTreeMap::new(),
        payload_missing_policy: Default::default(),
    }
}

fn selected_row_action() -> UiActionRef {
    UiActionRef {
        route: Some("plugin.operation".to_string()),
        action: None,
        payload: BTreeMap::from([(
            "entity".to_string(),
            Value::String("=control.RowList.prop.selected_row_identity".to_string()),
        )]),
        payload_missing_policy: Default::default(),
    }
}

fn invocation() -> UiTemplateActionInvocation {
    UiTemplateActionInvocation::route("plugin.operation", BTreeMap::new())
}

fn row(surface_entity: i64) -> Value {
    Value::Table(toml::map::Map::from_iter([(
        "surface_entity".to_string(),
        Value::Integer(surface_entity),
    )]))
}

#[test]
fn action_token_requires_the_current_plugin_document_generation() {
    let first_owner = EditorPluginV2DocumentOwner::new("navigation", 1)
        .expect("first owner generation should be valid");
    let replacement_owner = EditorPluginV2DocumentOwner::new("navigation", 2)
        .expect("replacement owner generation should be valid");
    let mut registry = TemplateActionRegistry::default();
    let token = registry.bind(
        "navigation.bake",
        "navigation.bake.panel",
        "BakeSelected/Click",
        Some(first_owner.clone()),
        BTreeMap::new(),
        action(),
        BTreeMap::new(),
    );

    assert_eq!(
        registry.action_for_token(&token, |_| Some(first_owner)),
        Some(invocation())
    );
    assert_eq!(
        registry.action_for_token(&token, |_| Some(replacement_owner)),
        None
    );
}

#[test]
fn replacing_a_pane_or_retiring_a_document_drops_stale_action_tokens() {
    let mut registry = TemplateActionRegistry::default();
    let stale_pane_token = registry.bind(
        "navigation.bake",
        "navigation.bake.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::new(),
        action(),
        BTreeMap::new(),
    );
    let retained_pane_token = registry.bind(
        "navigation.other",
        "navigation.other.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::new(),
        action(),
        BTreeMap::new(),
    );

    registry.remove_pane("navigation.bake");
    assert!(registry
        .action_for_token(&stale_pane_token, |_| None)
        .is_none());
    assert_eq!(
        registry.action_for_token(&retained_pane_token, |_| None),
        Some(invocation())
    );

    registry.remove_document("navigation.other.panel");
    assert!(registry
        .action_for_token(&retained_pane_token, |_| None)
        .is_none());
}

#[test]
fn recreating_a_removed_pane_never_reuses_its_action_token() {
    let mut registry = TemplateActionRegistry::default();
    let retired_token = registry.bind(
        "navigation.bake",
        "navigation.bake.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::new(),
        action(),
        BTreeMap::new(),
    );

    registry.remove_pane("navigation.bake");
    let replacement_token = registry.bind(
        "navigation.bake",
        "navigation.bake.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::new(),
        action(),
        BTreeMap::new(),
    );

    assert_ne!(retired_token, replacement_token);
    assert!(registry
        .action_for_token(&retired_token, |_| None)
        .is_none());
    assert_eq!(
        registry.action_for_token(&replacement_token, |_| None),
        Some(invocation())
    );
}

#[test]
fn recreating_a_pane_after_the_u64_epoch_limit_keeps_tokens_distinct() {
    let mut registry = TemplateActionRegistry::default();
    registry
        .pane_binding_epochs
        .insert("navigation.bake".to_string(), u64::MAX.into());
    let retired_token = registry.bind(
        "navigation.bake",
        "navigation.bake.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::new(),
        action(),
        BTreeMap::new(),
    );

    registry.remove_pane("navigation.bake");
    let replacement_token = registry.bind(
        "navigation.bake",
        "navigation.bake.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::new(),
        action(),
        BTreeMap::new(),
    );

    assert_ne!(retired_token, replacement_token);
    assert!(registry
        .action_for_token(&retired_token, |_| None)
        .is_none());
}

#[test]
fn control_state_refresh_re_resolves_the_current_pane_action_payload() {
    let mut registry = TemplateActionRegistry::default();
    let token = registry.bind(
        "plugin.rows",
        "plugin.rows.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::new(),
        selected_row_action(),
        BTreeMap::from([("RowList".to_string(), BTreeMap::new())]),
    );

    assert!(registry.action_for_token(&token, |_| None).is_none());
    assert!(registry.update_control_attributes_for_pane(
        "plugin.rows",
        "RowList",
        &BTreeMap::from([("selected_row_identity".to_string(), Value::Integer(11))]),
    ));
    assert_eq!(
        registry.action_for_token(&token, |_| None),
        Some(UiTemplateActionInvocation::route(
            "plugin.operation",
            BTreeMap::from([(
                "entity".to_string(),
                zircon_runtime_interface::ui::component::UiValue::Int(11),
            )]),
        ))
    );
    assert!(registry.update_control_attributes_for_pane(
        "plugin.rows",
        "RowList",
        &BTreeMap::from([("selected_row_identity".to_string(), Value::Integer(22))]),
    ));
    assert_eq!(
        registry.action_for_token(&token, |_| None),
        Some(UiTemplateActionInvocation::route(
            "plugin.operation",
            BTreeMap::from([(
                "entity".to_string(),
                zircon_runtime_interface::ui::component::UiValue::Int(22),
            )]),
        ))
    );
}

#[test]
fn disabled_action_source_does_not_resolve_an_invocation() {
    let mut registry = TemplateActionRegistry::default();
    let token = registry.bind(
        "plugin.rows",
        "plugin.rows.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::from([("disabled".to_string(), Value::Boolean(true))]),
        action(),
        BTreeMap::new(),
    );

    assert!(registry.action_for_token(&token, |_| None).is_none());
}

#[test]
fn explicitly_disabled_action_source_does_not_resolve_an_invocation() {
    let mut registry = TemplateActionRegistry::default();
    let token = registry.bind(
        "plugin.rows",
        "plugin.rows.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::from([("enabled".to_string(), Value::Boolean(false))]),
        action(),
        BTreeMap::new(),
    );

    assert!(registry.action_for_token(&token, |_| None).is_none());
}

#[test]
fn dynamic_disabled_state_blocks_an_already_bound_control_action() {
    let mut registry = TemplateActionRegistry::default();
    registry.rebind_pane("plugin.rows", "plugin.rows.panel", None, BTreeMap::new());
    let token = registry.bind_for_control(
        "plugin.rows",
        "plugin.rows.panel",
        "BakeSelected/Click",
        None,
        Some("BakeSelected"),
        BTreeMap::new(),
        action(),
        BTreeMap::new(),
    );
    assert!(registry.action_for_token(&token, |_| None).is_some());

    assert!(registry.update_control_attributes_for_pane(
        "plugin.rows",
        "BakeSelected",
        &BTreeMap::from([("disabled".to_string(), Value::Boolean(true))]),
    ));
    assert!(registry.action_for_token(&token, |_| None).is_none());
}

#[test]
fn table_row_selection_uses_the_current_snapshot_identity() {
    let mut registry = TemplateActionRegistry::default();
    let token = registry.bind(
        "plugin.rows",
        "plugin.rows.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::new(),
        selected_row_action(),
        BTreeMap::from([(
            "RowList".to_string(),
            BTreeMap::from([
                (
                    "row_identity_field".to_string(),
                    Value::String("surface_entity".to_string()),
                ),
                ("rows".to_string(), Value::Array(vec![row(41), row(73)])),
            ]),
        )]),
    );

    assert!(registry.action_for_token(&token, |_| None).is_none());
    assert!(registry.select_table_row("plugin.rows", "RowList", 0, "integer", "41",));
    assert_eq!(
        registry.action_for_token(&token, |_| None),
        Some(UiTemplateActionInvocation::route(
            "plugin.operation",
            BTreeMap::from([(
                "entity".to_string(),
                zircon_runtime_interface::ui::component::UiValue::Int(41),
            )]),
        ))
    );
    assert!(registry.select_table_row("plugin.rows", "RowList", 1, "integer", "73",));
    assert_eq!(
        registry.action_for_token(&token, |_| None),
        Some(UiTemplateActionInvocation::route(
            "plugin.operation",
            BTreeMap::from([(
                "entity".to_string(),
                zircon_runtime_interface::ui::component::UiValue::Int(73),
            )]),
        ))
    );
    assert!(!registry.select_table_row("plugin.rows", "RowList", 0, "integer", "73",));
}

#[test]
fn dynamic_row_updates_clear_a_stale_table_selection() {
    let mut registry = TemplateActionRegistry::default();
    let attributes = BTreeMap::from([(
        "RowList".to_string(),
        BTreeMap::from([
            (
                "row_identity_field".to_string(),
                Value::String("surface_entity".to_string()),
            ),
            ("rows".to_string(), Value::Array(vec![row(41), row(73)])),
        ]),
    )]);
    let token = registry.bind(
        "plugin.rows",
        "plugin.rows.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::new(),
        selected_row_action(),
        attributes,
    );

    assert!(registry.select_table_row("plugin.rows", "RowList", 1, "integer", "73"));
    assert!(registry.action_for_token(&token, |_| None).is_some());

    assert!(registry.update_control_attributes_for_pane(
        "plugin.rows",
        "RowList",
        &BTreeMap::from([("rows".to_string(), Value::Array(vec![row(41), row(99)]))]),
    ));
    assert!(registry.action_for_token(&token, |_| None).is_none());
}

#[test]
fn same_generation_rebind_preserves_only_a_current_table_selection() {
    let mut registry = TemplateActionRegistry::default();
    let attributes = BTreeMap::from([(
        "RowList".to_string(),
        BTreeMap::from([
            (
                "row_identity_field".to_string(),
                Value::String("surface_entity".to_string()),
            ),
            ("rows".to_string(), Value::Array(vec![row(41), row(73)])),
        ]),
    )]);

    registry.rebind_pane("plugin.rows", "plugin.rows.panel", None, attributes.clone());
    let first_token = registry.bind(
        "plugin.rows",
        "plugin.rows.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::new(),
        selected_row_action(),
        attributes.clone(),
    );
    assert!(registry.select_table_row("plugin.rows", "RowList", 1, "integer", "73"));
    assert!(registry.action_for_token(&first_token, |_| None).is_some());

    let rebound_attributes =
        registry.rebind_pane("plugin.rows", "plugin.rows.panel", None, attributes.clone());
    assert_eq!(
        rebound_attributes
            .get("RowList")
            .and_then(|attributes| attributes.get("selected_row_identity")),
        Some(&Value::Integer(73))
    );
    let rebound_token = registry.bind(
        "plugin.rows",
        "plugin.rows.panel",
        "BakeSelected/Click",
        None,
        BTreeMap::new(),
        selected_row_action(),
        attributes.clone(),
    );
    assert_eq!(
        registry.action_for_token(&rebound_token, |_| None),
        Some(UiTemplateActionInvocation::route(
            "plugin.operation",
            BTreeMap::from([(
                "entity".to_string(),
                zircon_runtime_interface::ui::component::UiValue::Int(73),
            )]),
        ))
    );
    assert_ne!(first_token, rebound_token);
    assert!(registry.action_for_token(&first_token, |_| None).is_none());

    let replacement_owner = EditorPluginV2DocumentOwner::new("plugin.rows", 2)
        .expect("replacement generation should be valid");
    registry.rebind_pane(
        "plugin.rows",
        "plugin.rows.panel",
        Some(&replacement_owner),
        attributes.clone(),
    );
    let replacement_token = registry.bind(
        "plugin.rows",
        "plugin.rows.panel",
        "BakeSelected/Click",
        Some(replacement_owner.clone()),
        BTreeMap::new(),
        selected_row_action(),
        attributes,
    );
    assert!(registry
        .action_for_token(&replacement_token, |_| Some(replacement_owner))
        .is_none());
}
