use std::fmt;

use crate::core::asset::AssetTypeId;
use crate::core::editor_extension::EditorExtensionRegistryError;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::InspectorRegistrationError;

use super::super::ContributionBatch;
use super::{ContributionCounts, ContributionSource, ContributionTicket, PluginContributionId};

pub(super) fn validate_source_namespace(
    source: &ContributionSource,
    batch: &ContributionBatch,
) -> Result<(), ContributionError> {
    let ContributionSource::Plugin(plugin_id) = source else {
        if let Some(declaration) = batch.tool_resource_kinds().next() {
            return Err(ContributionError::ToolResourceKindRequiresPluginSource {
                kind: declaration.kind().as_str().to_owned(),
            });
        }
        if let Some((command_id, binding)) = batch.native_command_bindings().next() {
            return validate_native_binding_source(source, command_id, binding.plugin_id());
        }
        return Ok(());
    };
    let prefix = plugin_id.namespace_prefix();
    macro_rules! validate_keys {
        ($kind:literal, $keys:expr) => {
            for id in $keys {
                validate_plugin_key(plugin_id, &prefix, $kind, id)?;
            }
        };
    }
    validate_keys!("view", batch.views.keys().map(String::as_str));
    validate_keys!("drawer", batch.drawers.keys().map(String::as_str));
    validate_keys!(
        "menu operation",
        batch
            .menu_items
            .values()
            .map(|item| item.operation().as_str())
    );
    validate_keys!(
        "inspector customization",
        batch.inspector_customizations.keys().map(String::as_str)
    );
    validate_keys!(
        "field editor",
        batch.field_editors.keys().map(String::as_str)
    );
    validate_keys!("ui template", batch.ui_templates.keys().map(String::as_str));
    validate_keys!(
        "ui template pane data source",
        batch
            .ui_template_pane_data_sources
            .keys()
            .map(String::as_str)
    );
    validate_keys!(
        "asset importer",
        batch.asset_importers.keys().map(String::as_str)
    );
    validate_keys!(
        "asset type contribution",
        batch
            .asset_type_contributions
            .keys()
            .map(AssetTypeId::as_str)
    );
    for bundle_id in batch.localization_bundles.keys() {
        if bundle_id != plugin_id.as_str() {
            return Err(ContributionError::PluginLocalizationBundleOwner {
                plugin_id: plugin_id.clone(),
                bundle_id: bundle_id.clone(),
            });
        }
    }
    for (command_id, binding) in batch.native_command_bindings() {
        validate_native_binding_source(source, command_id, binding.plugin_id())?;
    }
    validate_keys!(
        "settings page",
        batch.settings_pages.keys().map(String::as_str)
    );
    validate_keys!("scene mode", batch.scene_modes.keys().map(String::as_str));
    validate_keys!(
        "viewport overlay provider",
        batch.viewport_overlay_providers.keys().map(String::as_str)
    );
    validate_keys!(
        "graph editor",
        batch.graph_editors.keys().map(AssetTypeId::as_str)
    );
    validate_keys!(
        "graph node palette",
        batch.graph_node_palettes.keys().map(String::as_str)
    );
    validate_keys!(
        "timeline editor",
        batch.timeline_editors.keys().map(AssetTypeId::as_str)
    );
    validate_keys!(
        "timeline track type",
        batch.timeline_track_types.keys().map(String::as_str)
    );
    validate_keys!(
        "tool resource kind",
        batch
            .tool_resource_kinds()
            .map(|declaration| declaration.kind().as_str())
    );
    validate_keys!(
        "command",
        batch.commands.keys().map(EditorOperationPath::as_str)
    );
    validate_keys!(
        "operation factory",
        batch
            .operation_factories
            .keys()
            .map(EditorOperationPath::as_str)
    );
    Ok(())
}

fn validate_plugin_key(
    plugin_id: &PluginContributionId,
    prefix: &str,
    kind: &'static str,
    id: &str,
) -> Result<(), ContributionError> {
    if id.starts_with(prefix) && id.len() > prefix.len() {
        return Ok(());
    }
    Err(ContributionError::PluginNamespace {
        plugin_id: plugin_id.clone(),
        kind,
        id: id.to_owned(),
    })
}

fn validate_native_binding_owner(
    plugin_id: &PluginContributionId,
    command_id: &EditorOperationPath,
    binding_plugin_id: &str,
) -> Result<(), ContributionError> {
    if binding_plugin_id == plugin_id.as_str() {
        return Ok(());
    }
    Err(ContributionError::NativeBindingOwner {
        plugin_id: plugin_id.clone(),
        command_id: command_id.clone(),
        binding_plugin_id: binding_plugin_id.to_owned(),
    })
}

