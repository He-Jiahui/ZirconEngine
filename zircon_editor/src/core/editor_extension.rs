use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::core::asset::{
    AssetTypeContribution, AssetTypeId, AssetTypeIdError, AssetTypeRegistryError,
};
use crate::core::commands::{
    EditorCommandContributionSet, EditorCommandDescriptor, EditorCommandRegistryError,
};
use crate::core::editing::operation::OperationCommandFactoryRegistration;
use crate::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodePaletteDescriptor, TimelineEditorDescriptor,
    TimelineTrackDescriptor, ViewportToolModeDescriptor,
};
use crate::core::editor_operation::{EditorOperationPath, EditorOperationPathError};

mod view_descriptor;

pub use view_descriptor::ViewDescriptor;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorExtensionRegistry {
    views: BTreeMap<String, ViewDescriptor>,
    drawers: BTreeMap<String, DrawerDescriptor>,
    menu_items: BTreeMap<String, EditorMenuItemDescriptor>,
    component_drawers: BTreeMap<String, ComponentDrawerDescriptor>,
    ui_templates: BTreeMap<String, EditorUiTemplateDescriptor>,
    asset_importers: BTreeMap<String, AssetImporterDescriptor>,
    asset_type_contributions: BTreeMap<AssetTypeId, AssetTypeContribution>,
    viewport_tool_modes: BTreeMap<String, ViewportToolModeDescriptor>,
    graph_editors: BTreeMap<AssetTypeId, GraphEditorDescriptor>,
    graph_node_palettes: BTreeMap<String, GraphNodePaletteDescriptor>,
    timeline_editors: BTreeMap<AssetTypeId, TimelineEditorDescriptor>,
    timeline_track_types: BTreeMap<String, TimelineTrackDescriptor>,
    command_contributions: EditorCommandContributionSet,
}

