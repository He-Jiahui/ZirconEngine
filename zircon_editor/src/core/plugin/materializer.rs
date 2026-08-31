//! Materialization of host-safe serialized editor contributions.

use std::collections::BTreeMap;
use std::fmt;

use zircon_runtime::plugin::native::NativePluginEditorCommandBinding;
use zircon_runtime_interface::editor_contribution::{
    SerializedToolResourceChannelPolicy, SerializedToolScopeKind,
};
use zircon_runtime_interface::{SerializedContributionBatch, SerializedEditorContribution};

use crate::core::asset::{
    AssetTypeContribution, AssetTypeId, AssetTypePresentation, ThumbnailProviderDescriptor,
};
use crate::core::commands::{
    EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor, EditorCommandPresentation,
};
use crate::core::commands::{EditorCommandMenuPath, EditorCommandMenuSegment};
use crate::core::editor_extension::{
    DrawerDescriptor, EditorExtensionRegistry, EditorMenuItemDescriptor, ViewDescriptor,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::EditorLocalizationBundle;
use crate::core::settings::SettingsPageDescriptor;
use crate::core::tools::{
    ToolResourceChannelPolicy, ToolResourceKindDeclaration, ToolResourceKindId, ToolScopeKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SerializedContributionMaterializationError {
    InvalidOperation {
        kind: &'static str,
        id: String,
        detail: String,
    },
    InvalidAssetType {
        id: String,
        detail: String,
    },
    Registry {
        kind: &'static str,
        id: String,
        detail: String,
    },
    Unsupported {
        kind: &'static str,
        id: String,
    },
    MissingExecutor {
        id: String,
    },
}

impl fmt::Display for SerializedContributionMaterializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOperation { kind, id, detail } => {
                write!(
                    formatter,
                    "serialized editor {kind} `{id}` has invalid operation: {detail}"
                )
            }
            Self::InvalidAssetType { id, detail } => {
                write!(
                    formatter,
                    "serialized editor asset type `{id}` is invalid: {detail}"
                )
            }
            Self::Registry { kind, id, detail } => {
                write!(
                    formatter,
                    "serialized editor {kind} `{id}` cannot register: {detail}"
                )
            }
            Self::Unsupported { kind, id } => write!(
                formatter,
                "serialized editor {kind} `{id}` requires a host-safe descriptor that is not registered"
            ),
            Self::MissingExecutor { id } => write!(
                formatter,
                "serialized editor command `{id}` has no admitted executable route"
            ),
        }
    }
}

impl std::error::Error for SerializedContributionMaterializationError {}

/// Materializes all host-safe contributions atomically into an extension registry.
pub fn materialize_serialized_contribution_batch(
    batch: &SerializedContributionBatch,
    registry: &mut EditorExtensionRegistry,
) -> Result<(), SerializedContributionMaterializationError> {
    let mut candidate = registry.clone();
    for contribution in batch.contributions() {
        materialize_contribution(contribution, batch.package_id(), &mut candidate, None, None)?;
    }
    *registry = candidate;
    Ok(())
}

/// Materializes a native batch and atomically returns the callback bindings alongside its
/// descriptors. Serialized commands are accepted only when the loader has admitted the exact
/// command name through the native plugin behavior boundary.
pub(crate) fn materialize_serialized_native_contribution_batch(
    batch: &SerializedContributionBatch,
    registry: &mut EditorExtensionRegistry,
    bindings: &mut BTreeMap<EditorOperationPath, NativePluginEditorCommandBinding>,
    bind_command: impl Fn(&str) -> Result<NativePluginEditorCommandBinding, String>,
) -> Result<(), SerializedContributionMaterializationError> {
    let mut candidate = registry.clone();
    let mut candidate_bindings = bindings.clone();
    for contribution in batch.contributions() {
        materialize_contribution(
            contribution,
            batch.package_id(),
            &mut candidate,
            Some(&bind_command),
            Some(&mut candidate_bindings),
        )?;
    }
    *registry = candidate;
    *bindings = candidate_bindings;
    Ok(())
}

