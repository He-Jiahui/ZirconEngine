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
            Self::DuplicateContribution { kind, id } => {
                write!(formatter, "editor {kind} `{id}` already contributed")
            }
            Self::FieldEditor(error) => error.fmt(formatter),
            Self::Batch(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ContributionError {}
