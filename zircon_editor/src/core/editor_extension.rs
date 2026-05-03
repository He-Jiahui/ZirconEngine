use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::core::editor_authoring_extension::{
    AssetCreationTemplateDescriptor, GraphEditorDescriptor, GraphNodePaletteDescriptor,
    TimelineEditorDescriptor, TimelineTrackDescriptor, ViewportToolModeDescriptor,
};
use crate::core::editor_operation::{
    EditorOperationDescriptor, EditorOperationPath, EditorOperationRegistry,
    EditorOperationRegistryError,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorExtensionRegistry {
    views: BTreeMap<String, ViewDescriptor>,
    drawers: BTreeMap<String, DrawerDescriptor>,
    menu_items: BTreeMap<String, EditorMenuItemDescriptor>,
    component_drawers: BTreeMap<String, ComponentDrawerDescriptor>,
    ui_templates: BTreeMap<String, EditorUiTemplateDescriptor>,
    asset_importers: BTreeMap<String, AssetImporterDescriptor>,
    asset_editors: BTreeMap<String, AssetEditorDescriptor>,
    asset_creation_templates: BTreeMap<String, AssetCreationTemplateDescriptor>,
    viewport_tool_modes: BTreeMap<String, ViewportToolModeDescriptor>,
    graph_editors: BTreeMap<String, GraphEditorDescriptor>,
    graph_node_palettes: BTreeMap<String, GraphNodePaletteDescriptor>,
    timeline_editors: BTreeMap<String, TimelineEditorDescriptor>,
    timeline_track_types: BTreeMap<String, TimelineTrackDescriptor>,
    operations: EditorOperationRegistry,
}

impl EditorExtensionRegistry {
    pub fn register_view(
        &mut self,
        descriptor: ViewDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        descriptor
            .open_operation_path()
            .map_err(EditorExtensionRegistryError::Operation)?;
        insert_unique(&mut self.views, descriptor.id.clone(), descriptor, "view")
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
        validate_component_drawer_bindings(&descriptor)?;
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

    pub fn register_asset_editor(
        &mut self,
        descriptor: AssetEditorDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_contribution_id("asset editor", descriptor.asset_kind())?;
        insert_unique(
            &mut self.asset_editors,
            descriptor.asset_kind.clone(),
            descriptor,
            "asset editor",
        )
    }

    pub fn register_asset_creation_template(
        &mut self,
        descriptor: AssetCreationTemplateDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_contribution_id("asset creation template", descriptor.id())?;
        validate_contribution_id("asset kind", descriptor.asset_kind())?;
        insert_unique(
            &mut self.asset_creation_templates,
            descriptor.id().to_string(),
            descriptor,
            "asset creation template",
        )
    }

    pub fn register_viewport_tool_mode(
        &mut self,
        descriptor: ViewportToolModeDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        validate_contribution_id("viewport tool mode", descriptor.id())?;
        validate_contribution_id("viewport tool view", descriptor.view_id())?;
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
        validate_contribution_id("graph editor asset kind", descriptor.asset_kind())?;
        validate_contribution_id("graph editor view", descriptor.view_id())?;
        insert_unique(
            &mut self.graph_editors,
            descriptor.asset_kind().to_string(),
            descriptor,
            "graph editor",
        )
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
        validate_contribution_id("timeline editor asset kind", descriptor.asset_kind())?;
        validate_contribution_id("timeline editor view", descriptor.view_id())?;
        insert_unique(
            &mut self.timeline_editors,
            descriptor.asset_kind().to_string(),
            descriptor,
            "timeline editor",
        )
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

    pub fn register_operation(
        &mut self,
        descriptor: EditorOperationDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        self.operations
            .register(descriptor)
            .map_err(EditorExtensionRegistryError::Operation)
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

    pub fn asset_editors(&self) -> Vec<&AssetEditorDescriptor> {
        self.asset_editors.values().collect()
    }

    pub fn asset_creation_templates(&self) -> Vec<&AssetCreationTemplateDescriptor> {
        self.asset_creation_templates.values().collect()
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

    pub fn operations(&self) -> &EditorOperationRegistry {
        &self.operations
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorExtensionRegistration {
    registry: EditorExtensionRegistry,
    required_capabilities: Vec<String>,
}

impl EditorExtensionRegistration {
    pub fn new(registry: EditorExtensionRegistry) -> Self {
        Self {
            registry,
            required_capabilities: Vec::new(),
        }
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
pub struct ViewDescriptor {
    id: String,
    display_name: String,
    category: String,
}

impl ViewDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            category: category.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn open_operation_path(&self) -> Result<EditorOperationPath, EditorOperationRegistryError> {
        EditorOperationPath::parse(format!("View.{}.Open", self.id))
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

fn validate_component_drawer_bindings(
    descriptor: &ComponentDrawerDescriptor,
) -> Result<(), EditorExtensionRegistryError> {
    if let Some(template_id) = descriptor.template_id() {
        validate_contribution_id("component drawer template", template_id)?;
    }
    if let Some(data_root) = descriptor.data_root() {
        validate_contribution_id("component drawer data root", data_root)?;
    }
    for binding in descriptor.bindings() {
        EditorOperationPath::parse(binding.clone())
            .map_err(EditorExtensionRegistryError::Operation)?;
    }
    Ok(())
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
    output_kind: Option<String>,
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
            output_kind: None,
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

    pub fn with_output_kind(mut self, output_kind: impl Into<String>) -> Self {
        self.output_kind = Some(output_kind.into());
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

    pub fn output_kind(&self) -> Option<&str> {
        self.output_kind.as_deref()
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
    validate_contribution_id("graph node palette asset kind", descriptor.asset_kind())?;
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetEditorDescriptor {
    asset_kind: String,
    view_id: String,
    display_name: String,
    operation: EditorOperationPath,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl AssetEditorDescriptor {
    pub fn new(
        asset_kind: impl Into<String>,
        view_id: impl Into<String>,
        display_name: impl Into<String>,
        operation: EditorOperationPath,
    ) -> Self {
        Self {
            asset_kind: asset_kind.into(),
            view_id: view_id.into(),
            display_name: display_name.into(),
            operation,
            required_capabilities: Vec::new(),
        }
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

    pub fn asset_kind(&self) -> &str {
        &self.asset_kind
    }

    pub fn view_id(&self) -> &str {
        &self.view_id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn operation(&self) -> &EditorOperationPath {
        &self.operation
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorExtensionRegistryError {
    DuplicateContribution { kind: &'static str, id: String },
    InvalidContributionId { kind: &'static str, id: String },
    InvalidAssetImporterExtensions(String),
    InvalidMenuPath(String),
    Operation(EditorOperationRegistryError),
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
            Self::InvalidAssetImporterExtensions(id) => {
                write!(
                    formatter,
                    "editor asset importer `{id}` must declare at least one source extension"
                )
            }
            Self::InvalidMenuPath(path) => {
                write!(formatter, "editor menu item path `{path}` is invalid")
            }
            Self::Operation(error) => write!(formatter, "{error}"),
            Self::View(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for EditorExtensionRegistryError {}

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
