use std::collections::BTreeMap;

use super::{
    EditorCommandExecutionContract, SerializedContributionBatch, SerializedEditorContribution,
    SerializedToolResourceChannelPolicy, SerializedToolScopeKind,
};

#[test]
fn batch_sorts_contributions_and_rejects_duplicate_kind_id_pairs() {
    let batch = SerializedContributionBatch::new(
        "plugin.sample",
        vec![
            SerializedEditorContribution::View {
                id: "view.z".to_string(),
                schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
                title: "Z".to_string(),
                category: "Sample".to_string(),
            },
            SerializedEditorContribution::Command {
                id: "plugin.command.a".to_string(),
                schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
                localization_bundle_id: "plugin.sample".to_string(),
                label_key: "command.plugin.command.a.label".to_string(),
                description_key: "command.plugin.command.a.description".to_string(),
                execution_contract: None,
            },
        ],
    )
    .expect("distinct contributions should be accepted");
    assert_eq!(
        batch.contributions()[0].key(),
        ("command", "plugin.command.a")
    );

    let duplicate = SerializedContributionBatch::new(
        "plugin.sample",
        vec![
            SerializedEditorContribution::Drawer {
                id: "drawer".to_string(),
                schema: SerializedEditorContribution::DRAWER_SCHEMA.to_string(),
                display_name: "One".to_string(),
            },
            SerializedEditorContribution::View {
                id: "view.between".to_string(),
                schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
                title: "Between".to_string(),
                category: "Sample".to_string(),
            },
            SerializedEditorContribution::Drawer {
                id: "drawer".to_string(),
                schema: SerializedEditorContribution::DRAWER_SCHEMA.to_string(),
                display_name: "Two".to_string(),
            },
        ],
    );
    assert!(duplicate.is_err());
}

#[test]
fn batch_rejects_a_contribution_with_the_wrong_schema() {
    let batch = SerializedContributionBatch::new(
        "plugin.sample",
        vec![SerializedEditorContribution::SettingsPage {
            id: "settings.page".to_string(),
            schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
            label_key: "settings.sample.label".to_string(),
            description_key: "settings.sample.description".to_string(),
            category_keys: vec!["settings.category.plugins".to_string()],
        }],
    );

    assert!(batch.is_err());
}

#[test]
fn batch_rejects_command_ids_outside_the_shared_host_grammar() {
    let command = SerializedContributionBatch::new(
        "plugin.sample",
        vec![SerializedEditorContribution::Command {
            id: "sample.command".to_string(),
            schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
            localization_bundle_id: "plugin.sample".to_string(),
            label_key: "command.sample.command.label".to_string(),
            description_key: "command.sample.command.description".to_string(),
            execution_contract: None,
        }],
    );
    let menu = SerializedContributionBatch::new(
        "plugin.sample",
        vec![SerializedEditorContribution::Menu {
            id: "plugin.sample.menu".to_string(),
            schema: SerializedEditorContribution::MENU_SCHEMA.to_string(),
            command_id: "sample.command".to_string(),
            root_id: "tools".to_string(),
            root_label_key: "menu.tools.label".to_string(),
            group_ids: Vec::new(),
            group_label_keys: Vec::new(),
            leaf_label_key: "command.sample.command.label".to_string(),
        }],
    );

    assert!(command.is_err());
    assert!(menu.is_err());
}

#[test]
fn command_schema_v3_roundtrips_a_versioned_execution_contract() {
    let contract = EditorCommandExecutionContract::new(
        crate::EditorCommandResultCodecId::parse("zircon.editor.command-result.v1").unwrap(),
        crate::EditorCommandResourceBudget::new(4096, 8192, 250).unwrap(),
    );
    let command = SerializedEditorContribution::Command {
        id: "plugin.sample.command".to_string(),
        schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
        localization_bundle_id: "plugin.sample".to_string(),
        label_key: "command.plugin.sample.command.label".to_string(),
        description_key: "command.plugin.sample.command.description".to_string(),
        execution_contract: Some(contract),
    };
    let encoded = serde_json::to_string(&command).expect("command should serialize");
    let decoded: SerializedEditorContribution =
        serde_json::from_str(&encoded).expect("command should deserialize");
    assert_eq!(decoded, command);
    assert!(encoded.contains("zircon.editor.command/3"));
}

