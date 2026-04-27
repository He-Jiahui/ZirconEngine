use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

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
    operations: EditorOperationRegistry,
}

impl EditorExtensionRegistry {
    pub fn register_view(
        &mut self,
        descriptor: ViewDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
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
            required_capabilities: Vec::new(),
        }
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentDrawerDescriptor {
    component_type: String,
    ui_document: String,
    controller: String,
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
            bindings: Vec::new(),
        }
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

    pub fn bindings(&self) -> &[String] {
        &self.bindings
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorExtensionRegistryError {
    DuplicateContribution { kind: &'static str, id: String },
    Operation(EditorOperationRegistryError),
    View(String),
}

impl fmt::Display for EditorExtensionRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateContribution { kind, id } => {
                write!(formatter, "editor {kind} {id} already registered")
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
