use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::asset::{AssetTypeContribution, AssetTypeId};
use crate::core::commands::EditorCommandDescriptor;
use crate::core::editing::operation::OperationCommandFactoryRegistration;
use crate::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodePaletteDescriptor, SceneModeDescriptor,
    TimelineEditorDescriptor, TimelineTrackDescriptor,
};
use crate::core::editor_extension::{
    AssetImporterDescriptor, DrawerDescriptor, EditorMenuItemDescriptor,
    EditorUiTemplateDescriptor, EditorUiTemplatePaneDataSource, ViewDescriptor,
    ViewportOverlayProviderRegistration,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{FieldEditorDefinition, InspectorCustomization};
use crate::core::settings::SettingsPageDescriptor;
use crate::scene::modes::SceneModeRegistration;

use super::{CapabilitySet, ContributionSource, ContributionTicket};

#[derive(Clone)]
pub(super) struct IndexedContribution<T> {
    ticket: ContributionTicket,
    source: ContributionSource,
    required_capabilities: Arc<[String]>,
    value: T,
}

impl<T> IndexedContribution<T> {
    pub(super) fn new(
        ticket: ContributionTicket,
        source: &ContributionSource,
        required_capabilities: &Arc<[String]>,
        value: T,
    ) -> Self {
        Self {
            ticket,
            source: source.clone(),
            required_capabilities: Arc::clone(required_capabilities),
            value,
        }
    }

    fn is_enabled_by(&self, capabilities: &CapabilitySet) -> bool {
        self.required_capabilities
            .iter()
            .all(|capability| capabilities.contains(capability))
    }
}

pub(super) type IndexedMap<K, V> = Arc<BTreeMap<K, IndexedContribution<V>>>;

#[derive(Clone, Default)]
pub struct ContributionSnapshot {
    pub(super) generation: u64,
    pub(super) views: IndexedMap<String, ViewDescriptor>,
    pub(super) drawers: IndexedMap<String, DrawerDescriptor>,
    pub(super) menu_items: IndexedMap<String, EditorMenuItemDescriptor>,
    pub(super) inspector_customizations: IndexedMap<String, Arc<dyn InspectorCustomization>>,
    pub(super) field_editors: IndexedMap<String, FieldEditorDefinition>,
    pub(super) ui_templates: IndexedMap<String, EditorUiTemplateDescriptor>,
    pub(super) ui_template_pane_data_sources:
        IndexedMap<String, Arc<dyn EditorUiTemplatePaneDataSource>>,
    pub(super) asset_importers: IndexedMap<String, AssetImporterDescriptor>,
    pub(super) asset_type_contributions: IndexedMap<AssetTypeId, AssetTypeContribution>,
    pub(super) settings_pages: IndexedMap<String, SettingsPageDescriptor>,
    pub(super) scene_modes: IndexedMap<String, SceneModeRegistration>,
    pub(super) viewport_overlay_providers: IndexedMap<String, ViewportOverlayProviderRegistration>,
    pub(super) graph_editors: IndexedMap<AssetTypeId, GraphEditorDescriptor>,
    pub(super) graph_node_palettes: IndexedMap<String, GraphNodePaletteDescriptor>,
    pub(super) timeline_editors: IndexedMap<AssetTypeId, TimelineEditorDescriptor>,
    pub(super) timeline_track_types: IndexedMap<String, TimelineTrackDescriptor>,
    pub(super) commands: IndexedMap<EditorOperationPath, EditorCommandDescriptor>,
    pub(super) operation_factories:
        IndexedMap<EditorOperationPath, OperationCommandFactoryRegistration>,
}

macro_rules! snapshot_iter {
    ($name:ident, $field:ident, $value:ty) => {
        pub fn $name<'a>(
            &'a self,
            capabilities: &'a CapabilitySet,
        ) -> impl Iterator<Item = &'a $value> + 'a {
            self.$field
                .values()
                .filter(move |entry| entry.is_enabled_by(capabilities))
                .map(|entry| &entry.value)
        }
    };
}

