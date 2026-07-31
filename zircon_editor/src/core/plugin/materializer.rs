//! Materialization of host-safe serialized editor contributions.

use std::fmt;

use zircon_runtime_interface::{SerializedContributionBatch, SerializedEditorContribution};

use crate::core::asset::{
    AssetTypeContribution, AssetTypeId, AssetTypePresentation, ThumbnailProviderDescriptor,
};
use crate::core::commands::EditorCommandDescriptor;
use crate::core::editor_extension::{
    DrawerDescriptor, EditorExtensionRegistry, EditorMenuItemDescriptor, ViewDescriptor,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::settings::SettingsPageDescriptor;

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
        materialize_contribution(contribution, &mut candidate)?;
    }
    *registry = candidate;
    Ok(())
}

fn materialize_contribution(
    contribution: &SerializedEditorContribution,
    registry: &mut EditorExtensionRegistry,
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
            path, command_id, ..
        } => {
            let operation = parse_operation("menu", path, command_id)?;
            registry
                .register_menu_item(EditorMenuItemDescriptor::new(path, operation))
                .map_err(|error| registry_error("menu", path, error))
        }
        SerializedEditorContribution::Command {
            id, display_name, ..
        } => {
            let operation = parse_operation("command", id, id)?;
            registry
                .register_command(EditorCommandDescriptor::operation(operation, display_name))
                .map_err(|error| registry_error("command", id, error))
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
        SerializedEditorContribution::SettingsPage {
            id,
            display_name,
            category_path,
            ..
        } => registry
            .register_settings_page(SettingsPageDescriptor::new(id, display_name, category_path))
            .map_err(|error| registry_error("settings page", id, error)),
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

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{SerializedContributionBatch, SerializedEditorContribution};

    use super::materialize_serialized_contribution_batch;
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};

    fn batch(contributions: Vec<SerializedEditorContribution>) -> SerializedContributionBatch {
        SerializedContributionBatch::new("fixture.editor", contributions)
            .expect("fixture contribution batch should be valid")
    }

    #[test]
    fn materializes_every_supported_contribution_kind() {
        let mut registry = EditorExtensionRegistry::default();
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
                path: "Tools/Fixture".to_string(),
                schema: SerializedEditorContribution::MENU_SCHEMA.to_string(),
                command_id: "fixture.command".to_string(),
            },
            SerializedEditorContribution::Command {
                id: "fixture.command".to_string(),
                schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
                display_name: "Fixture command".to_string(),
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
                id: "fixture.settings".to_string(),
                schema: SerializedEditorContribution::SETTINGS_PAGE_SCHEMA.to_string(),
                display_name: "Fixture settings".to_string(),
                category_path: "Plugins/Fixture".to_string(),
            },
        ]);

        materialize_serialized_contribution_batch(&contributions, &mut registry)
            .expect("all supported fixture contributions should materialize");

        assert_eq!(registry.views().len(), 1);
        assert_eq!(registry.drawers().len(), 1);
        assert_eq!(registry.menu_items().len(), 1);
        assert_eq!(registry.command_ids().count(), 1);
        assert_eq!(registry.asset_type_contributions().len(), 1);
        assert_eq!(registry.settings_pages().len(), 1);
    }

    #[test]
    fn failed_batch_does_not_publish_partial_contributions() {
        let mut registry = EditorExtensionRegistry::default();
        registry
            .register_view(ViewDescriptor::new("fixture.existing", "Existing", "Tests"))
            .expect("existing view should register");
        let contributions = batch(vec![
            SerializedEditorContribution::Command {
                id: "fixture.command".to_string(),
                schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
                display_name: "Fixture command".to_string(),
            },
            SerializedEditorContribution::View {
                id: "fixture.existing".to_string(),
                schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
                title: "Conflicting view".to_string(),
                category: "Tests".to_string(),
            },
        ]);

        let error = materialize_serialized_contribution_batch(&contributions, &mut registry)
            .expect_err("duplicate view should reject the candidate registry");

        assert!(error.to_string().contains("fixture.existing"));
        assert_eq!(registry.views().len(), 1);
        assert_eq!(registry.command_ids().count(), 0);
        assert!(registry.drawers().is_empty());
    }
}
