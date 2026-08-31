use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use crate::core::asset::AssetTypeId;
use crate::core::editor_operation::EditorOperationPath;

use super::super::ContributionBatch;
use super::lifecycle::ContributionError;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapabilitySet(BTreeSet<String>);

impl CapabilitySet {
    pub fn contains(&self, capability: &str) -> bool {
        self.0.contains(capability)
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }
}

impl<S, const N: usize> From<[S; N]> for CapabilitySet
where
    S: Into<String>,
{
    fn from(capabilities: [S; N]) -> Self {
        Self(capabilities.into_iter().map(Into::into).collect())
    }
}

impl<S> FromIterator<S> for CapabilitySet
where
    S: Into<String>,
{
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginContributionId(String);

impl PluginContributionId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ContributionError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.trim() == value
            && value.split('.').all(|segment| {
                !segment.is_empty()
                    && segment
                        .chars()
                        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
            });
        if !valid {
            return Err(ContributionError::InvalidPluginId(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(super) fn namespace_prefix(&self) -> String {
        format!("plugin.{}.", self.0)
    }
}

impl fmt::Display for PluginContributionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContributionSource {
    Builtin,
    Plugin(PluginContributionId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContributionTicket(pub(super) u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContributionChangeKind {
    Contributed,
    Replaced,
    Revoked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributionChange {
    pub(super) generation: u64,
    pub(super) ticket: ContributionTicket,
    pub(super) source: ContributionSource,
    pub(super) kind: ContributionChangeKind,
    pub(super) counts: ContributionCounts,
}

impl ContributionChange {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn ticket(&self) -> ContributionTicket {
        self.ticket
    }

    pub fn source(&self) -> &ContributionSource {
        &self.source
    }

    pub fn kind(&self) -> ContributionChangeKind {
        self.kind
    }

    pub fn counts(&self) -> &ContributionCounts {
        &self.counts
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributionDelta {
    pub(super) from_generation: u64,
    pub(super) to_generation: u64,
    pub(super) reset: bool,
    pub(super) changes: Arc<[ContributionChange]>,
}

impl ContributionDelta {
    pub fn from_generation(&self) -> u64 {
        self.from_generation
    }

    pub fn to_generation(&self) -> u64 {
        self.to_generation
    }

    /// The requested generation predates the retained journal; rebuild from the current snapshot.
    pub fn is_reset(&self) -> bool {
        self.reset
    }

    pub fn changes(&self) -> &[ContributionChange] {
        &self.changes
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContributionCounts {
    pub(super) views: usize,
    pub(super) drawers: usize,
    pub(super) menu_items: usize,
    pub(super) inspector_customizations: usize,
    pub(super) field_editors: usize,
    pub(super) ui_templates: usize,
    pub(super) ui_template_pane_data_sources: usize,
    pub(super) asset_importers: usize,
    pub(super) asset_type_contributions: usize,
    pub(super) localization_bundles: usize,
    pub(super) settings_pages: usize,
    pub(super) scene_modes: usize,
    pub(super) viewport_overlay_providers: usize,
    pub(super) graph_editors: usize,
    pub(super) graph_node_palettes: usize,
    pub(super) timeline_editors: usize,
    pub(super) timeline_track_types: usize,
    pub(super) commands: usize,
    pub(super) operation_factories: usize,
}

macro_rules! count_accessors {
    ($($method:ident => $field:ident),+ $(,)?) => {
        $(pub fn $method(&self) -> usize { self.$field })+
    };
}

impl ContributionCounts {
    count_accessors!(
        views => views,
        drawers => drawers,
        menu_items => menu_items,
        inspector_customizations => inspector_customizations,
        field_editors => field_editors,
        ui_templates => ui_templates,
        ui_template_pane_data_sources => ui_template_pane_data_sources,
        asset_importers => asset_importers,
        asset_type_contributions => asset_type_contributions,
        localization_bundles => localization_bundles,
        settings_pages => settings_pages,
        scene_modes => scene_modes,
        viewport_overlay_providers => viewport_overlay_providers,
        graph_editors => graph_editors,
        graph_node_palettes => graph_node_palettes,
        timeline_editors => timeline_editors,
        timeline_track_types => timeline_track_types,
        commands => commands,
        operation_factories => operation_factories,
    );
}

#[derive(Clone, Debug, Default)]
pub(super) struct ContributionKeys {
    pub(super) views: Vec<String>,
    pub(super) drawers: Vec<String>,
    pub(super) menu_items: Vec<String>,
    pub(super) inspector_customizations: Vec<String>,
    pub(super) field_editors: Vec<String>,
    pub(super) ui_templates: Vec<String>,
    pub(super) ui_template_pane_data_sources: Vec<String>,
    pub(super) asset_importers: Vec<String>,
    pub(super) asset_type_contributions: Vec<AssetTypeId>,
    pub(super) localization_bundles: Vec<String>,
    pub(super) settings_pages: Vec<String>,
    pub(super) scene_modes: Vec<String>,
    pub(super) viewport_overlay_providers: Vec<String>,
    pub(super) graph_editors: Vec<AssetTypeId>,
    pub(super) graph_node_palettes: Vec<String>,
    pub(super) timeline_editors: Vec<AssetTypeId>,
    pub(super) timeline_track_types: Vec<String>,
    pub(super) commands: Vec<EditorOperationPath>,
    pub(super) operation_factories: Vec<EditorOperationPath>,
}

impl ContributionKeys {
    pub(super) fn from_batch(batch: &ContributionBatch) -> Self {
        Self {
            views: batch.views.keys().cloned().collect(),
            drawers: batch.drawers.keys().cloned().collect(),
            menu_items: batch.menu_items.keys().cloned().collect(),
            inspector_customizations: batch.inspector_customizations.keys().cloned().collect(),
            field_editors: batch.field_editors.keys().cloned().collect(),
            ui_templates: batch.ui_templates.keys().cloned().collect(),
            ui_template_pane_data_sources: batch
                .ui_template_pane_data_sources
                .keys()
                .cloned()
                .collect(),
            asset_importers: batch.asset_importers.keys().cloned().collect(),
            asset_type_contributions: batch.asset_type_contributions.keys().cloned().collect(),
            localization_bundles: batch.localization_bundles.keys().cloned().collect(),
            settings_pages: batch.settings_pages.keys().cloned().collect(),
            scene_modes: batch.scene_modes.keys().cloned().collect(),
            viewport_overlay_providers: batch.viewport_overlay_providers.keys().cloned().collect(),
            graph_editors: batch.graph_editors.keys().cloned().collect(),
            graph_node_palettes: batch.graph_node_palettes.keys().cloned().collect(),
            timeline_editors: batch.timeline_editors.keys().cloned().collect(),
            timeline_track_types: batch.timeline_track_types.keys().cloned().collect(),
            commands: batch.commands.keys().cloned().collect(),
            operation_factories: batch.operation_factories.keys().cloned().collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct TicketRecord {
    pub(super) source: ContributionSource,
    pub(super) keys: ContributionKeys,
    pub(super) counts: ContributionCounts,
    pub(super) batch: ContributionBatch,
}

pub(crate) const CONTRIBUTION_CHANGE_JOURNAL_CAPACITY: usize = 4_096;