fn validate_native_binding_source(
    source: &ContributionSource,
    command_id: &EditorOperationPath,
    binding_plugin_id: &str,
) -> Result<(), ContributionError> {
    match source {
        ContributionSource::Builtin => Err(ContributionError::NativeBindingRequiresPluginSource {
            command_id: command_id.clone(),
            binding_plugin_id: binding_plugin_id.to_owned(),
        }),
        ContributionSource::Plugin(plugin_id) => {
            validate_native_binding_owner(plugin_id, command_id, binding_plugin_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_binding_owner_mismatch_is_rejected_at_store_namespace_boundary() {
        let plugin_id = PluginContributionId::parse("sample").unwrap();
        let command_id = EditorOperationPath::parse("plugin.sample.command").unwrap();

        let error = validate_native_binding_owner(&plugin_id, &command_id, "other")
            .expect_err("a callback from another plugin must not enter the contribution store");
        assert!(matches!(
            error,
            ContributionError::NativeBindingOwner {
                plugin_id: owner,
                command_id: id,
                binding_plugin_id,
            } if owner == plugin_id && id == command_id && binding_plugin_id == "other"
        ));
        assert!(validate_native_binding_owner(&plugin_id, &command_id, "sample").is_ok());

        let builtin_error =
            validate_native_binding_source(&ContributionSource::Builtin, &command_id, "sample")
                .expect_err("native callbacks must be attached to a plugin contribution source");
        assert!(matches!(
            builtin_error,
            ContributionError::NativeBindingRequiresPluginSource {
                command_id: id,
                binding_plugin_id,
            } if id == command_id && binding_plugin_id == "sample"
        ));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevokeReport {
    pub(super) ticket: ContributionTicket,
    pub(super) source: Option<ContributionSource>,
    pub(super) generation: u64,
    pub(super) removed: ContributionCounts,
}

impl RevokeReport {
    pub fn ticket(&self) -> ContributionTicket {
        self.ticket
    }

    pub fn source(&self) -> Option<&ContributionSource> {
        self.source.as_ref()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn removed(&self) -> &ContributionCounts {
        &self.removed
    }

    pub fn revoked(&self) -> bool {
        self.source.is_some()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContributionError {
    InvalidPluginId(String),
    UnknownTicket(ContributionTicket),
    PluginNamespace {
        plugin_id: PluginContributionId,
        kind: &'static str,
        id: String,
    },
    PluginLocalizationBundleOwner {
        plugin_id: PluginContributionId,
        bundle_id: String,
    },
    NativeBindingOwner {
        plugin_id: PluginContributionId,
        command_id: EditorOperationPath,
        binding_plugin_id: String,
    },
    NativeBindingRequiresPluginSource {
        command_id: EditorOperationPath,
        binding_plugin_id: String,
    },
    ToolResourceKindRequiresPluginSource {
        kind: String,
    },
    DuplicateContribution {
        kind: &'static str,
        id: String,
    },
    FieldEditor(InspectorRegistrationError),
    Batch(EditorExtensionRegistryError),
}

impl From<InspectorRegistrationError> for ContributionError {
    fn from(error: InspectorRegistrationError) -> Self {
        Self::FieldEditor(error)
    }
}

impl From<EditorExtensionRegistryError> for ContributionError {
    fn from(error: EditorExtensionRegistryError) -> Self {
        Self::Batch(error)
    }
}

impl fmt::Display for ContributionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPluginId(id) => {
                write!(formatter, "plugin contribution id `{id}` is invalid")
            }
            Self::UnknownTicket(ticket) => {
                write!(formatter, "contribution ticket {} is not active", ticket.0)
            }
            Self::PluginNamespace {
                plugin_id,
                kind,
                id,
            } => write!(
                formatter,
                "plugin `{plugin_id}` {kind} `{id}` must use namespace `plugin.{plugin_id}.`"
            ),
            Self::PluginLocalizationBundleOwner {
                plugin_id,
                bundle_id,
            } => write!(
                formatter,
                "plugin `{plugin_id}` localization bundle `{bundle_id}` must use the plugin package id"
            ),
            Self::NativeBindingOwner {
                plugin_id,
                command_id,
                binding_plugin_id,
            } => write!(
                formatter,
                "plugin `{plugin_id}` native command `{command_id}` binds plugin `{binding_plugin_id}`; native bindings must use the contribution owner"
            ),
            Self::NativeBindingRequiresPluginSource {
                command_id,
                binding_plugin_id,
            } => write!(
                formatter,
                "builtin native command `{command_id}` binds plugin `{binding_plugin_id}`; native bindings require a plugin contribution source"
            ),
            Self::ToolResourceKindRequiresPluginSource { kind } => write!(
                formatter,
                "builtin contribution cannot register tool resource kind `{kind}`; extension tool resource kinds require a plugin contribution source"
            ),
            Self::DuplicateContribution { kind, id } => {
                write!(formatter, "editor {kind} `{id}` already contributed")
            }
            Self::FieldEditor(error) => error.fmt(formatter),
            Self::Batch(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ContributionError {}
