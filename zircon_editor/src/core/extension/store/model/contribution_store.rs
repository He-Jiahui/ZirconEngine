use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use crate::core::editor_extension::{EditorUiTemplateDescriptor, EditorUiTemplatePaneDataSource};
use crate::core::extension::FieldEditorContainer;

use super::super::ContributionBatch;
use super::lifecycle::{validate_source_namespace, ContributionError, RevokeReport};
use super::records::{
    ContributionChange, ContributionChangeKind, ContributionCounts, ContributionDelta,
    ContributionKeys, ContributionSource, ContributionTicket, TicketRecord,
    CONTRIBUTION_CHANGE_JOURNAL_CAPACITY,
};
use super::snapshot::{ContributionSnapshot, IndexedContribution, IndexedMap};

#[derive(Clone)]
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

    /// Enumerates the authoritative active batches in ticket order.
    ///
    /// Derived registries rebuild from this view so ticket revocation cannot leave executable
    /// side effects behind in a second mutable catalog.
    pub(crate) fn active_batches(&self) -> impl Iterator<Item = &ContributionBatch> {
        self.tickets.values().map(|record| &record.batch)
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
        reject_collisions!(
            localization_bundles,
            batch.localization_bundles,
            "localization bundle"
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
            localization_bundles,
            batch.localization_bundles,
            ticket,
            source,
            capabilities,
            counts,
            localization_bundles,
            "localization bundle"
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
        remove_keys(
            &mut candidate.localization_bundles,
            &record.keys.localization_bundles,
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
