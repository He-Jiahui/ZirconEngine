use std::collections::BTreeMap;

use zircon_runtime_interface::editor_contribution::{
    SerializedToolResourceChannelPolicy, SerializedToolScopeKind,
};
use zircon_runtime_interface::{
    EditorCommandExecutionContract, EditorCommandResourceBudget, EditorCommandResultCodecId,
    SerializedContributionBatch, SerializedEditorContribution,
};

use super::materialize_serialized_contribution_batch;
use crate::core::commands::EditorCommandDescriptor;
use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{
    CapabilitySet, ContributionSource, ContributionStore, PluginContributionId,
    SettingsPageProjection,
};
use crate::core::i18n::{EditorI18nService, EditorLocale};
use crate::core::settings::SettingsPageDescriptor;

fn batch(contributions: Vec<SerializedEditorContribution>) -> SerializedContributionBatch {
    SerializedContributionBatch::new("fixture.editor", contributions)
        .expect("fixture contribution batch should be valid")
}

#[test]
fn materializes_every_supported_non_executable_contribution_kind() {
    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_command(EditorCommandDescriptor::operation(
            EditorOperationPath::parse("fixture.editor.command").unwrap(),
        ))
        .unwrap();
    let contributions = batch(vec![
        SerializedEditorContribution::View {
            id: "fixture.view".to_string(),
            schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
            title: "Fixture view".to_string(),
            category: "Tests".to_string(),
        },
        SerializedEditorContribution::Drawer {
            id: "fixture.drawer".to_string(),
            schema: SerializedEditorContribution::DRAWER_SCHEMA.to_string(),
            display_name: "Fixture drawer".to_string(),
        },
        SerializedEditorContribution::Menu {
            id: "fixture.menu.command".to_string(),
            schema: SerializedEditorContribution::MENU_SCHEMA.to_string(),
            command_id: "fixture.editor.command".to_string(),
            root_id: "tools".to_string(),
            root_label_key: "menu.tools.label".to_string(),
            group_ids: vec!["fixture".to_string()],
            group_label_keys: vec!["menu.tools.fixture.label".to_string()],
            leaf_label_key: "command.fixture.editor.command.label".to_string(),
        },
        SerializedEditorContribution::AssetType {
            id: "fixture.asset".to_string(),
            schema: SerializedEditorContribution::ASSET_TYPE_SCHEMA.to_string(),
            display_name: "Fixture asset".to_string(),
            badge: "Fixture".to_string(),
            icon_name: "puzzle-piece".to_string(),
            color_token: "editor.accent".to_string(),
            thumbnail_icon: "puzzle-piece".to_string(),
        },
        SerializedEditorContribution::SettingsPage {
            id: "plugin.fixture.editor.settings".to_string(),
            schema: SerializedEditorContribution::SETTINGS_PAGE_SCHEMA.to_string(),
            label_key: "plugin.fixture.label".to_string(),
            description_key: "plugin.fixture.description".to_string(),
            category_keys: vec![
                "plugin.fixture.category.plugins".to_string(),
                "plugin.fixture.category.fixture".to_string(),
            ],
        },
        SerializedEditorContribution::ToolResourceKind {
            id: "plugin.fixture.editor.viewport-lock".to_string(),
            schema: SerializedEditorContribution::TOOL_RESOURCE_KIND_SCHEMA.to_string(),
            supported_scopes: vec![SerializedToolScopeKind::Viewport],
            channel_policy: SerializedToolResourceChannelPolicy::Forbidden,
        },
        SerializedEditorContribution::LocalizationBundle {
            id: "fixture.editor".to_string(),
            schema: SerializedEditorContribution::LOCALIZATION_BUNDLE_SCHEMA.to_string(),
            locales: BTreeMap::from([
                (
                    "en".to_string(),
                    BTreeMap::from([
                        ("plugin.fixture.label".to_string(), "Fixture".to_string()),
                        (
                            "plugin.fixture.description".to_string(),
                            "Fixture settings".to_string(),
                        ),
                        (
                            "plugin.fixture.category.plugins".to_string(),
                            "Plugins".to_string(),
                        ),
                        (
                            "plugin.fixture.category.fixture".to_string(),
                            "Fixture".to_string(),
                        ),
                        ("menu.tools.label".to_string(), "Tools".to_string()),
                        (
                            "menu.tools.fixture.label".to_string(),
                            "Fixture".to_string(),
                        ),
                        (
                            "command.fixture.editor.command.label".to_string(),
                            "Fixture command".to_string(),
                        ),
                        (
                            "command.fixture.editor.command.description".to_string(),
                            "Run the fixture command".to_string(),
                        ),
                    ]),
                ),
                (
                    "zh-CN".to_string(),
                    BTreeMap::from([
                        ("plugin.fixture.label".to_string(), "示例".to_string()),
                        (
                            "plugin.fixture.description".to_string(),
                            "示例设置".to_string(),
                        ),
                        (
                            "plugin.fixture.category.plugins".to_string(),
                            "插件".to_string(),
                        ),
                        (
                            "plugin.fixture.category.fixture".to_string(),
                            "示例".to_string(),
                        ),
                        ("menu.tools.label".to_string(), "工具".to_string()),
                        ("menu.tools.fixture.label".to_string(), "示例".to_string()),
                        (
                            "command.fixture.editor.command.label".to_string(),
                            "示例命令".to_string(),
                        ),
                        (
                            "command.fixture.editor.command.description".to_string(),
                            "运行示例命令".to_string(),
                        ),
                    ]),
                ),
            ]),
        },
    ]);

    materialize_serialized_contribution_batch(&contributions, &mut registry)
        .expect("all supported fixture contributions should materialize");

    assert_eq!(registry.views().len(), 1);
    assert_eq!(registry.drawers().len(), 1);
    assert_eq!(registry.menu_items().len(), 1);
    let menu_item = &registry.menu_items()[0];
    assert_eq!(menu_item.path(), "tools/fixture/fixture.editor.command");
    assert_eq!(menu_item.menu_path().root().id().as_str(), "tools");
    assert_eq!(menu_item.menu_path().root().label_key(), "menu.tools.label");
    assert_eq!(menu_item.menu_path().groups()[0].id().as_str(), "fixture");
    assert_eq!(
        menu_item.menu_path().groups()[0].label_key(),
        "menu.tools.fixture.label"
    );
    assert_eq!(
        menu_item.menu_path().leaf().label_key(),
        "command.fixture.editor.command.label"
    );
    assert_eq!(registry.command_ids().count(), 1);
    assert_eq!(registry.asset_type_contributions().len(), 1);
    assert_eq!(registry.localization_bundles().len(), 1);
    assert_eq!(registry.settings_pages().len(), 1);
    assert_eq!(registry.tool_resource_kinds().len(), 1);
    assert_eq!(
        registry.settings_pages()[0],
        &SettingsPageDescriptor::new(
            "plugin.fixture.editor.settings",
            "fixture.editor",
            "plugin.fixture.label",
            "plugin.fixture.description",
            [
                "plugin.fixture.category.plugins",
                "plugin.fixture.category.fixture",
            ],
        )
        .unwrap(),
        "serialized and in-process page authoring must converge to one descriptor"
    );
}

