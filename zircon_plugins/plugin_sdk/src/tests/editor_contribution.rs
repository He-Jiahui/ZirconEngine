use std::collections::BTreeMap;

use zircon_runtime_interface::editor_contribution::{
    SerializedToolResourceChannelPolicy, SerializedToolScopeKind,
};
use zircon_runtime_interface::{
    EditorCommandResourceBudget, EditorCommandResultCodecId, SerializedEditorContribution,
};

use super::EditorContributionBuilder;

#[test]
fn build_returns_a_canonically_sorted_batch() {
    let batch = EditorContributionBuilder::new("plugin.sample")
        .view("sample.view", "Sample", "Sample")
        .command(
            "plugin.sample.command",
            "command.plugin.sample.command.label",
            "command.plugin.sample.command.description",
        )
        .build()
        .expect("distinct contributions should be accepted");

    assert_eq!(batch.package_id(), "plugin.sample");
    assert_eq!(
        batch.contributions()[0].key(),
        ("command", "plugin.sample.command")
    );
}

#[test]
fn build_reuses_the_shared_duplicate_rejection() {
    let result = EditorContributionBuilder::new("plugin.sample")
        .drawer("sample.drawer", "First")
        .drawer("sample.drawer", "Second")
        .build();

    assert!(result.is_err());
}

#[test]
fn build_reuses_the_shared_command_id_grammar() {
    let command = EditorContributionBuilder::new("plugin.sample")
        .command(
            "sample.command",
            "command.sample.command.label",
            "command.sample.command.description",
        )
        .build();
    let menu = EditorContributionBuilder::new("plugin.sample")
        .menu(
            "plugin.sample.menu",
            "sample.command",
            "tools",
            "menu.tools.label",
            std::iter::empty::<(&str, &str)>(),
            "command.sample.command.label",
        )
        .build();

    assert!(command.is_err());
    assert!(menu.is_err());
}

#[test]
fn command_builder_can_declare_a_versioned_execution_contract() {
    let contract = zircon_runtime_interface::EditorCommandExecutionContract::new(
        EditorCommandResultCodecId::parse("zircon.editor.command-result.v1").unwrap(),
        EditorCommandResourceBudget::new(4096, 8192, 250).unwrap(),
    );
    let batch = EditorContributionBuilder::new("plugin.sample")
        .command_with_execution_contract(
            "plugin.sample.command",
            "command.plugin.sample.command.label",
            "command.plugin.sample.command.description",
            contract,
        )
        .build()
        .expect("contract-bearing command should build");

    assert!(matches!(
        &batch.contributions()[0],
        SerializedEditorContribution::Command {
            schema,
            execution_contract: Some(_),
            ..
        } if schema == SerializedEditorContribution::COMMAND_SCHEMA
    ));
}

#[test]
fn settings_page_uses_its_package_bundle_and_locale_neutral_keys() {
    let batch = EditorContributionBuilder::new("fixture.editor")
        .localization_bundle(BTreeMap::from([(
            "en".to_string(),
            BTreeMap::from([
                ("plugin.fixture.label".to_string(), "Fixture".to_string()),
                (
                    "plugin.fixture.description".to_string(),
                    "Fixture settings".to_string(),
                ),
                ("plugin.category.plugins".to_string(), "Plugins".to_string()),
                ("plugin.category.fixture".to_string(), "Fixture".to_string()),
            ]),
        )]))
        .settings_page(
            "plugin.fixture.editor.settings",
            "plugin.fixture.label",
            "plugin.fixture.description",
            ["plugin.category.plugins", "plugin.category.fixture"],
        )
        .build()
        .expect("locale-neutral settings contributions should build");

    assert!(matches!(
        &batch.contributions()[0],
        SerializedEditorContribution::LocalizationBundle { id, .. }
            if id == "fixture.editor"
    ));
    assert!(matches!(
        &batch.contributions()[1],
        SerializedEditorContribution::SettingsPage { label_key, category_keys, .. }
            if label_key == "plugin.fixture.label"
                && category_keys == &["plugin.category.plugins", "plugin.category.fixture"]
    ));
}

#[test]
fn menu_uses_typed_segment_ids_and_locale_neutral_keys() {
    let batch = EditorContributionBuilder::new("fixture.editor")
        .menu(
            "fixture.menu.command",
            "fixture.editor.command",
            "tools",
            "menu.tools.label",
            [("fixture", "menu.tools.fixture.label")],
            "command.fixture.editor.command.label",
        )
        .build()
        .expect("typed menu contribution should build");

    assert!(matches!(
        &batch.contributions()[0],
        SerializedEditorContribution::Menu {
            schema,
            command_id,
            root_id,
            root_label_key,
            group_ids,
            group_label_keys,
            leaf_label_key,
            ..
        } if schema == SerializedEditorContribution::MENU_SCHEMA
            && command_id == "fixture.editor.command"
            && root_id == "tools"
            && root_label_key == "menu.tools.label"
            && group_ids == &["fixture"]
            && group_label_keys == &["menu.tools.fixture.label"]
            && leaf_label_key == "command.fixture.editor.command.label"
    ));
}

#[test]
fn tool_resource_kind_uses_the_shared_canonical_declaration() {
    let batch = EditorContributionBuilder::new("sample")
        .tool_resource_kind(
            "plugin.sample.viewport-lock",
            [
                SerializedToolScopeKind::Viewport,
                SerializedToolScopeKind::Window,
                SerializedToolScopeKind::Viewport,
            ],
            SerializedToolResourceChannelPolicy::Required,
        )
        .build()
        .expect("tool resource kind contribution should build");

    assert!(matches!(
        &batch.contributions()[0],
        SerializedEditorContribution::ToolResourceKind {
            schema,
            supported_scopes,
            channel_policy: SerializedToolResourceChannelPolicy::Required,
            ..
        } if schema == SerializedEditorContribution::TOOL_RESOURCE_KIND_SCHEMA
            && supported_scopes == &[
                SerializedToolScopeKind::Window,
                SerializedToolScopeKind::Viewport,
            ]
    ));
}
