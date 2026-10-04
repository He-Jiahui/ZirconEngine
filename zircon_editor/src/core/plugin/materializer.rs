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
#[path = "tests/materializer.rs"]
mod tests;