#[test]
fn serialized_command_without_an_executor_rejects_the_batch_atomically() {
    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_view(ViewDescriptor::new("fixture.existing", "Existing", "Tests"))
        .unwrap();
    let contributions = batch(vec![
        SerializedEditorContribution::View {
            id: "fixture.candidate".to_string(),
            schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
            title: "Candidate".to_string(),
            category: "Tests".to_string(),
        },
        SerializedEditorContribution::Command {
            id: "fixture.editor.command".to_string(),
            schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
            localization_bundle_id: "fixture.editor".to_string(),
            label_key: "command.fixture.editor.command.label".to_string(),
            description_key: "command.fixture.editor.command.description".to_string(),
            execution_contract: Some(EditorCommandExecutionContract::new(
                EditorCommandResultCodecId::parse("zircon.editor.command-result.v1").unwrap(),
                EditorCommandResourceBudget::new(4096, 4096, 5000).unwrap(),
            )),
        },
    ]);

    let error = materialize_serialized_contribution_batch(&contributions, &mut registry)
        .expect_err("contract-bearing command without an executable route must fail closed");

    assert_eq!(
        error,
        super::SerializedContributionMaterializationError::MissingExecutor {
            id: "fixture.editor.command".to_string(),
        }
    );
    assert_eq!(registry.views().len(), 1);
    assert_eq!(registry.views()[0].id(), "fixture.existing");
    assert_eq!(registry.command_ids().count(), 0);
}

#[test]
fn native_binding_owner_must_match_serialized_package() {
    let error = super::validate_native_binding_owner(
        "fixture.editor.command",
        "fixture.editor",
        "other.editor",
    )
    .expect_err("cross-package native callback binding must fail closed");
    assert!(error
        .to_string()
        .contains("does not match serialized package"));
    assert!(super::validate_native_binding_owner(
        "fixture.editor.command",
        "fixture.editor",
        "fixture.editor",
    )
    .is_ok());
}