impl ContributionSnapshot {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    snapshot_iter!(views, views, ViewDescriptor);
    snapshot_iter!(drawers, drawers, DrawerDescriptor);
    snapshot_iter!(menu_items, menu_items, EditorMenuItemDescriptor);
    pub fn inspector_customizations<'a>(
        &'a self,
        capabilities: &'a CapabilitySet,
    ) -> impl Iterator<Item = Arc<dyn InspectorCustomization>> + 'a {
        self.inspector_customizations
            .values()
            .filter(move |entry| entry.is_enabled_by(capabilities))
            .map(|entry| Arc::clone(&entry.value))
    }
    snapshot_iter!(field_editors, field_editors, FieldEditorDefinition);
    snapshot_iter!(ui_templates, ui_templates, EditorUiTemplateDescriptor);
    snapshot_iter!(asset_importers, asset_importers, AssetImporterDescriptor);
    snapshot_iter!(
        asset_type_contributions,
        asset_type_contributions,
        AssetTypeContribution
    );
    snapshot_iter!(settings_pages, settings_pages, SettingsPageDescriptor);
    snapshot_iter!(
        viewport_overlay_providers,
        viewport_overlay_providers,
        ViewportOverlayProviderRegistration
    );
    snapshot_iter!(graph_editors, graph_editors, GraphEditorDescriptor);
    snapshot_iter!(
        graph_node_palettes,
        graph_node_palettes,
        GraphNodePaletteDescriptor
    );
    snapshot_iter!(timeline_editors, timeline_editors, TimelineEditorDescriptor);
    snapshot_iter!(
        timeline_track_types,
        timeline_track_types,
        TimelineTrackDescriptor
    );
    snapshot_iter!(commands, commands, EditorCommandDescriptor);
    snapshot_iter!(
        operation_factories,
        operation_factories,
        OperationCommandFactoryRegistration
    );

    pub fn scene_mode_descriptors<'a>(
        &'a self,
        capabilities: &'a CapabilitySet,
    ) -> impl Iterator<Item = &'a SceneModeDescriptor> + 'a {
        self.scene_modes
            .values()
            .filter(move |entry| entry.is_enabled_by(capabilities))
            .map(|entry| entry.value.descriptor())
    }

    pub fn scene_mode_registrations<'a>(
        &'a self,
        capabilities: &'a CapabilitySet,
    ) -> impl Iterator<Item = &'a SceneModeRegistration> + 'a {
        self.scene_modes
            .values()
            .filter(move |entry| entry.is_enabled_by(capabilities))
            .map(|entry| &entry.value)
    }

    pub fn ui_template_pane_data_source(
        &self,
        template_id: &str,
        capabilities: &CapabilitySet,
    ) -> Option<Arc<dyn EditorUiTemplatePaneDataSource>> {
        self.ui_template_pane_data_sources
            .get(template_id)
            .filter(|entry| entry.is_enabled_by(capabilities))
            .map(|entry| Arc::clone(&entry.value))
    }

    pub(crate) fn ui_template_pane_data_sources<'a>(
        &'a self,
        capabilities: &'a CapabilitySet,
    ) -> impl Iterator<Item = (&'a str, Arc<dyn EditorUiTemplatePaneDataSource>)> + 'a {
        self.ui_template_pane_data_sources
            .iter()
            .filter(move |(_, entry)| entry.is_enabled_by(capabilities))
            .map(|(template_id, entry)| (template_id.as_str(), Arc::clone(&entry.value)))
    }

    pub fn source_for_view(&self, view_id: &str) -> Option<&ContributionSource> {
        self.views.get(view_id).map(|entry| &entry.source)
    }

    /// Enumerates every current view owned by `ticket`, including capability-gated views.
    ///
    /// Host materialization uses this before revoking a ticket so it can remove the exact
    /// descriptor set without maintaining a second owner-to-view registry.
    pub fn views_for_ticket(
        &self,
        ticket: ContributionTicket,
    ) -> impl Iterator<Item = &ViewDescriptor> {
        self.views
            .values()
            .filter(move |entry| entry.ticket == ticket)
            .map(|entry| &entry.value)
    }

    pub(crate) fn ui_templates_with_source<'a>(
        &'a self,
        capabilities: &'a CapabilitySet,
    ) -> impl Iterator<Item = (&'a ContributionSource, &'a EditorUiTemplateDescriptor)> + 'a {
        self.ui_templates
            .values()
            .filter(move |entry| entry.is_enabled_by(capabilities))
            .map(|entry| (&entry.source, &entry.value))
    }

    pub(crate) fn asset_type_contributions_with_source<'a>(
        &'a self,
        capabilities: &'a CapabilitySet,
    ) -> impl Iterator<Item = (&'a ContributionSource, &'a AssetTypeContribution)> + 'a {
        self.asset_type_contributions
            .values()
            .filter(move |entry| entry.is_enabled_by(capabilities))
            .map(|entry| (&entry.source, &entry.value))
    }

    pub(crate) fn all_asset_type_contributions_with_source(
        &self,
    ) -> impl Iterator<Item = (&ContributionSource, &AssetTypeContribution)> {
        self.asset_type_contributions
            .values()
            .map(|entry| (&entry.source, &entry.value))
    }

    pub fn ticket_for_view(&self, view_id: &str) -> Option<ContributionTicket> {
        self.views.get(view_id).map(|entry| entry.ticket)
    }
}