fn materialize_contribution(
    contribution: &SerializedEditorContribution,
    package_id: &str,
    registry: &mut EditorExtensionRegistry,
    bind_command: Option<&dyn Fn(&str) -> Result<NativePluginEditorCommandBinding, String>>,
    native_bindings: Option<&mut BTreeMap<EditorOperationPath, NativePluginEditorCommandBinding>>,
) -> Result<(), SerializedContributionMaterializationError> {
    match contribution {
        SerializedEditorContribution::View {
            id,
            title,
            category,
            ..
        } => registry
            .register_view(ViewDescriptor::new(id, title, category))
            .map_err(|error| registry_error("view", id, error)),
        SerializedEditorContribution::Drawer {
            id, display_name, ..
        } => registry
            .register_drawer(DrawerDescriptor::new(id, display_name))
            .map_err(|error| registry_error("drawer", id, error)),
        SerializedEditorContribution::Menu {
            id,
            command_id,
            root_id,
            root_label_key,
            group_ids,
            group_label_keys,
            leaf_label_key,
            ..
        } => {
            let operation = parse_operation("menu", id, command_id)?;
            if group_ids.len() != group_label_keys.len() {
                return Err(SerializedContributionMaterializationError::Registry {
                    kind: "menu",
                    id: id.clone(),
                    detail: "group ids and localization keys must have identical lengths"
                        .to_string(),
                });
            }
            let segment_error = |detail| SerializedContributionMaterializationError::Registry {
                kind: "menu",
                id: id.clone(),
                detail,
            };
            let root = EditorCommandMenuSegment::parse(root_id, root_label_key)
                .map_err(|detail| segment_error(detail))?;
            let groups = group_ids
                .iter()
                .zip(group_label_keys)
                .map(|(segment_id, label_key)| {
                    EditorCommandMenuSegment::parse(segment_id, label_key)
                        .map_err(|detail| segment_error(detail))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let leaf = EditorCommandMenuSegment::parse(command_id, leaf_label_key)
                .map_err(segment_error)?;
            let menu_path = EditorCommandMenuPath::new(root, groups, leaf);
            registry
                .register_menu_item(EditorMenuItemDescriptor::new(menu_path, operation))
                .map_err(|error| registry_error("menu", id, error))
        }
        SerializedEditorContribution::Command {
            id,
            localization_bundle_id,
            label_key,
            description_key,
            execution_contract,
            ..
        } => {
            let operation = parse_operation("command", id, id)?;
            let Some(contract) = execution_contract.clone() else {
                return Err(SerializedContributionMaterializationError::Registry {
                    kind: "command",
                    id: id.clone(),
                    detail: "native endpoint commands require an execution contract".to_owned(),
                });
            };
            let Some(bind_command) = bind_command else {
                return Err(
                    SerializedContributionMaterializationError::MissingExecutor { id: id.clone() },
                );
            };
            let binding = bind_command(id).map_err(|detail| {
                SerializedContributionMaterializationError::Registry {
                    kind: "command",
                    id: id.clone(),
                    detail,
                }
            })?;
            validate_native_binding_owner(id, package_id, binding.plugin_id())?;
            let presentation = EditorCommandPresentation::localized(
                localization_bundle_id,
                label_key,
                description_key,
            )
            .map_err(|detail| {
                SerializedContributionMaterializationError::InvalidOperation {
                    kind: "command",
                    id: id.clone(),
                    detail,
                }
            })?;
            registry
                .register_command(
                    EditorCommandDescriptor::localized(
                        operation.clone(),
                        presentation,
                        EditorCommandCategory::Command,
                        EditorCommandAction::NativeEndpoint,
                    )
                    .with_payload_schema_id(binding.payload_schema_id())
                    .with_execution_contract(contract),
                )
                .map_err(|error| registry_error("command", id, error))?;
            if let Some(native_bindings) = native_bindings {
                native_bindings.insert(operation, binding);
            }
            Ok(())
        }
        SerializedEditorContribution::AssetType {
            id,
            display_name,
            badge,
            icon_name,
            color_token,
            thumbnail_icon,
            ..
        } => {
            let asset_type = AssetTypeId::parse(id).map_err(|error| {
                SerializedContributionMaterializationError::InvalidAssetType {
                    id: id.clone(),
                    detail: error.to_string(),
                }
            })?;
            let contribution = AssetTypeContribution::define(
                asset_type,
                AssetTypePresentation::new(display_name, badge, icon_name, color_token),
                ThumbnailProviderDescriptor::Icon(thumbnail_icon.clone()),
            );
            registry
                .register_asset_type_contribution(contribution)
                .map_err(|error| registry_error("asset type", id, error))
        }
        SerializedEditorContribution::LocalizationBundle { id, locales, .. } => {
            if id != package_id {
                return Err(SerializedContributionMaterializationError::Registry {
                    kind: "localization bundle",
                    id: id.clone(),
                    detail: format!(
                        "bundle owner must match serialized contribution package `{package_id}`"
                    ),
                });
            }
            let bundle = EditorLocalizationBundle::from_locale_maps(id, locales.clone()).map_err(
                |detail| SerializedContributionMaterializationError::Registry {
                    kind: "localization bundle",
                    id: id.clone(),
                    detail,
                },
            )?;
            registry
                .register_localization_bundle(bundle)
                .map_err(|error| registry_error("localization bundle", id, error))
        }
        SerializedEditorContribution::SettingsPage {
            id,
            label_key,
            description_key,
            category_keys,
            ..
        } => {
            let descriptor = SettingsPageDescriptor::new(
                id,
                package_id,
                label_key,
                description_key,
                category_keys.iter().cloned(),
            )
            .map_err(|detail| {
                SerializedContributionMaterializationError::Registry {
                    kind: "settings page",
                    id: id.clone(),
                    detail,
                }
            })?;
            registry
                .register_settings_page(descriptor)
                .map_err(|error| registry_error("settings page", id, error))
        }
        SerializedEditorContribution::ToolResourceKind {
            id,
            supported_scopes,
            channel_policy,
            ..
        } => {
            let kind = ToolResourceKindId::parse(id).map_err(|error| {
                SerializedContributionMaterializationError::Registry {
                    kind: "tool resource kind",
                    id: id.clone(),
                    detail: error.to_string(),
                }
            })?;
            let supported_scopes = supported_scopes.iter().copied().map(|scope| match scope {
                SerializedToolScopeKind::Editor => ToolScopeKind::Editor,
                SerializedToolScopeKind::Project => ToolScopeKind::Project,
                SerializedToolScopeKind::Document => ToolScopeKind::Document,
                SerializedToolScopeKind::Window => ToolScopeKind::Window,
                SerializedToolScopeKind::Viewport => ToolScopeKind::Viewport,
            });
            let channel_policy = match channel_policy {
                SerializedToolResourceChannelPolicy::Forbidden => {
                    ToolResourceChannelPolicy::Forbidden
                }
                SerializedToolResourceChannelPolicy::Optional => {
                    ToolResourceChannelPolicy::Optional
                }
                SerializedToolResourceChannelPolicy::Required => {
                    ToolResourceChannelPolicy::Required
                }
            };
            let declaration =
                ToolResourceKindDeclaration::new(kind, supported_scopes, channel_policy)
                    .map_err(|error| registry_error("tool resource kind", id, error))?;
            registry
                .register_tool_resource_kind(declaration)
                .map_err(|error| registry_error("tool resource kind", id, error))
        }
    }
}

fn parse_operation(
    kind: &'static str,
    id: &str,
    operation: &str,
) -> Result<EditorOperationPath, SerializedContributionMaterializationError> {
    EditorOperationPath::parse(operation).map_err(|error| {
        SerializedContributionMaterializationError::InvalidOperation {
            kind,
            id: id.to_string(),
            detail: error.to_string(),
        }
    })
}

fn registry_error(
    kind: &'static str,
    id: &str,
    error: impl fmt::Display,
) -> SerializedContributionMaterializationError {
    SerializedContributionMaterializationError::Registry {
        kind,
        id: id.to_string(),
        detail: error.to_string(),
    }
}

fn validate_native_binding_owner(
    command_id: &str,
    package_id: &str,
    binding_plugin_id: &str,
) -> Result<(), SerializedContributionMaterializationError> {
    if binding_plugin_id == package_id {
        return Ok(());
    }
    Err(SerializedContributionMaterializationError::Registry {
        kind: "command",
        id: command_id.to_owned(),
        detail: format!(
            "native binding owner `{binding_plugin_id}` does not match serialized package `{package_id}`"
        ),
    })
}

#[cfg(test)]
mod tests {
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
        assert!(
            error
                .to_string()
                .contains("does not match serialized package")
        );
        assert!(
            super::validate_native_binding_owner(
                "fixture.editor.command",
                "fixture.editor",
                "fixture.editor",
            )
            .is_ok()
        );
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
}
