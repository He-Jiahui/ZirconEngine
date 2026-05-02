use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use zircon_runtime::scene::components::NodeKind;

use crate::core::editor_event::{EditorEvent, EditorEventSource, LayoutCommand, MenuAction};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EditorOperationPath(String);

impl EditorOperationPath {
    pub fn parse(value: impl Into<String>) -> Result<Self, EditorOperationRegistryError> {
        let value = value.into();
        let segments = value.split('.').collect::<Vec<_>>();
        let valid = segments
            .iter()
            .all(|segment| !segment.is_empty() && segment.chars().all(operation_path_char));
        if !valid || segments.len() < MIN_OPERATION_PATH_SEGMENTS {
            return Err(EditorOperationRegistryError::InvalidOperationPath(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EditorOperationPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

const MIN_OPERATION_PATH_SEGMENTS: usize = 3;

fn operation_path_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_' || value == '-'
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoableEditorOperation {
    display_name: String,
}

impl UndoableEditorOperation {
    pub fn new(display_name: impl Into<String>) -> Self {
        Self {
            display_name: display_name.into(),
        }
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorOperationDescriptor {
    path: EditorOperationPath,
    display_name: String,
    menu_path: Option<String>,
    callable_from_remote: bool,
    undoable: Option<UndoableEditorOperation>,
    event: Option<EditorEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl EditorOperationDescriptor {
    pub fn new(path: EditorOperationPath, display_name: impl Into<String>) -> Self {
        Self {
            path,
            display_name: display_name.into(),
            menu_path: None,
            callable_from_remote: true,
            undoable: None,
            event: None,
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_menu_path(mut self, menu_path: impl Into<String>) -> Self {
        self.menu_path = Some(menu_path.into());
        self
    }

    pub fn with_callable_from_remote(mut self, callable_from_remote: bool) -> Self {
        self.callable_from_remote = callable_from_remote;
        self
    }

    pub fn with_undoable(mut self, undoable: UndoableEditorOperation) -> Self {
        self.undoable = Some(undoable);
        self
    }

    pub fn with_event(mut self, event: EditorEvent) -> Self {
        self.event = Some(event);
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

    pub fn path(&self) -> &EditorOperationPath {
        &self.path
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn menu_path(&self) -> Option<&str> {
        self.menu_path.as_deref()
    }

    pub fn callable_from_remote(&self) -> bool {
        self.callable_from_remote
    }

    pub fn undoable(&self) -> Option<&UndoableEditorOperation> {
        self.undoable.as_ref()
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }

    pub(crate) fn event(&self) -> Option<&EditorEvent> {
        self.event.as_ref()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorOperationRegistry {
    descriptors: BTreeMap<EditorOperationPath, EditorOperationDescriptor>,
}

impl EditorOperationRegistry {
    pub fn with_builtin_operations() -> Self {
        let mut registry = Self::default();
        for descriptor in builtin_operation_descriptors() {
            registry
                .register(descriptor)
                .expect("built-in editor operation ids are unique");
        }
        registry
    }

    pub fn register(
        &mut self,
        descriptor: EditorOperationDescriptor,
    ) -> Result<(), EditorOperationRegistryError> {
        if self.descriptors.contains_key(descriptor.path()) {
            return Err(EditorOperationRegistryError::DuplicateOperation(
                descriptor.path().clone(),
            ));
        }
        validate_operation_menu_path(&descriptor)?;
        self.descriptors.insert(descriptor.path.clone(), descriptor);
        Ok(())
    }

    pub fn descriptor(&self, path: &EditorOperationPath) -> Option<&EditorOperationDescriptor> {
        self.descriptors.get(path)
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &EditorOperationDescriptor> {
        self.descriptors.values()
    }

    pub(crate) fn descriptor_for_event(
        &self,
        event: &EditorEvent,
    ) -> Option<&EditorOperationDescriptor> {
        self.descriptors
            .values()
            .find(|descriptor| descriptor.event() == Some(event))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorOperationInvocation {
    pub operation_id: EditorOperationPath,
    #[serde(default)]
    pub arguments: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_group: Option<String>,
}

impl EditorOperationInvocation {
    pub fn new(operation_id: EditorOperationPath) -> Self {
        Self {
            operation_id,
            arguments: Value::Null,
            operation_group: None,
        }
    }

    pub fn parse(operation_id: impl Into<String>) -> Result<Self, EditorOperationRegistryError> {
        Ok(Self::new(EditorOperationPath::parse(operation_id)?))
    }

    pub fn with_arguments(mut self, arguments: Value) -> Self {
        self.arguments = arguments;
        self
    }

    pub fn with_operation_group(mut self, group: impl Into<String>) -> Self {
        self.operation_group = Some(group.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorOperationSource {
    Menu,
    UiBinding,
    Remote,
    Cli,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorOperationControlRequest {
    InvokeOperation(EditorOperationInvocation),
    ListOperations,
    QueryOperationStack,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorOperationStack {
    undo_stack: Vec<EditorOperationStackEntry>,
    redo_stack: Vec<EditorOperationStackEntry>,
}

impl EditorOperationStack {
    pub fn record(&mut self, entry: EditorOperationStackEntry) {
        if let Some(group) = entry.operation_group.as_deref() {
            if let Some(last) = self.undo_stack.last_mut() {
                if last.operation_id == entry.operation_id
                    && last.operation_group.as_deref() == Some(group)
                {
                    *last = entry;
                    self.redo_stack.clear();
                    return;
                }
            }
        }
        self.undo_stack.push(entry);
        self.redo_stack.clear();
    }

    pub fn move_undo_to_redo(&mut self) -> bool {
        let Some(entry) = self.undo_stack.pop() else {
            return false;
        };
        self.redo_stack.push(entry);
        true
    }

    pub fn move_redo_to_undo(&mut self) -> bool {
        let Some(entry) = self.redo_stack.pop() else {
            return false;
        };
        self.undo_stack.push(entry);
        true
    }

    pub fn undo_stack(&self) -> &[EditorOperationStackEntry] {
        &self.undo_stack
    }

    pub fn redo_stack(&self) -> &[EditorOperationStackEntry] {
        &self.redo_stack
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorOperationStackEntry {
    pub operation_id: EditorOperationPath,
    pub display_name: String,
    pub source: EditorEventSource,
    pub sequence: u64,
    pub operation_group: Option<String>,
}

impl EditorOperationStackEntry {
    pub fn new(
        operation_id: EditorOperationPath,
        display_name: impl Into<String>,
        source: EditorEventSource,
        sequence: u64,
    ) -> Self {
        Self {
            operation_id,
            display_name: display_name.into(),
            source,
            sequence,
            operation_group: None,
        }
    }

    pub fn with_operation_group(mut self, group: Option<String>) -> Self {
        self.operation_group = group;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorOperationControlResponse {
    pub operation_id: Option<String>,
    pub value: Option<Value>,
    pub error: Option<String>,
}

impl EditorOperationControlResponse {
    pub fn success(operation_id: impl Into<String>, value: Option<Value>) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            value,
            error: None,
        }
    }

    pub fn failure(operation_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            value: None,
            error: Some(error.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorOperationRegistryError {
    DuplicateOperation(EditorOperationPath),
    InvalidOperationPath(String),
    InvalidOperationMenuPath(String),
    MissingOperation(EditorOperationPath),
    OperationNotCallableFromRemote(EditorOperationPath),
    OperationHasNoHandler(EditorOperationPath),
}

impl fmt::Display for EditorOperationRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateOperation(path) => {
                write!(formatter, "editor operation {path} already registered")
            }
            Self::InvalidOperationPath(path) => {
                write!(formatter, "editor operation path `{path}` is invalid")
            }
            Self::InvalidOperationMenuPath(path) => {
                write!(formatter, "editor operation menu path `{path}` is invalid")
            }
            Self::MissingOperation(path) => {
                write!(formatter, "editor operation {path} is not registered")
            }
            Self::OperationNotCallableFromRemote(path) => {
                write!(
                    formatter,
                    "editor operation {path} is not callable from remote control"
                )
            }
            Self::OperationHasNoHandler(path) => {
                write!(formatter, "editor operation {path} has no event handler")
            }
        }
    }
}

impl std::error::Error for EditorOperationRegistryError {}

fn validate_operation_menu_path(
    descriptor: &EditorOperationDescriptor,
) -> Result<(), EditorOperationRegistryError> {
    if let Some(menu_path) = descriptor.menu_path() {
        let segments = menu_path.split('/').collect::<Vec<_>>();
        if segments.len() < MIN_MENU_PATH_SEGMENTS
            || segments
                .iter()
                .any(|segment| segment.trim().is_empty() || segment.trim() != *segment)
        {
            return Err(EditorOperationRegistryError::InvalidOperationMenuPath(
                menu_path.to_string(),
            ));
        }
    }
    Ok(())
}

const MIN_MENU_PATH_SEGMENTS: usize = 2;

fn builtin_operation_descriptors() -> Vec<EditorOperationDescriptor> {
    vec![
        operation(
            "Window.Layout.Reset",
            "Reset Layout",
            "Window/Reset Layout",
            EditorEvent::WorkbenchMenu(MenuAction::ResetLayout),
        )
        .with_undoable(UndoableEditorOperation::new("Reset Layout")),
        operation(
            "Scene.Node.CreateCube",
            "Create Cube",
            "GameObject/3D Object/Cube",
            EditorEvent::WorkbenchMenu(MenuAction::CreateNode(NodeKind::Cube)),
        )
        .with_undoable(UndoableEditorOperation::new("Create Cube")),
        operation(
            "Scene.Node.DeleteSelected",
            "Delete Selected",
            "Edit/Delete",
            EditorEvent::WorkbenchMenu(MenuAction::DeleteSelected),
        )
        .with_undoable(UndoableEditorOperation::new("Delete Selected")),
        operation(
            "Edit.History.Undo",
            "Undo",
            "Edit/Undo",
            EditorEvent::WorkbenchMenu(MenuAction::Undo),
        ),
        operation(
            "Edit.History.Redo",
            "Redo",
            "Edit/Redo",
            EditorEvent::WorkbenchMenu(MenuAction::Redo),
        ),
        operation(
            "Window.Layout.Default",
            "Load Default Layout",
            "Window/Layout/Default",
            EditorEvent::Layout(LayoutCommand::ResetToDefault),
        ),
    ]
}

fn operation(
    path: &str,
    display_name: &str,
    menu_path: &str,
    event: EditorEvent,
) -> EditorOperationDescriptor {
    EditorOperationDescriptor::new(
        EditorOperationPath::parse(path).expect("valid built-in operation path"),
        display_name,
    )
    .with_menu_path(menu_path)
    .with_event(event)
}
