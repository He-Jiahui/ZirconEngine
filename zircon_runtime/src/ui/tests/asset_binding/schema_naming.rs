use std::collections::BTreeSet;

use zircon_runtime_interface::ui::{
    component::{UiComponentEventKind, UiValueKind},
    template::{UiActionPayloadFieldName, UiBindingContractTerm, UiBindingSchemaNameKind},
};

use super::*;

#[test]
fn component_event_schema_names_round_trip_from_the_interface_owner() {
    assert_eq!(UiComponentEventKind::ALL.len(), 35);

    let mut names = BTreeSet::new();
    for kind in UiComponentEventKind::ALL.iter().copied() {
        let name = kind.schema_name();
        assert!(names.insert(name), "duplicate component event name {name}");
        assert_eq!(UiComponentEventKind::from_schema_name(name), Some(kind));
    }
    assert_eq!(
        UiComponentEventKind::from_schema_name("value_changed"),
        None
    );
}

#[test]
fn action_payload_field_schema_owns_known_names_and_value_kinds() {
    assert_eq!(
        UiActionPayloadFieldName::from_schema_name("checked")
            .and_then(UiActionPayloadFieldName::expected_value_kind),
        Some(UiValueKind::Bool)
    );
    assert_eq!(
        UiActionPayloadFieldName::from_schema_name("surface_entity")
            .and_then(UiActionPayloadFieldName::expected_value_kind),
        Some(UiValueKind::Int)
    );
    assert_eq!(
        UiActionPayloadFieldName::from_schema_name("source")
            .and_then(UiActionPayloadFieldName::expected_value_kind),
        None
    );
    assert_eq!(
        UiActionPayloadFieldName::from_schema_name("product_specific"),
        None
    );
}

#[test]
fn binding_contract_terms_are_distinct_stable_and_drive_name_diagnostics() {
    assert_eq!(
        UiBindingContractTerm::ALL.map(UiBindingContractTerm::schema_name),
        ["event", "binding", "target", "route", "action", "command"]
    );
    for term in UiBindingContractTerm::ALL {
        assert!(!term.definition().is_empty(), "{term:?}");
    }
    assert_eq!(
        UiBindingSchemaNameKind::Route.contract_term(),
        Some(UiBindingContractTerm::Route)
    );
    assert_eq!(
        UiBindingSchemaNameKind::Action.contract_term(),
        Some(UiBindingContractTerm::Action)
    );
    assert_eq!(UiBindingSchemaNameKind::PayloadField.contract_term(), None);
    assert!(UiBindingSchemaNameKind::Route
        .validate("workbench..open")
        .unwrap_err()
        .to_string()
        .starts_with("route name"));
    assert!(UiBindingSchemaNameKind::Action
        .validate("view/console")
        .unwrap_err()
        .to_string()
        .starts_with("action name"));
    assert!(UiBindingSchemaNameKind::PayloadField
        .validate("not valid")
        .unwrap_err()
        .to_string()
        .starts_with("action payload field name"));
}

#[test]
fn binding_name_schema_preserves_product_routes_and_rejects_ambiguous_names() {
    for route in ["save", "workbench.asset.open", "RuntimeAction.TrackQuest"] {
        assert!(
            UiBindingSchemaNameKind::Route.validate(route).is_ok(),
            "{route}"
        );
    }
    for action in ["view.console.clear", "OpenPopupAudit"] {
        assert!(
            UiBindingSchemaNameKind::Action.validate(action).is_ok(),
            "{action}"
        );
    }
    for invalid in ["workbench..open", ".workbench.open", "workbench/open"] {
        assert!(
            UiBindingSchemaNameKind::Route.validate(invalid).is_err(),
            "{invalid}"
        );
    }
    assert!(UiBindingSchemaNameKind::PayloadField
        .validate("not valid")
        .is_err());
}

#[test]
fn compiler_rejects_invalid_route_action_and_payload_field_names() {
    for (case_index, (label, declaration)) in [
        ("route", "route = \"workbench..open\""),
        ("action", "action = { action = \"view/console\" }"),
        (
            "action payload field",
            "action = { route = \"workbench.open\", payload = { \"not valid\" = true } }",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let source = format!(
            r#"
[asset]
kind = "layout"
id = "test.binding.schema.case{case_index}"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "Root"
props = {{ text = "Open" }}

[[root.bindings]]
id = "Root/Open"
event = "Click"
{declaration}
"#
        );
        let document = UiAssetLoader::load_toml_str(&source).unwrap();
        let error = UiDocumentCompiler::default()
            .compile(&document)
            .expect_err("invalid binding schema names must fail before publication");

        assert!(error.to_string().contains(label), "{error}");
    }
}
