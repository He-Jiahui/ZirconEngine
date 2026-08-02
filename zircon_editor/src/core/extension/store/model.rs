use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::sync::Arc;

use crate::core::asset::AssetTypeId;
use crate::core::editor_extension::{EditorUiTemplateDescriptor, EditorUiTemplatePaneDataSource};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::FieldEditorContainer;

use super::ContributionBatch;

mod lifecycle;
mod snapshot;

use lifecycle::validate_source_namespace;
pub use lifecycle::{ContributionError, RevokeReport};
pub use snapshot::ContributionSnapshot;
use snapshot::{IndexedContribution, IndexedMap};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapabilitySet(BTreeSet<String>);

impl CapabilitySet {
    pub fn contains(&self, capability: &str) -> bool {
        self.0.contains(capability)
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

    fn namespace_prefix(&self) -> String {
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
pub struct ContributionTicket(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContributionChangeKind {
    Contributed,
    Replaced,
    Revoked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributionChange {
    generation: u64,
    ticket: ContributionTicket,
    source: ContributionSource,
    kind: ContributionChangeKind,
    counts: ContributionCounts,
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
    from_generation: u64,
    to_generation: u64,
    reset: bool,
    changes: Arc<[ContributionChange]>,
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
    views: usize,
    drawers: usize,
    menu_items: usize,
    inspector_customizations: usize,
    field_editors: usize,
    ui_templates: usize,
    ui_template_pane_data_sources: usize,
    asset_importers: usize,
    asset_type_contributions: usize,
    settings_pages: usize,
    scene_modes: usize,
    viewport_overlay_providers: usize,
    graph_editors: usize,
    graph_node_palettes: usize,
    timeline_editors: usize,
    timeline_track_types: usize,
    commands: usize,
    operation_factories: usize,
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
struct ContributionKeys {
    views: Vec<String>,
    drawers: Vec<String>,
    menu_items: Vec<String>,
    inspector_customizations: Vec<String>,
    field_editors: Vec<String>,
    ui_templates: Vec<String>,
    ui_template_pane_data_sources: Vec<String>,
    asset_importers: Vec<String>,
    asset_type_contributions: Vec<AssetTypeId>,
    settings_pages: Vec<String>,
    scene_modes: Vec<String>,
    viewport_overlay_providers: Vec<String>,
    graph_editors: Vec<AssetTypeId>,
    graph_node_palettes: Vec<String>,
    timeline_editors: Vec<AssetTypeId>,
    timeline_track_types: Vec<String>,
    commands: Vec<EditorOperationPath>,
    operation_factories: Vec<EditorOperationPath>,
}

impl ContributionKeys {
    fn from_batch(batch: &ContributionBatch) -> Self {
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
struct TicketRecord {
    source: ContributionSource,
    keys: ContributionKeys,
    counts: ContributionCounts,
    batch: ContributionBatch,
}

pub(crate) const CONTRIBUTION_CHANGE_JOURNAL_CAPACITY: usize = 4_096;

pub struct ContributionStore {
    generation: u64,
    next_ticket: u64,
    current: Arc<ContributionSnapshot>,
    tickets: BTreeMap<ContributionTicket, TicketRecord>,
    changes: VecDeque<ContributionChange>,
}

impl Default for ContributionStore {
    fn default() -> Self {
        Self {
            generation: 0,
            next_ticket: 1,
            current: Arc::new(ContributionSnapshot::default()),
            tickets: BTreeMap::new(),
            changes: VecDeque::new(),
        }
    }
}

macro_rules! publish_entries {
    ($candidate:ident, $field:ident, $values:expr, $ticket:ident, $source:ident, $caps:ident, $counts:ident, $count:ident, $kind:literal) => {{
        let values = $values;
        if !values.is_empty() {
            let target = Arc::make_mut(&mut $candidate.$field);
            for (id, value) in values {
                if target.contains_key(&id) {
                    return Err(ContributionError::DuplicateContribution {
                        kind: $kind,
                        id: id.to_string(),
                    });
                }
                target.insert(
                    id,
                    IndexedContribution::new($ticket, &$source, &$caps, value),
                );
                $counts.$count += 1;
            }
        }
    }};
}

impl ContributionStore {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn snapshot(&self) -> Arc<ContributionSnapshot> {
        Arc::clone(&self.current)
    }

    pub fn len(&self) -> usize {
        self.tickets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tickets.is_empty()
    }

    /// Returns the immutable source batch for an active ticket.
    ///
    /// Host materializers inspect this before revocation to tear down all runtime side effects
    /// from the Store-owned contribution set rather than keeping an independent owner cache.
    pub(crate) fn batch_for_ticket(
        &self,
        ticket: ContributionTicket,
    ) -> Option<&ContributionBatch> {
        self.tickets.get(&ticket).map(|record| &record.batch)
    }

    pub fn validate_contribution(
        &self,
        source: &ContributionSource,
        batch: &ContributionBatch,
    ) -> Result<(), ContributionError> {
        validate_source_namespace(source, batch)?;
        FieldEditorContainer::with_contributions(batch.field_editors.values().cloned())?;
        macro_rules! reject_collisions {
            ($field:ident, $values:expr, $kind:literal) => {
                if let Some(id) = $values
                    .keys()
                    .find(|id| self.current.$field.contains_key(*id))
                {
                    return Err(ContributionError::DuplicateContribution {
                        kind: $kind,
                        id: id.to_string(),
                    });
                }
            };
        }
        reject_collisions!(views, batch.views, "view");
        reject_collisions!(drawers, batch.drawers, "drawer");
        reject_collisions!(menu_items, batch.menu_items, "menu item");
        reject_collisions!(
            inspector_customizations,
            batch.inspector_customizations,
            "inspector customization"
        );
        reject_collisions!(field_editors, batch.field_editors, "field editor");
        reject_collisions!(ui_templates, batch.ui_templates, "ui template");
        reject_collisions!(
            ui_template_pane_data_sources,
            batch.ui_template_pane_data_sources,
            "ui template pane data source"
        );
        reject_collisions!(asset_importers, batch.asset_importers, "asset importer");
        reject_collisions!(
            asset_type_contributions,
            batch.asset_type_contributions,
            "asset type contribution"
        );
        reject_collisions!(settings_pages, batch.settings_pages, "settings page");
        reject_collisions!(scene_modes, batch.scene_modes, "scene mode");
        reject_collisions!(
            viewport_overlay_providers,
            batch.viewport_overlay_providers,
            "viewport overlay provider"
        );
        reject_collisions!(graph_editors, batch.graph_editors, "graph editor");
        reject_collisions!(
            graph_node_palettes,
            batch.graph_node_palettes,
            "graph node palette"
        );
        reject_collisions!(timeline_editors, batch.timeline_editors, "timeline editor");
        reject_collisions!(
            timeline_track_types,
            batch.timeline_track_types,
            "timeline track type"
        );
        reject_collisions!(commands, batch.commands, "command");
        reject_collisions!(
            operation_factories,
            batch.operation_factories,
            "operation factory"
        );
        Ok(())
    }

    pub fn contribute(
        &mut self,
        source: ContributionSource,
        batch: ContributionBatch,
    ) -> Result<ContributionTicket, ContributionError> {
        self.validate_contribution(&source, &batch)?;
        let ticket = ContributionTicket(self.next_ticket);
        let keys = ContributionKeys::from_batch(&batch);
        let retained_batch = batch.clone();
        let capabilities: Arc<[String]> = batch.required_capabilities.clone().into();
        let mut candidate = (*self.current).clone();
        let mut counts = ContributionCounts::default();

        publish_entries!(
            candidate,
            views,
            batch.views,
            ticket,
            source,
            capabilities,
            counts,
            views,
            "view"
        );
        publish_entries!(
            candidate,
            drawers,
            batch.drawers,
            ticket,
            source,
            capabilities,
            counts,
            drawers,
            "drawer"
        );
        publish_entries!(
            candidate,
            menu_items,
            batch.menu_items,
            ticket,
            source,
            capabilities,
            counts,
            menu_items,
            "menu item"
        );
        publish_entries!(
            candidate,
            inspector_customizations,
            batch.inspector_customizations,
            ticket,
            source,
            capabilities,
            counts,
            inspector_customizations,
            "inspector customization"
        );
        publish_entries!(
            candidate,
            field_editors,
            batch.field_editors,
            ticket,
            source,
            capabilities,
            counts,
            field_editors,
            "field editor"
        );
        publish_entries!(
            candidate,
            ui_templates,
            batch.ui_templates,
            ticket,
            source,
            capabilities,
            counts,
            ui_templates,
            "ui template"
        );
        publish_entries!(
            candidate,
            ui_template_pane_data_sources,
            batch.ui_template_pane_data_sources,
            ticket,
            source,
            capabilities,
            counts,
            ui_template_pane_data_sources,
            "ui template pane data source"
        );
        publish_entries!(
            candidate,
            asset_importers,
            batch.asset_importers,
            ticket,
            source,
            capabilities,
            counts,
            asset_importers,
            "asset importer"
        );
        publish_entries!(
            candidate,
            asset_type_contributions,
            batch.asset_type_contributions,
            ticket,
            source,
            capabilities,
            counts,
            asset_type_contributions,
            "asset type contribution"
        );
        publish_entries!(
            candidate,
            settings_pages,
            batch.settings_pages,
            ticket,
            source,
            capabilities,
            counts,
            settings_pages,
            "settings page"
        );
        publish_entries!(
            candidate,
            scene_modes,
            batch.scene_modes,
            ticket,
            source,
            capabilities,
            counts,
            scene_modes,
            "scene mode"
        );
        publish_entries!(
            candidate,
            viewport_overlay_providers,
            batch.viewport_overlay_providers,
            ticket,
            source,
            capabilities,
            counts,
            viewport_overlay_providers,
            "viewport overlay provider"
        );
        publish_entries!(
            candidate,
            graph_editors,
            batch.graph_editors,
            ticket,
            source,
            capabilities,
            counts,
            graph_editors,
            "graph editor"
        );
        publish_entries!(
            candidate,
            graph_node_palettes,
            batch.graph_node_palettes,
            ticket,
            source,
            capabilities,
            counts,
            graph_node_palettes,
            "graph node palette"
        );
        publish_entries!(
            candidate,
            timeline_editors,
            batch.timeline_editors,
            ticket,
            source,
            capabilities,
            counts,
            timeline_editors,
            "timeline editor"
        );
        publish_entries!(
            candidate,
            timeline_track_types,
            batch.timeline_track_types,
            ticket,
            source,
            capabilities,
            counts,
            timeline_track_types,
            "timeline track type"
        );
        publish_entries!(
            candidate,
            commands,
            batch.commands,
            ticket,
            source,
            capabilities,
            counts,
            commands,
            "command"
        );
        publish_entries!(
            candidate,
            operation_factories,
            batch.operation_factories,
            ticket,
            source,
            capabilities,
            counts,
            operation_factories,
            "operation factory"
        );

        self.generation = self.generation.saturating_add(1);
        candidate.generation = self.generation;
        self.next_ticket = self.next_ticket.saturating_add(1);
        self.current = Arc::new(candidate);
        self.tickets.insert(
            ticket,
            TicketRecord {
                source: source.clone(),
                keys,
                counts: counts.clone(),
                batch: retained_batch,
            },
        );
        self.record_change(ContributionChange {
            generation: self.generation,
            ticket,
            source,
            kind: ContributionChangeKind::Contributed,
            counts,
        });
        Ok(ticket)
    }

    pub(crate) fn replace_ui_template_contributions(
        &mut self,
        ticket: ContributionTicket,
        templates: impl IntoIterator<Item = EditorUiTemplateDescriptor>,
        pane_data_sources: BTreeMap<String, Arc<dyn EditorUiTemplatePaneDataSource>>,
    ) -> Result<(), ContributionError> {
        let Some(record) = self.tickets.get(&ticket) else {
            return Err(ContributionError::UnknownTicket(ticket));
        };
        let source = record.source.clone();
        let old_template_keys = record.keys.ui_templates.clone();
        let old_source_keys = record.keys.ui_template_pane_data_sources.clone();
        let mut replacement_batch = record.batch.clone();
        replacement_batch.replace_ui_template_contributions(templates, pane_data_sources)?;
        validate_source_namespace(&source, &replacement_batch)?;

        let capabilities: Arc<[String]> = replacement_batch.required_capabilities.clone().into();
        let mut candidate = (*self.current).clone();
        remove_keys(&mut candidate.ui_templates, &old_template_keys);
        remove_keys(
            &mut candidate.ui_template_pane_data_sources,
            &old_source_keys,
        );
        {
            let target = Arc::make_mut(&mut candidate.ui_templates);
            for (id, value) in replacement_batch.ui_templates.iter() {
                if target.contains_key(id) {
                    return Err(ContributionError::DuplicateContribution {
                        kind: "ui template",
                        id: id.clone(),
                    });
                }
                target.insert(
                    id.clone(),
                    IndexedContribution::new(ticket, &source, &capabilities, value.clone()),
                );
            }
        }
        {
            let target = Arc::make_mut(&mut candidate.ui_template_pane_data_sources);
            for (id, value) in replacement_batch.ui_template_pane_data_sources.iter() {
                if target.contains_key(id) {
                    return Err(ContributionError::DuplicateContribution {
                        kind: "ui template pane data source",
                        id: id.clone(),
                    });
                }
                target.insert(
                    id.clone(),
                    IndexedContribution::new(ticket, &source, &capabilities, Arc::clone(value)),
                );
            }
        }

        let counts = {
            let Some(record) = self.tickets.get_mut(&ticket) else {
                return Err(ContributionError::UnknownTicket(ticket));
            };
            record.keys.ui_templates = replacement_batch.ui_templates.keys().cloned().collect();
            record.keys.ui_template_pane_data_sources = replacement_batch
                .ui_template_pane_data_sources
                .keys()
                .cloned()
                .collect();
            record.counts.ui_templates = record.keys.ui_templates.len();
            record.counts.ui_template_pane_data_sources =
                record.keys.ui_template_pane_data_sources.len();
            record.batch = replacement_batch;
            record.counts.clone()
        };
        self.generation = self.generation.saturating_add(1);
        candidate.generation = self.generation;
        self.current = Arc::new(candidate);
        self.record_change(ContributionChange {
            generation: self.generation,
            ticket,
            source,
            kind: ContributionChangeKind::Replaced,
            counts,
        });
        Ok(())
    }

    pub fn revoke(&mut self, ticket: ContributionTicket) -> RevokeReport {
        let Some(record) = self.tickets.remove(&ticket) else {
            return RevokeReport {
                ticket,
                source: None,
                generation: self.generation,
                removed: ContributionCounts::default(),
            };
        };
        let mut candidate = (*self.current).clone();
        remove_keys(&mut candidate.views, &record.keys.views);
        remove_keys(&mut candidate.drawers, &record.keys.drawers);
        remove_keys(&mut candidate.menu_items, &record.keys.menu_items);
        remove_keys(
            &mut candidate.inspector_customizations,
            &record.keys.inspector_customizations,
        );
        remove_keys(&mut candidate.field_editors, &record.keys.field_editors);
        remove_keys(&mut candidate.ui_templates, &record.keys.ui_templates);
        remove_keys(
            &mut candidate.ui_template_pane_data_sources,
            &record.keys.ui_template_pane_data_sources,
        );
        remove_keys(&mut candidate.asset_importers, &record.keys.asset_importers);
        remove_keys(
            &mut candidate.asset_type_contributions,
            &record.keys.asset_type_contributions,
        );
        remove_keys(&mut candidate.settings_pages, &record.keys.settings_pages);
        remove_keys(&mut candidate.scene_modes, &record.keys.scene_modes);
        remove_keys(
            &mut candidate.viewport_overlay_providers,
            &record.keys.viewport_overlay_providers,
        );
        remove_keys(&mut candidate.graph_editors, &record.keys.graph_editors);
        remove_keys(
            &mut candidate.graph_node_palettes,
            &record.keys.graph_node_palettes,
        );
        remove_keys(
            &mut candidate.timeline_editors,
            &record.keys.timeline_editors,
        );
        remove_keys(
            &mut candidate.timeline_track_types,
            &record.keys.timeline_track_types,
        );
        remove_keys(&mut candidate.commands, &record.keys.commands);
        remove_keys(
            &mut candidate.operation_factories,
            &record.keys.operation_factories,
        );

        self.generation = self.generation.saturating_add(1);
        candidate.generation = self.generation;
        self.current = Arc::new(candidate);
        self.record_change(ContributionChange {
            generation: self.generation,
            ticket,
            source: record.source.clone(),
            kind: ContributionChangeKind::Revoked,
            counts: record.counts.clone(),
        });
        RevokeReport {
            ticket,
            source: Some(record.source),
            generation: self.generation,
            removed: record.counts,
        }
    }

    pub fn changed_since(&self, generation: u64) -> ContributionDelta {
        let reset = !self.can_replay_from(generation);
        ContributionDelta {
            from_generation: generation,
            to_generation: self.generation,
            reset,
            changes: if reset {
                Vec::<ContributionChange>::new().into()
            } else {
                self.changes
                    .iter()
                    .filter(|change| change.generation > generation)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into()
            },
        }
    }

    fn record_change(&mut self, change: ContributionChange) {
        if self.changes.len() == CONTRIBUTION_CHANGE_JOURNAL_CAPACITY {
            self.changes.pop_front();
        }
        self.changes.push_back(change);
    }

    fn can_replay_from(&self, generation: u64) -> bool {
        if generation >= self.generation {
            return generation == self.generation;
        }
        self.changes
            .front()
            .is_some_and(|oldest| generation >= oldest.generation().saturating_sub(1))
    }
}

fn remove_keys<K, V>(map: &mut IndexedMap<K, V>, keys: &[K])
where
    K: Clone + Ord,
    V: Clone,
{
    if keys.is_empty() {
        return;
    }
    let map = Arc::make_mut(map);
    for key in keys {
        map.remove(key);
    }
}