impl EditorExtensionRegistry {
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
            descriptor.id.clone(),
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
            descriptor.path.clone(),
            descriptor,
            "menu item",
        )
    }

    pub fn register_component_drawer(
        &mut self,
        descriptor: ComponentDrawerDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_component_drawer(&descriptor)?;
        insert_unique(
            &mut self.component_drawers,
            descriptor.component_type.clone(),
            descriptor,
            "component drawer",
        )
    }

    pub fn register_ui_template(
        &mut self,
        descriptor: EditorUiTemplateDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_ui_template_document("ui template document", descriptor.ui_document())?;
        insert_unique(
            &mut self.ui_templates,
            descriptor.id.clone(),
            descriptor,
            "ui template",
        )
    }

    pub fn register_asset_importer(
        &mut self,
        descriptor: AssetImporterDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_asset_importer(&descriptor)?;
        insert_unique(
            &mut self.asset_importers,
            descriptor.id.clone(),
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

    pub fn register_viewport_tool_mode(
        &mut self,
        descriptor: ViewportToolModeDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_contribution_id("viewport tool mode", descriptor.id())?;
        validate_contribution_id("viewport tool view", descriptor.view_id())?;
        if let Some(provider_id) = descriptor.overlay_provider_id() {
            validate_contribution_id("viewport overlay provider", provider_id)?;
        }
        insert_unique(
            &mut self.viewport_tool_modes,
            descriptor.id().to_string(),
            descriptor,
            "viewport tool mode",
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

    pub fn views(&self) -> Vec<&ViewDescriptor> {
        self.views.values().collect()
    }

    pub fn drawers(&self) -> Vec<&DrawerDescriptor> {
        self.drawers.values().collect()
    }

    pub fn menu_items(&self) -> Vec<&EditorMenuItemDescriptor> {
        self.menu_items.values().collect()
    }

    pub fn component_drawers(&self) -> Vec<&ComponentDrawerDescriptor> {
        self.component_drawers.values().collect()
    }

    pub fn ui_templates(&self) -> Vec<&EditorUiTemplateDescriptor> {
        self.ui_templates.values().collect()
    }

    pub fn asset_importers(&self) -> Vec<&AssetImporterDescriptor> {
        self.asset_importers.values().collect()
    }

    pub fn asset_type_contributions(&self) -> Vec<&AssetTypeContribution> {
        self.asset_type_contributions.values().collect()
    }

    pub fn viewport_tool_modes(&self) -> Vec<&ViewportToolModeDescriptor> {
        self.viewport_tool_modes.values().collect()
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawerDescriptor {
    id: String,
    display_name: String,
}

impl DrawerDescriptor {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMenuItemDescriptor {
    path: String,
    operation: EditorOperationPath,
    #[serde(default)]
    priority: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shortcut: Option<String>,
    #[serde(default = "default_menu_item_enabled")]
    enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl EditorMenuItemDescriptor {
    pub fn new(path: impl Into<String>, operation: EditorOperationPath) -> Self {
        Self {
            path: path.into(),
            operation,
            priority: 0,
            shortcut: None,
            enabled: true,
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_capabilities
            .extend(capabilities.into_iter().map(Into::into));
        self.required_capabilities.sort();
        self.required_capabilities.dedup();
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn operation(&self) -> &EditorOperationPath {
        &self.operation
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn shortcut(&self) -> Option<&str> {
        self.shortcut.as_deref()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

fn default_menu_item_enabled() -> bool {
    true
}

fn validate_menu_item_path(
    descriptor: &EditorMenuItemDescriptor,
) -> Result<(), EditorExtensionRegistryError> {
    let segments = descriptor.path.split('/').collect::<Vec<_>>();
    if segments.len() < MIN_MENU_PATH_SEGMENTS
        || segments
            .iter()
            .any(|segment| segment.trim().is_empty() || segment.trim() != *segment)
    {
        return Err(EditorExtensionRegistryError::InvalidMenuPath(
            descriptor.path.clone(),
        ));
    }
    Ok(())
}

const MIN_MENU_PATH_SEGMENTS: usize = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentDrawerDescriptor {
    component_type: String,
    ui_document: String,
    controller: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    template_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    data_root: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    bindings: Vec<String>,
}

impl ComponentDrawerDescriptor {
    pub fn new(
        component_type: impl Into<String>,
        ui_document: impl Into<String>,
        controller: impl Into<String>,
    ) -> Self {
        Self {
            component_type: component_type.into(),
            ui_document: ui_document.into(),
            controller: controller.into(),
            template_id: None,
            data_root: None,
            bindings: Vec::new(),
        }
    }

    pub fn with_template_id(mut self, template_id: impl Into<String>) -> Self {
        self.template_id = Some(template_id.into());
        self
    }

    pub fn with_data_root(mut self, data_root: impl Into<String>) -> Self {
        self.data_root = Some(data_root.into());
        self
    }

    pub fn with_binding(mut self, binding: impl Into<String>) -> Self {
        self.bindings.push(binding.into());
        self
    }

    pub fn component_type(&self) -> &str {
        &self.component_type
    }

    pub fn ui_document(&self) -> &str {
        &self.ui_document
    }

    pub fn controller(&self) -> &str {
        &self.controller
    }

    pub fn template_id(&self) -> Option<&str> {
        self.template_id.as_deref()
    }

    pub fn data_root(&self) -> Option<&str> {
        self.data_root.as_deref()
    }

    pub fn bindings(&self) -> &[String] {
        &self.bindings
    }
}

fn validate_component_drawer(
    descriptor: &ComponentDrawerDescriptor,
) -> Result<(), EditorExtensionRegistryError> {
    validate_zui_component_document("component drawer document", descriptor.ui_document())?;
    if let Some(template_id) = descriptor.template_id() {
        validate_contribution_id("component drawer template", template_id)?;
    }
    if let Some(data_root) = descriptor.data_root() {
        validate_contribution_id("component drawer data root", data_root)?;
    }
    for binding in descriptor.bindings() {
        EditorOperationPath::parse(binding.clone())
            .map_err(EditorExtensionRegistryError::OperationPath)?;
    }
    Ok(())
}

fn validate_ui_template_document(
    kind: &'static str,
    document: &str,
) -> Result<(), EditorExtensionRegistryError> {
    if is_invalid_ui_document(document) || !document.ends_with(".zui") {
        return Err(EditorExtensionRegistryError::InvalidUiDocument {
            kind,
            document: document.to_string(),
        });
    }
    Ok(())
}

fn validate_zui_component_document(
    kind: &'static str,
    document: &str,
) -> Result<(), EditorExtensionRegistryError> {
    if is_invalid_ui_document(document) || !document.ends_with(".zui") {
        return Err(EditorExtensionRegistryError::InvalidUiDocument {
            kind,
            document: document.to_string(),
        });
    }
    Ok(())
}

fn is_invalid_ui_document(document: &str) -> bool {
    document.trim().is_empty() || document.trim() != document
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorUiTemplateDescriptor {
    id: String,
    ui_document: String,
}

impl EditorUiTemplateDescriptor {
    pub fn new(id: impl Into<String>, ui_document: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ui_document: ui_document.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn ui_document(&self) -> &str {
        &self.ui_document
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetImporterDescriptor {
    id: String,
    display_name: String,
    operation: EditorOperationPath,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    source_extensions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    output_type: Option<AssetTypeId>,
    #[serde(default)]
    priority: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl AssetImporterDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        operation: EditorOperationPath,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            operation,
            source_extensions: Vec::new(),
            output_type: None,
            priority: 0,
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_source_extension(mut self, extension: impl AsRef<str>) -> Self {
        push_normalized_extension(&mut self.source_extensions, extension.as_ref());
        self
    }

    pub fn with_source_extensions<I, S>(mut self, extensions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for extension in extensions {
            push_normalized_extension(&mut self.source_extensions, extension.as_ref());
        }
        self
    }

    pub fn with_output_type(mut self, output_type: AssetTypeId) -> Self {
        self.output_type = Some(output_type);
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_capabilities
            .extend(capabilities.into_iter().map(Into::into));
        self.required_capabilities.sort();
        self.required_capabilities.dedup();
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn operation(&self) -> &EditorOperationPath {
        &self.operation
    }

    pub fn source_extensions(&self) -> &[String] {
        &self.source_extensions
    }

    pub fn output_type(&self) -> Option<&AssetTypeId> {
        self.output_type.as_ref()
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

fn push_normalized_extension(extensions: &mut Vec<String>, extension: &str) {
    let extension = extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    if !extension.is_empty() && !extensions.contains(&extension) {
        extensions.push(extension);
        extensions.sort();
    }
}

fn validate_asset_importer(
    descriptor: &AssetImporterDescriptor,
) -> Result<(), EditorExtensionRegistryError> {
    validate_contribution_id("asset importer", &descriptor.id)?;
    if descriptor.source_extensions.is_empty() {
        return Err(
            EditorExtensionRegistryError::InvalidAssetImporterExtensions(descriptor.id.clone()),
        );
    }
    Ok(())
}

fn validate_graph_node_palette(
    descriptor: &GraphNodePaletteDescriptor,
) -> Result<(), EditorExtensionRegistryError> {
    validate_contribution_id("graph node palette", descriptor.id())?;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorExtensionRegistryError {
    DuplicateContribution {
        kind: &'static str,
        id: String,
    },
    InvalidContributionId {
        kind: &'static str,
        id: String,
    },
    InvalidUiDocument {
        kind: &'static str,
        document: String,
    },
    InvalidAssetImporterExtensions(String),
    InvalidMenuPath(String),
    CommandViewTargetConflict {
        command_id: EditorOperationPath,
        view_id: String,
    },
    MenuCapabilitiesRequireContributedCommand {
        command_id: EditorOperationPath,
    },
    Command(EditorCommandRegistryError),
    OperationPath(EditorOperationPathError),
    AssetTypeId(AssetTypeIdError),
    AssetTypeRegistry(AssetTypeRegistryError),
    View(String),
}

impl fmt::Display for EditorExtensionRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateContribution { kind, id } => {
                write!(formatter, "editor {kind} {id} already registered")
            }
            Self::InvalidContributionId { kind, id } => {
                write!(formatter, "editor {kind} id `{id}` is invalid")
            }
            Self::InvalidUiDocument { kind, document } => write!(
                formatter,
                "editor {kind} `{document}` must reference a supported editor UI asset"
            ),
            Self::InvalidAssetImporterExtensions(id) => {
                write!(
                    formatter,
                    "editor asset importer `{id}` must declare at least one source extension"
                )
            }
            Self::InvalidMenuPath(path) => {
                write!(formatter, "editor menu item path `{path}` is invalid")
            }
            Self::CommandViewTargetConflict {
                command_id,
                view_id,
            } => write!(
                formatter,
                "editor command {command_id} does not open extension view {view_id}"
            ),
            Self::MenuCapabilitiesRequireContributedCommand { command_id } => write!(
                formatter,
                "editor menu capability constraints for {command_id} must be owned by a command contributed by the same extension"
            ),
            Self::Command(error) => write!(formatter, "{error}"),
            Self::OperationPath(error) => write!(formatter, "{error}"),
            Self::AssetTypeId(error) => write!(formatter, "{error}"),
            Self::AssetTypeRegistry(error) => write!(formatter, "{error}"),
            Self::View(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for EditorExtensionRegistryError {}

impl From<AssetTypeIdError> for EditorExtensionRegistryError {
    fn from(error: AssetTypeIdError) -> Self {
        Self::AssetTypeId(error)
    }
}

impl From<AssetTypeRegistryError> for EditorExtensionRegistryError {
    fn from(error: AssetTypeRegistryError) -> Self {
        Self::AssetTypeRegistry(error)
    }
}

fn insert_unique<T>(
    map: &mut BTreeMap<String, T>,
    id: String,
    descriptor: T,
    kind: &'static str,
) -> Result<(), EditorExtensionRegistryError> {
    if map.contains_key(&id) {
        return Err(EditorExtensionRegistryError::DuplicateContribution { kind, id });
    }
    map.insert(id, descriptor);
    Ok(())
}
