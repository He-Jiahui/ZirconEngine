use std::collections::{BTreeMap, btree_map::Entry};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::asset::{AssetTypeContribution, AssetTypeId};
use crate::core::commands::{
    EditorCommandContributionSet, EditorCommandDescriptor, EditorCommandRegistryError,
};
use crate::core::editing::operation::OperationCommandFactoryRegistration;
use crate::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodePaletteDescriptor, SceneModeDescriptor,
    TimelineEditorDescriptor, TimelineTrackDescriptor,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{ContributionTicket, InspectorCustomizationDescriptor};
use crate::core::i18n::EditorLocalizationBundle;
use crate::core::settings::SettingsPageDescriptor;
use crate::core::tools::{ToolOwnerGeneration, ToolResourceKindDeclaration};
use crate::scene::modes::SceneModeRegistration;

use contribution_descriptors::{validate_asset_importer, validate_menu_item_path};

mod contribution_descriptors;
mod template_contributions;
mod view_descriptor;
mod viewport_overlay_provider;

pub use contribution_descriptors::{
    AssetImporterDescriptor, DrawerDescriptor, EditorExtensionRegistryError,
    EditorMenuItemDescriptor,
};
pub use template_contributions::EditorUiTemplateDescriptor;
pub use view_descriptor::{
    EditorUiTemplatePaneDataSnapshot, EditorUiTemplatePaneDataSource, ViewDescriptor,
};
pub use viewport_overlay_provider::{
    ViewportOverlayProvider, ViewportOverlayProviderContext, ViewportOverlayProviderFactory,
    ViewportOverlayProviderRegistration,
};

/// Exact host-issued identity of one live editor contribution generation.
///
/// Display owner ids are intentionally insufficient for retirement: a delayed teardown from an
/// older load must not revoke a newer contribution that reused the same plugin id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorContributionHandle {
    owner_id: String,
    ticket: ContributionTicket,
    owner_generation: ToolOwnerGeneration,
}

impl EditorContributionHandle {
    pub(crate) fn new(
        owner_id: impl Into<String>,
        ticket: ContributionTicket,
        owner_generation: ToolOwnerGeneration,
    ) -> Self {
        Self {
            owner_id: owner_id.into(),
            ticket,
            owner_generation,
        }
    }

    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub(crate) const fn ticket(&self) -> ContributionTicket {
        self.ticket
    }

