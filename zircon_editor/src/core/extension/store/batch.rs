use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::core::asset::{AssetTypeContribution, AssetTypeId};
use crate::core::commands::{EditorCommandDescriptor, EditorCommandRegistryError};
use crate::core::editing::operation::OperationCommandFactoryRegistration;
use crate::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodePaletteDescriptor, SceneModeDescriptor,
    TimelineEditorDescriptor, TimelineTrackDescriptor,
};
use crate::core::editor_extension::{
    AssetImporterDescriptor, DrawerDescriptor, EditorExtensionRegistryError,
    EditorMenuItemDescriptor, EditorUiTemplateDescriptor, EditorUiTemplatePaneDataSource,
    ViewDescriptor, ViewportOverlayProviderRegistration,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{FieldEditorDefinition, InspectorCustomization};
use crate::core::settings::SettingsPageDescriptor;
use crate::scene::modes::SceneModeRegistration;

#[derive(Clone, Default)]
pub struct ContributionBatch {
    pub(super) views: BTreeMap<String, ViewDescriptor>,
    pub(super) drawers: BTreeMap<String, DrawerDescriptor>,
    pub(super) menu_items: BTreeMap<String, EditorMenuItemDescriptor>,
    pub(super) inspector_customizations: BTreeMap<String, Arc<dyn InspectorCustomization>>,
    pub(super) field_editors: BTreeMap<String, FieldEditorDefinition>,
    pub(super) ui_templates: BTreeMap<String, EditorUiTemplateDescriptor>,
    pub(super) ui_template_pane_data_sources:
        BTreeMap<String, Arc<dyn EditorUiTemplatePaneDataSource>>,
    pub(super) asset_importers: BTreeMap<String, AssetImporterDescriptor>,
    pub(super) asset_type_contributions: BTreeMap<AssetTypeId, AssetTypeContribution>,
    pub(super) settings_pages: BTreeMap<String, SettingsPageDescriptor>,
    pub(super) scene_modes: BTreeMap<String, SceneModeRegistration>,
    pub(super) viewport_overlay_providers: BTreeMap<String, ViewportOverlayProviderRegistration>,
    pub(super) graph_editors: BTreeMap<AssetTypeId, GraphEditorDescriptor>,
    pub(super) graph_node_palettes: BTreeMap<String, GraphNodePaletteDescriptor>,
    pub(super) timeline_editors: BTreeMap<AssetTypeId, TimelineEditorDescriptor>,
    pub(super) timeline_track_types: BTreeMap<String, TimelineTrackDescriptor>,
    pub(super) commands: BTreeMap<EditorOperationPath, EditorCommandDescriptor>,
    pub(super) operation_factories:
        BTreeMap<EditorOperationPath, OperationCommandFactoryRegistration>,
    pub(super) required_capabilities: Vec<String>,
}

impl fmt::Debug for ContributionBatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ContributionBatch")
            .field("views", &self.views.len())
            .field("drawers", &self.drawers.len())
            .field("menu_items", &self.menu_items.len())
            .field(
                "inspector_customizations",
                &self.inspector_customizations.len(),
            )
            .field("field_editors", &self.field_editors.len())
            .field("ui_templates", &self.ui_templates.len())
            .field(
                "ui_template_pane_data_sources",
                &self.ui_template_pane_data_sources.len(),
            )
            .field("asset_importers", &self.asset_importers.len())
            .field(
                "asset_type_contributions",
                &self.asset_type_contributions.len(),
            )
            .field("settings_pages", &self.settings_pages.len())
            .field("scene_modes", &self.scene_modes.len())
            .field(
                "viewport_overlay_providers",
                &self.viewport_overlay_providers.len(),
            )
            .field("graph_editors", &self.graph_editors.len())
            .field("graph_node_palettes", &self.graph_node_palettes.len())
            .field("timeline_editors", &self.timeline_editors.len())
            .field("timeline_track_types", &self.timeline_track_types.len())
            .field("commands", &self.commands.len())
            .field("operation_factories", &self.operation_factories.len())
            .field("required_capabilities", &self.required_capabilities)
            .finish()
    }
}