#[test]
fn settings_page_v1_literal_payload_is_rejected_by_the_hard_cut() {
    let payload = r#"{
            "kind":"settings_page",
            "id":"settings.page",
            "schema":"zircon.editor.settings-page/1",
            "display_name":"Settings",
            "category_path":"Plugin/Settings"
        }"#;

    assert!(serde_json::from_str::<SerializedEditorContribution>(payload).is_err());
}

#[test]
fn settings_page_v2_and_package_bundle_roundtrip_without_literal_fields() {
    let bundle = SerializedEditorContribution::LocalizationBundle {
        id: "plugin.sample".to_string(),
        schema: SerializedEditorContribution::LOCALIZATION_BUNDLE_SCHEMA.to_string(),
        locales: BTreeMap::from([(
            "en".to_string(),
            BTreeMap::from([(
                "plugin.sample.settings.label".to_string(),
                "Sample".to_string(),
            )]),
        )]),
    };
    let page = SerializedEditorContribution::SettingsPage {
        id: "plugin.sample.settings".to_string(),
        schema: SerializedEditorContribution::SETTINGS_PAGE_SCHEMA.to_string(),
        label_key: "plugin.sample.settings.label".to_string(),
        description_key: "plugin.sample.settings.description".to_string(),
        category_keys: vec!["plugin.sample.category.settings".to_string()],
    };
    let batch = SerializedContributionBatch::new("plugin.sample", vec![page.clone(), bundle])
        .expect("V2 settings page and bundle should be accepted");

    let json = serde_json::to_string(&batch).unwrap();
    let decoded = serde_json::from_str::<SerializedContributionBatch>(&json).unwrap();
    assert_eq!(decoded, batch);
    assert!(
        SerializedContributionBatch::new("plugin.sample", vec![page.clone(), page]).is_err(),
        "page identity must remain duplicate-checked independently of presentation"
    );
}

#[test]
fn tool_resource_kind_scopes_are_nonempty_canonical_and_roundtrip() {
    let resource = SerializedEditorContribution::ToolResourceKind {
        id: "plugin.sample.viewport-lock".to_string(),
        schema: SerializedEditorContribution::TOOL_RESOURCE_KIND_SCHEMA.to_string(),
        supported_scopes: vec![
            SerializedToolScopeKind::Viewport,
            SerializedToolScopeKind::Window,
            SerializedToolScopeKind::Viewport,
        ],
        channel_policy: SerializedToolResourceChannelPolicy::Optional,
    };
    let batch = SerializedContributionBatch::new("sample", vec![resource])
        .expect("tool resource declaration should canonicalize");
    assert!(matches!(
        &batch.contributions()[0],
        SerializedEditorContribution::ToolResourceKind {
            supported_scopes,
            ..
        } if supported_scopes == &[
            SerializedToolScopeKind::Window,
            SerializedToolScopeKind::Viewport,
        ]
    ));
    let json = serde_json::to_string(&batch).unwrap();
    assert_eq!(
        serde_json::from_str::<SerializedContributionBatch>(&json).unwrap(),
        batch
    );

    let empty = SerializedContributionBatch::new(
        "sample",
        vec![SerializedEditorContribution::ToolResourceKind {
            id: "plugin.sample.empty".to_string(),
            schema: SerializedEditorContribution::TOOL_RESOURCE_KIND_SCHEMA.to_string(),
            supported_scopes: Vec::new(),
            channel_policy: SerializedToolResourceChannelPolicy::Forbidden,
        }],
    );
    assert!(matches!(
        empty,
        Err(super::SerializedContributionBatchError::EmptyToolResourceScopes { .. })
    ));
}