    pub(crate) const fn owner_generation(&self) -> ToolOwnerGeneration {
        self.owner_generation
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorExtensionRegistry {
    views: BTreeMap<String, ViewDescriptor>,
    drawers: BTreeMap<String, DrawerDescriptor>,
    menu_items: BTreeMap<String, EditorMenuItemDescriptor>,
    inspector_customizations: BTreeMap<String, InspectorCustomizationDescriptor>,
    ui_templates: BTreeMap<String, EditorUiTemplateDescriptor>,
    #[serde(skip)]
    ui_template_root: Option<PathBuf>,
    #[serde(skip)]
    ui_template_pane_data_sources:
        BTreeMap<String, template_contributions::EditorUiTemplatePaneDataSourceRegistration>,
    asset_importers: BTreeMap<String, AssetImporterDescriptor>,
    asset_type_contributions: BTreeMap<AssetTypeId, AssetTypeContribution>,
    #[serde(skip)]
    localization_bundles: BTreeMap<String, EditorLocalizationBundle>,
    settings_pages: BTreeMap<String, SettingsPageDescriptor>,
    #[serde(skip)]
    scene_mode_descriptors: BTreeMap<String, SceneModeDescriptor>,
    #[serde(skip)]
    scene_mode_registrations: BTreeMap<String, SceneModeRegistration>,
    #[serde(skip)]
    viewport_overlay_providers: BTreeMap<String, ViewportOverlayProviderRegistration>,
    graph_editors: BTreeMap<AssetTypeId, GraphEditorDescriptor>,
    graph_node_palettes: BTreeMap<String, GraphNodePaletteDescriptor>,
    timeline_editors: BTreeMap<AssetTypeId, TimelineEditorDescriptor>,
    timeline_track_types: BTreeMap<String, TimelineTrackDescriptor>,
    tool_resource_kinds: BTreeMap<String, ToolResourceKindDeclaration>,
    command_contributions: EditorCommandContributionSet,
}

impl EditorExtensionRegistry {
    pub(crate) fn into_contribution_batch(
        mut self,
    ) -> Result<crate::core::extension::ContributionBatch, EditorExtensionRegistryError> {
        let pane_data_sources = self.ui_template_pane_data_sources();
        let mut commands = self.take_command_contributions();
        for command in &mut commands {
            let Some(bundle_id) = command.presentation().source().bundle_id() else {
                continue;
            };
            let bundle = self
                .localization_bundles
                .get(bundle_id.as_str())
                .ok_or_else(|| EditorExtensionRegistryError::CommandLocalization {
                    command_id: command.id().clone(),
                    detail: format!("missing localization bundle `{bundle_id}`"),
                })?;
            command.bind_localization_bundle(bundle).map_err(|detail| {
                EditorExtensionRegistryError::CommandLocalization {
                    command_id: command.id().clone(),
                    detail,
                }
            })?;
        }
        let mut factories = self
            .take_operation_factories()
            .into_iter()
            .map(|factory| (factory.operation().clone(), factory))
            .collect::<BTreeMap<_, _>>();
        let mut batch = crate::core::extension::ContributionBatch::default();

        for descriptor in self.views.into_values() {
            batch.register_view(descriptor)?;
        }
        for descriptor in self.drawers.into_values() {
            batch.register_drawer(descriptor)?;
        }
        for descriptor in self.menu_items.into_values() {
            batch.register_menu_item(descriptor)?;
        }
        for descriptor in self.inspector_customizations.into_values() {
            batch.register_inspector_customization(Arc::new(descriptor))?;
        }
        for descriptor in self.ui_templates.into_values() {
            batch.register_ui_template(descriptor)?;
        }
        for (template_id, source) in pane_data_sources {
            batch.register_ui_template_pane_data_source(template_id, source)?;
        }
        for descriptor in self.asset_importers.into_values() {
            batch.register_asset_importer(descriptor)?;
        }
        for contribution in self.asset_type_contributions.into_values() {
            batch.register_asset_type_contribution(contribution)?;
        }
        for bundle in self.localization_bundles.into_values() {
            batch.register_localization_bundle(bundle)?;
        }
        for descriptor in self.settings_pages.into_values() {
            batch.register_settings_page(descriptor)?;
        }
        for registration in self.scene_mode_registrations.into_values() {
            batch.register_scene_mode(registration)?;
        }
        for registration in self.viewport_overlay_providers.into_values() {
            batch.register_viewport_overlay_provider(registration)?;
        }
        for descriptor in self.graph_editors.into_values() {
            batch.register_graph_editor(descriptor)?;
        }
        for descriptor in self.graph_node_palettes.into_values() {
            batch.register_graph_node_palette(descriptor)?;
        }
        for descriptor in self.timeline_editors.into_values() {
            batch.register_timeline_editor(descriptor)?;
        }
        for descriptor in self.timeline_track_types.into_values() {
            batch.register_timeline_track_type(descriptor)?;
        }
        for declaration in self.tool_resource_kinds.into_values() {
            batch.register_tool_resource_kind(declaration)?;
        }
        for command in commands {
            if let Some(factory) = factories.remove(command.id()) {
                batch.register_operation_command(command, factory)?;
            } else {
                batch.register_command(command)?;
            }
        }
        if let Some((operation, _)) = factories.into_iter().next() {
            return Err(EditorExtensionRegistryError::Command(
                EditorCommandRegistryError::OperationFactory(
                    crate::core::editing::operation::OperationCommandFactoryError::OrphanFactory {
                        operation,
                    },
                ),
            ));
        }
        Ok(batch)
    }

    pub fn register_view(
        &mut self,
        descriptor: ViewDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        descriptor
            .open_operation_path()
            .map_err(EditorExtensionRegistryError::OperationPath)?;
        let id = descriptor.id().to_owned();
        insert_unique(&mut self.views, id, descriptor, "view")
    }

    pub fn register_drawer(
        &mut self,
        descriptor: DrawerDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        insert_unique(
            &mut self.drawers,
            descriptor.id().to_string(),
            descriptor,
            "drawer",
        )
    }

    pub fn register_menu_item(
        &mut self,
        descriptor: EditorMenuItemDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_menu_item_path(&descriptor)?;
        insert_unique(
            &mut self.menu_items,
            descriptor.path().to_string(),
            descriptor,
            "menu item",
        )
    }

    pub fn register_inspector_customization(
        &mut self,
        descriptor: InspectorCustomizationDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        descriptor
            .validate()
            .map_err(|error| EditorExtensionRegistryError::View(error.to_string()))?;
        crate::core::extension::ContributionBatch::default()
            .register_inspector_customization(Arc::new(descriptor.clone()))?;
        let id = descriptor.id().to_string();
        insert_unique(
            &mut self.inspector_customizations,
            id,
            descriptor,
            "inspector customization",
        )
    }

    pub fn register_asset_importer(
        &mut self,
        descriptor: AssetImporterDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_asset_importer(&descriptor)?;
        insert_unique(
            &mut self.asset_importers,
            descriptor.id().to_string(),
            descriptor,
            "asset importer",
        )
    }

    pub fn register_asset_type_contribution(
        &mut self,
        contribution: AssetTypeContribution,
    ) -> Result<(), EditorExtensionRegistryError> {
        let asset_type = contribution.asset_type().clone();
        if self.asset_type_contributions.contains_key(&asset_type) {
            return Err(EditorExtensionRegistryError::DuplicateContribution {
                kind: "asset type contribution",
                id: asset_type.to_string(),
            });
        }
        self.asset_type_contributions
            .insert(asset_type, contribution);
        Ok(())
    }

    pub fn register_settings_page(
        &mut self,
        descriptor: SettingsPageDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_contribution_id("settings page", descriptor.id())?;
        let Some(bundle) = self
            .localization_bundles
            .get(descriptor.localization_bundle_id())
        else {
            return Err(EditorExtensionRegistryError::MissingLocalizationBundle {
                page_id: descriptor.id().to_owned(),
                bundle_id: descriptor.localization_bundle_id().to_owned(),
            });
        };
        if let Some(key) = descriptor
            .localization_keys()
            .find(|key| !bundle.contains_key(key))
            .map(str::to_owned)
        {
            return Err(EditorExtensionRegistryError::UnknownLocalizationKey {
                page_id: descriptor.id().to_owned(),
                bundle_id: descriptor.localization_bundle_id().to_owned(),
                key,
            });
        }
        insert_unique(
            &mut self.settings_pages,
            descriptor.id().to_string(),
            descriptor,
            "settings page",
        )
    }

    pub fn register_localization_bundle(
        &mut self,
        bundle: EditorLocalizationBundle,
    ) -> Result<(), EditorExtensionRegistryError> {
        insert_unique(
            &mut self.localization_bundles,
            bundle.id().to_owned(),
            bundle,
            "localization bundle",
        )
    }

    pub fn register_scene_mode(
        &mut self,
        registration: SceneModeRegistration,
    ) -> Result<(), EditorExtensionRegistryError> {
        let descriptor = registration.descriptor().clone();
        validate_contribution_id("scene mode", descriptor.id())?;
        validate_contribution_id("scene mode view", descriptor.view_id())?;
        if let Some(provider_id) = descriptor.overlay_provider_id() {
            validate_contribution_id("viewport overlay provider", provider_id)?;
        }
        let mode_id = descriptor.id().to_string();
        if self.scene_mode_descriptors.contains_key(&mode_id)
            || self.scene_mode_registrations.contains_key(&mode_id)
        {
            return Err(EditorExtensionRegistryError::DuplicateContribution {
                kind: "scene mode",
                id: mode_id,
            });
        }
        self.scene_mode_descriptors
            .insert(mode_id.clone(), descriptor);
        self.scene_mode_registrations.insert(mode_id, registration);
        Ok(())
    }

    pub fn register_viewport_overlay_provider(
        &mut self,
        registration: ViewportOverlayProviderRegistration,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_contribution_id("viewport overlay provider", registration.provider_id())?;
        insert_unique(
            &mut self.viewport_overlay_providers,
            registration.provider_id().to_string(),
            registration,
            "viewport overlay provider",
        )
    }

    pub fn register_graph_editor(
        &mut self,
        descriptor: GraphEditorDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_contribution_id("graph editor view", descriptor.view_id())?;
        let asset_type = descriptor.asset_type().clone();
        if self.graph_editors.contains_key(&asset_type) {
            return Err(EditorExtensionRegistryError::DuplicateContribution {
                kind: "graph editor",
                id: asset_type.to_string(),
            });
        }
        self.graph_editors.insert(asset_type, descriptor);
        Ok(())
    }

    pub fn register_graph_node_palette(
        &mut self,
        descriptor: GraphNodePaletteDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_graph_node_palette(&descriptor)?;
        insert_unique(
            &mut self.graph_node_palettes,
            descriptor.id().to_string(),
            descriptor,
            "graph node palette",
        )
    }

    pub fn register_timeline_editor(
        &mut self,
        descriptor: TimelineEditorDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_contribution_id("timeline editor view", descriptor.view_id())?;
        let asset_type = descriptor.asset_type().clone();
        if self.timeline_editors.contains_key(&asset_type) {
            return Err(EditorExtensionRegistryError::DuplicateContribution {
                kind: "timeline editor",
                id: asset_type.to_string(),
            });
        }
        self.timeline_editors.insert(asset_type, descriptor);
        Ok(())
    }

    pub fn register_timeline_track_type(
        &mut self,
        descriptor: TimelineTrackDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_contribution_id("timeline track type", descriptor.id())?;
        validate_contribution_id("timeline track value kind", descriptor.value_kind())?;
        insert_unique(
            &mut self.timeline_track_types,
            descriptor.id().to_string(),
            descriptor,
            "timeline track type",
        )
    }

    pub fn register_command(
        &mut self,
        descriptor: EditorCommandDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        self.command_contributions
            .register(descriptor)
            .map_err(EditorExtensionRegistryError::Command)
    }

    pub fn register_operation_command(
        &mut self,
        descriptor: EditorCommandDescriptor,
        factory: OperationCommandFactoryRegistration,
    ) -> Result<(), EditorExtensionRegistryError> {
        self.command_contributions
            .register_operation(descriptor, factory)
            .map_err(EditorExtensionRegistryError::Command)
    }

    pub fn register_tool_resource_kind(
        &mut self,
        declaration: ToolResourceKindDeclaration,
    ) -> Result<(), EditorExtensionRegistryError> {
        let kind = declaration.kind().as_str().to_owned();
        insert_unique(
            &mut self.tool_resource_kinds,
            kind,
            declaration,
            "tool resource kind",
        )
    }

    pub fn views(&self) -> Vec<&ViewDescriptor> {
        self.views.values().collect()
    }

    pub fn drawers(&self) -> Vec<&DrawerDescriptor> {
        self.drawers.values().collect()
    }

    pub fn menu_items(&self) -> Vec<&EditorMenuItemDescriptor> {
        self.menu_items.values().collect()
    }

    pub fn inspector_customizations(&self) -> Vec<&InspectorCustomizationDescriptor> {
        self.inspector_customizations.values().collect()
    }

    pub(crate) fn bind_matching_ui_templates_to_views(&mut self) {
        let template_ids = self
            .ui_templates
            .iter()
            .filter(|(_, template)| template.ui_document().starts_with("plugins://"))
            .map(|(template_id, _)| template_id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        for view in self.views.values_mut() {
            if view.ui_template_id().is_none() && template_ids.contains(view.id()) {
                view.bind_ui_template_id(view.id().to_string());
            }
        }
    }

    pub fn asset_importers(&self) -> Vec<&AssetImporterDescriptor> {
        self.asset_importers.values().collect()
    }

    pub fn asset_type_contributions(&self) -> Vec<&AssetTypeContribution> {
        self.asset_type_contributions.values().collect()
    }

    pub fn settings_pages(&self) -> Vec<&SettingsPageDescriptor> {
        self.settings_pages.values().collect()
    }

    pub fn localization_bundles(&self) -> Vec<&EditorLocalizationBundle> {
        self.localization_bundles.values().collect()
    }

    pub fn scene_mode_descriptors(&self) -> Vec<&SceneModeDescriptor> {
        self.scene_mode_descriptors.values().collect()
    }

    pub(crate) fn scene_mode_registrations(&self) -> Vec<&SceneModeRegistration> {
        self.scene_mode_registrations.values().collect()
    }

    pub(crate) fn take_scene_modes(&mut self) -> Vec<SceneModeRegistration> {
        std::mem::take(&mut self.scene_mode_registrations)
            .into_values()
            .collect()
    }

    pub(crate) fn has_viewport_overlay_provider(&self, provider_id: &str) -> bool {
        self.viewport_overlay_providers.contains_key(provider_id)
    }

    pub(crate) fn take_viewport_overlay_providers(
        &mut self,
    ) -> Vec<ViewportOverlayProviderRegistration> {
        std::mem::take(&mut self.viewport_overlay_providers)
            .into_values()
            .collect()
    }

    pub fn graph_editors(&self) -> Vec<&GraphEditorDescriptor> {
        self.graph_editors.values().collect()
    }

    pub fn graph_node_palettes(&self) -> Vec<&GraphNodePaletteDescriptor> {
        self.graph_node_palettes.values().collect()
    }

    pub fn timeline_editors(&self) -> Vec<&TimelineEditorDescriptor> {
        self.timeline_editors.values().collect()
    }

    pub fn timeline_track_types(&self) -> Vec<&TimelineTrackDescriptor> {
        self.timeline_track_types.values().collect()
    }

    pub fn tool_resource_kinds(&self) -> Vec<&ToolResourceKindDeclaration> {
        self.tool_resource_kinds.values().collect()
    }

    pub fn command_ids(&self) -> impl Iterator<Item = &EditorOperationPath> {
        self.command_contributions.command_ids()
    }

    pub fn pending_command(&self, id: &EditorOperationPath) -> Option<&EditorCommandDescriptor> {
        self.command_contributions.pending_command(id)
    }

    pub fn pending_commands(&self) -> impl Iterator<Item = &EditorCommandDescriptor> {
        self.command_contributions.pending_commands()
    }

    pub fn operation_factory(
        &self,
        id: &EditorOperationPath,
    ) -> Option<&OperationCommandFactoryRegistration> {
        self.command_contributions.pending_factory(id)
    }

    pub(crate) fn take_command_contributions(&mut self) -> Vec<EditorCommandDescriptor> {
        self.command_contributions.take_pending()
    }

    pub(crate) fn take_operation_factories(&mut self) -> Vec<OperationCommandFactoryRegistration> {
        self.command_contributions.take_pending_factories()
    }

    pub(crate) fn record_registered_command_id(&mut self, id: EditorOperationPath) {
        self.command_contributions.record_registered_id(id);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorExtensionRegistration {
    registry: EditorExtensionRegistry,
    owner_id: String,
    required_capabilities: Vec<String>,
}

impl EditorExtensionRegistration {
    pub fn new(registry: EditorExtensionRegistry) -> Self {
        Self {
            registry,
            owner_id: "editor.extension.direct".to_owned(),
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_owner_id(mut self, owner_id: impl Into<String>) -> Self {
        self.owner_id = owner_id.into();
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_capabilities = capabilities.into_iter().map(Into::into).collect();
        self.required_capabilities.sort();
        self.required_capabilities.dedup();
        self
    }

    pub fn registry(&self) -> &EditorExtensionRegistry {
        &self.registry
    }

    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }

    pub fn is_enabled_by<I, S>(&self, enabled_capabilities: I) -> bool
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let enabled = enabled_capabilities
            .into_iter()
            .map(|capability| capability.as_ref().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        self.required_capabilities
            .iter()
            .all(|capability| enabled.contains(capability))
    }
}

fn validate_graph_node_palette(
    descriptor: &GraphNodePaletteDescriptor,
) -> Result<(), EditorExtensionRegistryError> {
    validate_contribution_id("graph node palette", descriptor.id())?;
    validate_contribution_id("graph node palette owner", descriptor.owner_id())?;
    if descriptor.schema_version() == 0 {
        return Err(
            EditorExtensionRegistryError::InvalidDescriptorSchemaVersion {
                kind: "graph node palette",
                id: descriptor.id().to_owned(),
                version: descriptor.schema_version(),
            },
        );
    }
    if descriptor.nodes().is_empty() {
        return Err(EditorExtensionRegistryError::View(format!(
            "editor graph node palette `{}` must declare at least one node",
            descriptor.id()
        )));
    }
    let mut node_ids = std::collections::BTreeSet::new();
    for node in descriptor.nodes() {
        validate_contribution_id("graph node", node.id())?;
        if !node_ids.insert(node.id()) {
            return Err(EditorExtensionRegistryError::DuplicateContribution {
                kind: "graph node",
                id: node.id().to_string(),
            });
        }
    }
    Ok(())
}

fn validate_contribution_id(
    kind: &'static str,
    id: &str,
) -> Result<(), EditorExtensionRegistryError> {
    if id.trim().is_empty() || id.trim() != id {
        return Err(EditorExtensionRegistryError::InvalidContributionId {
            kind,
            id: id.to_string(),
        });
    }
    Ok(())
}

fn insert_unique<T>(
    map: &mut BTreeMap<String, T>,
    id: String,
    descriptor: T,
    kind: &'static str,
) -> Result<(), EditorExtensionRegistryError> {
    match map.entry(id) {
        Entry::Occupied(entry) => Err(EditorExtensionRegistryError::DuplicateContribution {
            kind,
            id: entry.key().clone(),
        }),
        Entry::Vacant(entry) => {
            entry.insert(descriptor);
            Ok(())
        }
    }
}