impl ContributionBatch {
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

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
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
                view.bind_ui_template_id(view.id().to_owned());
            }
        }
    }

    pub fn register_view(
        &mut self,
        descriptor: ViewDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        descriptor
            .open_operation_path()
            .map_err(EditorExtensionRegistryError::OperationPath)?;
        insert_unique(
            &mut self.views,
            descriptor.id().to_owned(),
            descriptor,
            "view",
        )
    }

    pub fn register_drawer(
        &mut self,
        descriptor: DrawerDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_id("drawer", descriptor.id())?;
        insert_unique(
            &mut self.drawers,
            descriptor.id().to_owned(),
            descriptor,
            "drawer",
        )
    }

    pub fn register_menu_item(
        &mut self,
        descriptor: EditorMenuItemDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        let mut segment_count = 0;
        let valid = descriptor.path().split('/').all(|segment| {
            segment_count += 1;
            !segment.trim().is_empty() && segment.trim() == segment
        });
        if !valid || segment_count < 2 {
            return Err(EditorExtensionRegistryError::InvalidMenuPath(
                descriptor.path().to_owned(),
            ));
        }
        insert_unique(
            &mut self.menu_items,
            descriptor.path().to_owned(),
            descriptor,
            "menu item",
        )
    }

    pub fn register_inspector_customization(
        &mut self,
        customization: Arc<dyn InspectorCustomization>,
    ) -> Result<(), EditorExtensionRegistryError> {
        let id = customization.id().to_string();
        customization.validate().map_err(|error| {
            EditorExtensionRegistryError::View(format!(
                "inspector customization `{id}` is invalid: {error}"
            ))
        })?;
        validate_id("inspector customization", &id)?;
        if let Some(surface) = customization.surface() {
            surface.validate().map_err(|error| {
                EditorExtensionRegistryError::View(format!(
                    "inspector customization `{id}` has an invalid surface: {error}"
                ))
            })?;
        }
        insert_unique(
            &mut self.inspector_customizations,
            id,
            customization,
            "inspector customization",
        )
    }

    pub fn register_field_editor(
        &mut self,
        definition: FieldEditorDefinition,
    ) -> Result<(), EditorExtensionRegistryError> {
        let type_name = definition.type_name().to_owned();
        definition.validate().map_err(|error| {
            EditorExtensionRegistryError::View(format!(
                "field editor `{type_name}` is invalid: {error}"
            ))
        })?;
        insert_unique(
            &mut self.field_editors,
            type_name,
            definition,
            "field editor",
        )
    }

    pub fn register_ui_template(
        &mut self,
        descriptor: EditorUiTemplateDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_id("ui template", descriptor.id())?;
        validate_zui_document("ui template document", descriptor.ui_document())?;
        insert_unique(
            &mut self.ui_templates,
            descriptor.id().to_owned(),
            descriptor,
            "ui template",
        )
    }

    pub fn register_ui_template_pane_data_source(
        &mut self,
        template_id: impl Into<String>,
        source: Arc<dyn EditorUiTemplatePaneDataSource>,
    ) -> Result<(), EditorExtensionRegistryError> {
        let template_id = template_id.into();
        validate_id("ui template pane data source", &template_id)?;
        if !self.ui_templates.contains_key(&template_id) {
            return Err(EditorExtensionRegistryError::MissingUiTemplate { template_id });
        }
        insert_unique(
            &mut self.ui_template_pane_data_sources,
            template_id,
            source,
            "ui template pane data source",
        )
    }

    pub(crate) fn replace_ui_template_contributions(
        &mut self,
        templates: impl IntoIterator<Item = EditorUiTemplateDescriptor>,
        pane_data_sources: BTreeMap<String, Arc<dyn EditorUiTemplatePaneDataSource>>,
    ) -> Result<(), EditorExtensionRegistryError> {
        let mut candidate_templates = BTreeMap::new();
        for mut descriptor in templates {
            descriptor.inherit_plugin_root_from(self.ui_templates.get(descriptor.id()));
            validate_id("ui template", descriptor.id())?;
            validate_zui_document("ui template document", descriptor.ui_document())?;
            insert_unique(
                &mut candidate_templates,
                descriptor.id().to_owned(),
                descriptor,
                "ui template",
            )?;
        }
        for view in self.views.values() {
            if let Some(template_id) = view.ui_template_id() {
                if !candidate_templates.contains_key(template_id) {
                    return Err(EditorExtensionRegistryError::MissingUiTemplate {
                        template_id: template_id.to_owned(),
                    });
                }
            }
        }

        let mut candidate_sources = BTreeMap::new();
        for (template_id, source) in pane_data_sources {
            validate_id("ui template pane data source", &template_id)?;
            if !candidate_templates.contains_key(&template_id) {
                return Err(EditorExtensionRegistryError::MissingUiTemplate { template_id });
            }
            insert_unique(
                &mut candidate_sources,
                template_id,
                source,
                "ui template pane data source",
            )?;
        }

        self.ui_templates = candidate_templates;
        self.ui_template_pane_data_sources = candidate_sources;
        self.bind_matching_ui_templates_to_views();
        Ok(())
    }

    pub fn register_asset_importer(
        &mut self,
        descriptor: AssetImporterDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_id("asset importer", descriptor.id())?;
        if descriptor.source_extensions().is_empty() {
            return Err(
                EditorExtensionRegistryError::InvalidAssetImporterExtensions(
                    descriptor.id().to_owned(),
                ),
            );
        }
        insert_unique(
            &mut self.asset_importers,
            descriptor.id().to_owned(),
            descriptor,
            "asset importer",
        )
    }

    pub fn register_asset_type_contribution(
        &mut self,
        contribution: AssetTypeContribution,
    ) -> Result<(), EditorExtensionRegistryError> {
        let asset_type = contribution.asset_type().clone();
        insert_unique(
            &mut self.asset_type_contributions,
            asset_type,
            contribution,
            "asset type contribution",
        )
    }

    pub fn register_settings_page(
        &mut self,
        descriptor: SettingsPageDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_id("settings page", descriptor.id())?;
        if !descriptor.is_valid_category_path() {
            return Err(EditorExtensionRegistryError::InvalidContributionId {
                kind: "settings page category",
                id: descriptor.category_path().to_owned(),
            });
        }
        insert_unique(
            &mut self.settings_pages,
            descriptor.id().to_owned(),
            descriptor,
            "settings page",
        )
    }

    pub fn register_scene_mode(
        &mut self,
        registration: SceneModeRegistration,
    ) -> Result<(), EditorExtensionRegistryError> {
        let descriptor = registration.descriptor();
        validate_id("scene mode", descriptor.id())?;
        validate_id("scene mode view", descriptor.view_id())?;
        if let Some(provider_id) = descriptor.overlay_provider_id() {
            validate_id("viewport overlay provider", provider_id)?;
        }
        insert_unique(
            &mut self.scene_modes,
            descriptor.id().to_owned(),
            registration,
            "scene mode",
        )
    }

    pub fn register_viewport_overlay_provider(
        &mut self,
        registration: ViewportOverlayProviderRegistration,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_id("viewport overlay provider", registration.provider_id())?;
        insert_unique(
            &mut self.viewport_overlay_providers,
            registration.provider_id().to_owned(),
            registration,
            "viewport overlay provider",
        )
    }

    pub fn register_graph_editor(
        &mut self,
        descriptor: GraphEditorDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_id("graph editor view", descriptor.view_id())?;
        insert_unique(
            &mut self.graph_editors,
            descriptor.asset_type().clone(),
            descriptor,
            "graph editor",
        )
    }

    pub fn register_graph_node_palette(
        &mut self,
        descriptor: GraphNodePaletteDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_id("graph node palette", descriptor.id())?;
        if descriptor.nodes().is_empty() {
            return Err(EditorExtensionRegistryError::View(format!(
                "editor graph node palette `{}` must declare at least one node",
                descriptor.id()
            )));
        }
        let mut node_ids = std::collections::BTreeSet::new();
        for node in descriptor.nodes() {
            validate_id("graph node", node.id())?;
            if !node_ids.insert(node.id()) {
                return Err(EditorExtensionRegistryError::DuplicateContribution {
                    kind: "graph node",
                    id: node.id().to_owned(),
                });
            }
        }
        insert_unique(
            &mut self.graph_node_palettes,
            descriptor.id().to_owned(),
            descriptor,
            "graph node palette",
        )
    }

    pub fn register_timeline_editor(
        &mut self,
        descriptor: TimelineEditorDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_id("timeline editor view", descriptor.view_id())?;
        insert_unique(
            &mut self.timeline_editors,
            descriptor.asset_type().clone(),
            descriptor,
            "timeline editor",
        )
    }

    pub fn register_timeline_track_type(
        &mut self,
        descriptor: TimelineTrackDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_id("timeline track type", descriptor.id())?;
        validate_id("timeline track value kind", descriptor.value_kind())?;
        insert_unique(
            &mut self.timeline_track_types,
            descriptor.id().to_owned(),
            descriptor,
            "timeline track type",
        )
    }

    pub fn register_command(
        &mut self,
        descriptor: EditorCommandDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        insert_unique(
            &mut self.commands,
            descriptor.id().clone(),
            descriptor,
            "command",
        )
    }

    pub fn register_operation_command(
        &mut self,
        descriptor: EditorCommandDescriptor,
        factory: OperationCommandFactoryRegistration,
    ) -> Result<(), EditorExtensionRegistryError> {
        if descriptor.id() != factory.operation() {
            return Err(EditorExtensionRegistryError::Command(
                EditorCommandRegistryError::OperationFactory(
                    crate::core::editing::operation::OperationCommandFactoryError::OperationMismatch {
                        descriptor_operation: descriptor.id().clone(),
                        factory_operation: factory.operation().clone(),
                    },
                ),
            ));
        }
        let operation = descriptor.id().clone();
        if self.commands.contains_key(&operation)
            || self.operation_factories.contains_key(&operation)
        {
            return Err(EditorExtensionRegistryError::DuplicateContribution {
                kind: "command",
                id: operation.to_string(),
            });
        }
        self.commands.insert(operation.clone(), descriptor);
        self.operation_factories.insert(operation, factory);
        Ok(())
    }

    pub fn views(&self) -> impl Iterator<Item = &ViewDescriptor> {
        self.views.values()
    }

    pub fn drawers(&self) -> impl Iterator<Item = &DrawerDescriptor> {
        self.drawers.values()
    }

    pub fn menu_items(&self) -> impl Iterator<Item = &EditorMenuItemDescriptor> {
        self.menu_items.values()
    }

    pub fn inspector_customizations(
        &self,
    ) -> impl Iterator<Item = Arc<dyn InspectorCustomization>> + '_ {
        self.inspector_customizations.values().map(Arc::clone)
    }

    pub fn field_editors(&self) -> impl Iterator<Item = &FieldEditorDefinition> {
        self.field_editors.values()
    }

    pub fn ui_templates(&self) -> impl Iterator<Item = &EditorUiTemplateDescriptor> {
        self.ui_templates.values()
    }

    pub fn ui_template_pane_data_sources(
        &self,
    ) -> BTreeMap<String, Arc<dyn EditorUiTemplatePaneDataSource>> {
        self.ui_template_pane_data_sources
            .iter()
            .map(|(template_id, source)| (template_id.clone(), Arc::clone(source)))
            .collect()
    }

    pub fn asset_importers(&self) -> impl Iterator<Item = &AssetImporterDescriptor> {
        self.asset_importers.values()
    }

    pub fn asset_type_contributions(&self) -> impl Iterator<Item = &AssetTypeContribution> {
        self.asset_type_contributions.values()
    }

    pub fn settings_pages(&self) -> impl Iterator<Item = &SettingsPageDescriptor> {
        self.settings_pages.values()
    }

    pub fn scene_mode_descriptors(&self) -> impl Iterator<Item = &SceneModeDescriptor> {
        self.scene_modes
            .values()
            .map(SceneModeRegistration::descriptor)
    }

    pub fn scene_mode_registrations(&self) -> impl Iterator<Item = &SceneModeRegistration> {
        self.scene_modes.values()
    }

    pub fn viewport_overlay_providers(
        &self,
    ) -> impl Iterator<Item = &ViewportOverlayProviderRegistration> {
        self.viewport_overlay_providers.values()
    }

    pub(crate) fn has_viewport_overlay_provider(&self, provider_id: &str) -> bool {
        self.viewport_overlay_providers.contains_key(provider_id)
    }

    pub fn graph_editors(&self) -> impl Iterator<Item = &GraphEditorDescriptor> {
        self.graph_editors.values()
    }

    pub fn graph_node_palettes(&self) -> impl Iterator<Item = &GraphNodePaletteDescriptor> {
        self.graph_node_palettes.values()
    }

    pub fn timeline_editors(&self) -> impl Iterator<Item = &TimelineEditorDescriptor> {
        self.timeline_editors.values()
    }

    pub fn timeline_track_types(&self) -> impl Iterator<Item = &TimelineTrackDescriptor> {
        self.timeline_track_types.values()
    }

    pub fn pending_commands(&self) -> impl Iterator<Item = &EditorCommandDescriptor> {
        self.commands.values()
    }

    pub fn operation_factory(
        &self,
        operation: &EditorOperationPath,
    ) -> Option<&OperationCommandFactoryRegistration> {
        self.operation_factories.get(operation)
    }

    pub(crate) fn take_scene_modes(&mut self) -> Vec<SceneModeRegistration> {
        std::mem::take(&mut self.scene_modes)
            .into_values()
            .collect()
    }

    pub(crate) fn take_viewport_overlay_providers(
        &mut self,
    ) -> Vec<ViewportOverlayProviderRegistration> {
        std::mem::take(&mut self.viewport_overlay_providers)
            .into_values()
            .collect()
    }

    pub(crate) fn take_command_contributions(&mut self) -> Vec<EditorCommandDescriptor> {
        std::mem::take(&mut self.commands).into_values().collect()
    }

    pub(crate) fn take_operation_factories(&mut self) -> Vec<OperationCommandFactoryRegistration> {
        std::mem::take(&mut self.operation_factories)
            .into_values()
            .collect()
    }
}

fn validate_id(kind: &'static str, id: &str) -> Result<(), EditorExtensionRegistryError> {
    if id.trim().is_empty() || id.trim() != id {
        return Err(EditorExtensionRegistryError::InvalidContributionId {
            kind,
            id: id.to_owned(),
        });
    }
    Ok(())
}

fn validate_zui_document(
    kind: &'static str,
    document: &str,
) -> Result<(), EditorExtensionRegistryError> {
    if document.trim().is_empty() || document.trim() != document || !document.ends_with(".zui") {
        return Err(EditorExtensionRegistryError::InvalidUiDocument {
            kind,
            document: document.to_owned(),
        });
    }
    Ok(())
}

fn insert_unique<K, V>(
    entries: &mut BTreeMap<K, V>,
    id: K,
    value: V,
    kind: &'static str,
) -> Result<(), EditorExtensionRegistryError>
where
    K: Ord + ToString,
{
    match entries.entry(id) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(value);
            Ok(())
        }
        std::collections::btree_map::Entry::Occupied(entry) => {
            Err(EditorExtensionRegistryError::DuplicateContribution {
                kind,
                id: entry.key().to_string(),
            })
        }
    }
}