#[test]
fn failed_batch_does_not_publish_partial_contributions() {
    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_view(ViewDescriptor::new("fixture.existing", "Existing", "Tests"))
        .expect("existing view should register");
    let contributions = batch(vec![
        SerializedEditorContribution::View {
            id: "fixture.existing".to_string(),
            schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
            title: "Conflicting view".to_string(),
            category: "Tests".to_string(),
        },
        SerializedEditorContribution::Command {
            id: "fixture.editor.command".to_string(),
            schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
            localization_bundle_id: "fixture.editor".to_string(),
            label_key: "command.fixture.editor.command.label".to_string(),
            description_key: "command.fixture.editor.command.description".to_string(),
            execution_contract: None,
        },
    ]);

    let error = materialize_serialized_contribution_batch(&contributions, &mut registry)
        .expect_err("duplicate view should reject the candidate registry");

    assert!(error.to_string().contains("fixture.existing"));
    assert_eq!(registry.views().len(), 1);
    assert_eq!(registry.command_ids().count(), 0);
    assert!(registry.drawers().is_empty());
}

#[test]
fn settings_page_rejects_a_key_unknown_to_its_package_bundle_atomically() {
    let mut registry = EditorExtensionRegistry::default();
    let contributions = batch(vec![
        SerializedEditorContribution::LocalizationBundle {
            id: "fixture.editor".to_string(),
            schema: SerializedEditorContribution::LOCALIZATION_BUNDLE_SCHEMA.to_string(),
            locales: BTreeMap::from([(
                "en".to_string(),
                BTreeMap::from([("plugin.fixture.label".to_string(), "Fixture".to_string())]),
            )]),
        },
        SerializedEditorContribution::SettingsPage {
            id: "plugin.fixture.settings".to_string(),
            schema: SerializedEditorContribution::SETTINGS_PAGE_SCHEMA.to_string(),
            label_key: "plugin.fixture.label".to_string(),
            description_key: "plugin.fixture.unknown_description".to_string(),
            category_keys: vec!["plugin.fixture.unknown_category".to_string()],
        },
    ]);

    let error = materialize_serialized_contribution_batch(&contributions, &mut registry)
        .expect_err("unknown package localization key must reject the whole batch");

    assert!(error.to_string().contains("unknown_description"));
    assert!(registry.localization_bundles().is_empty());
    assert!(registry.settings_pages().is_empty());
}

#[test]
fn serialized_settings_page_projects_both_locales_and_revokes_with_its_bundle() {
    let contributions = batch(vec![
        SerializedEditorContribution::LocalizationBundle {
            id: "fixture.editor".to_string(),
            schema: SerializedEditorContribution::LOCALIZATION_BUNDLE_SCHEMA.to_string(),
            locales: BTreeMap::from([
                (
                    "en".to_string(),
                    BTreeMap::from([
                        ("plugin.fixture.label".to_string(), "Fixture".to_string()),
                        (
                            "plugin.fixture.description".to_string(),
                            "Fixture settings".to_string(),
                        ),
                        ("plugin.fixture.category".to_string(), "Plugins".to_string()),
                    ]),
                ),
                (
                    "zh-CN".to_string(),
                    BTreeMap::from([
                        ("plugin.fixture.label".to_string(), "示例".to_string()),
                        (
                            "plugin.fixture.description".to_string(),
                            "示例设置".to_string(),
                        ),
                        ("plugin.fixture.category".to_string(), "插件".to_string()),
                    ]),
                ),
            ]),
        },
        SerializedEditorContribution::SettingsPage {
            id: "plugin.fixture.editor.settings".to_string(),
            schema: SerializedEditorContribution::SETTINGS_PAGE_SCHEMA.to_string(),
            label_key: "plugin.fixture.label".to_string(),
            description_key: "plugin.fixture.description".to_string(),
            category_keys: vec!["plugin.fixture.category".to_string()],
        },
    ]);
    let mut registry = EditorExtensionRegistry::default();
    materialize_serialized_contribution_batch(&contributions, &mut registry).unwrap();
    let contribution_batch = registry.into_contribution_batch().unwrap();
    let mut store = ContributionStore::default();
    let ticket = store
        .contribute(
            ContributionSource::Plugin(PluginContributionId::parse("fixture.editor").unwrap()),
            contribution_batch,
        )
        .unwrap();
    let i18n = EditorI18nService::default();
    let capabilities = CapabilitySet::default();

    let english = SettingsPageProjection::capture(&store.snapshot(), &capabilities, &i18n);
    assert_eq!(english.pages()[0].label(), "Fixture");
    i18n.set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap();
    let chinese = SettingsPageProjection::capture(&store.snapshot(), &capabilities, &i18n);
    assert_eq!(chinese.pages()[0].label(), "示例");

    let report = store.revoke(ticket);
    assert_eq!(report.removed().localization_bundles(), 1);
    assert_eq!(report.removed().settings_pages(), 1);
    assert!(
        SettingsPageProjection::capture(&store.snapshot(), &capabilities, &i18n)
            .pages()
            .is_empty()
    );
}
