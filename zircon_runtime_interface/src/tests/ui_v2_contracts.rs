use std::collections::BTreeMap;

use toml::Value;

use crate::ui::template::UiComponentApiVersion;
use crate::ui::v2::{
    UiV2CompiledDocument, UiV2ComponentGraph, UiV2NodeArena, UiV2NodeHandle,
    UiV2StyleDeclarationBlock, UiV2StyleRule, UiV2StyleSheet,
};

#[test]
fn ui_v2_style_and_compiled_graph_dtos_construct() {
    let stylesheet = UiV2StyleSheet {
        id: "editor_material".to_string(),
        rules: vec![UiV2StyleRule {
            id: Some("primary_button".to_string()),
            selector: "Button.primary:hover".to_string(),
            set: UiV2StyleDeclarationBlock {
                self_values: BTreeMap::from([(
                    "fg".to_string(),
                    Value::String("$material.primary".to_string()),
                )]),
                slot: BTreeMap::new(),
            },
        }],
    };
    let compiled = UiV2CompiledDocument {
        asset_id: "asset://ui/tests/contract.v2.ui".to_string(),
        arena: UiV2NodeArena::default(),
        node_handles: BTreeMap::from([("root".to_string(), UiV2NodeHandle::new(0))]),
        component_graph: UiV2ComponentGraph::default(),
    };

    assert_eq!(stylesheet.rules[0].selector, "Button.primary:hover");
    assert_eq!(compiled.node_handles["root"].index(), 0);
}

#[test]
fn ui_component_api_version_mismatch_is_rejected_with_parse_error() {
    let exported = "1.2.0".parse::<UiComponentApiVersion>().unwrap();
    let required = "2.0.0".parse::<UiComponentApiVersion>().unwrap();

    assert!(!exported.is_compatible_with(required));

    let parse_error = "2.0".parse::<UiComponentApiVersion>().unwrap_err();
    assert_eq!(
        parse_error.to_string(),
        "invalid ui component api version 2.0"
    );
}
