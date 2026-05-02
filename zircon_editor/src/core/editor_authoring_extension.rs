use serde::{Deserialize, Serialize};

use crate::core::editor_operation::EditorOperationPath;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetCreationTemplateDescriptor {
    id: String,
    display_name: String,
    asset_kind: String,
    operation: EditorOperationPath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default_document: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl AssetCreationTemplateDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        asset_kind: impl Into<String>,
        operation: EditorOperationPath,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            asset_kind: asset_kind.into(),
            operation,
            default_document: None,
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_default_document(mut self, document: impl Into<String>) -> Self {
        self.default_document = Some(document.into());
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        push_capabilities(&mut self.required_capabilities, capabilities);
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn asset_kind(&self) -> &str {
        &self.asset_kind
    }

    pub fn operation(&self) -> &EditorOperationPath {
        &self.operation
    }

    pub fn default_document(&self) -> Option<&str> {
        self.default_document.as_deref()
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewportToolModeDescriptor {
    id: String,
    display_name: String,
    view_id: String,
    activate_operation: EditorOperationPath,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl ViewportToolModeDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        view_id: impl Into<String>,
        activate_operation: EditorOperationPath,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            view_id: view_id.into(),
            activate_operation,
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        push_capabilities(&mut self.required_capabilities, capabilities);
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn view_id(&self) -> &str {
        &self.view_id
    }

    pub fn activate_operation(&self) -> &EditorOperationPath {
        &self.activate_operation
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPinDescriptor {
    name: String,
    value_type: String,
    #[serde(default)]
    required: bool,
}

impl GraphPinDescriptor {
    pub fn new(name: impl Into<String>, value_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value_type: value_type.into(),
            required: false,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value_type(&self) -> &str {
        &self.value_type
    }

    pub fn is_required(&self) -> bool {
        self.required
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodeDescriptor {
    id: String,
    display_name: String,
    category: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    inputs: Vec<GraphPinDescriptor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    outputs: Vec<GraphPinDescriptor>,
}

impl GraphNodeDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            category: category.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn with_input(mut self, pin: GraphPinDescriptor) -> Self {
        self.inputs.push(pin);
        self
    }

    pub fn with_output(mut self, pin: GraphPinDescriptor) -> Self {
        self.outputs.push(pin);
        self
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

    pub fn inputs(&self) -> &[GraphPinDescriptor] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[GraphPinDescriptor] {
        &self.outputs
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodePaletteDescriptor {
    id: String,
    asset_kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    nodes: Vec<GraphNodeDescriptor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl GraphNodePaletteDescriptor {
    pub fn new(id: impl Into<String>, asset_kind: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            asset_kind: asset_kind.into(),
            nodes: Vec::new(),
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_node(mut self, node: GraphNodeDescriptor) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        push_capabilities(&mut self.required_capabilities, capabilities);
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn asset_kind(&self) -> &str {
        &self.asset_kind
    }

    pub fn nodes(&self) -> &[GraphNodeDescriptor] {
        &self.nodes
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEditorDescriptor {
    asset_kind: String,
    view_id: String,
    display_name: String,
    open_operation: EditorOperationPath,
    validate_operation: EditorOperationPath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    compile_operation: Option<EditorOperationPath>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl GraphEditorDescriptor {
    pub fn new(
        asset_kind: impl Into<String>,
        view_id: impl Into<String>,
        display_name: impl Into<String>,
        open_operation: EditorOperationPath,
        validate_operation: EditorOperationPath,
    ) -> Self {
        Self {
            asset_kind: asset_kind.into(),
            view_id: view_id.into(),
            display_name: display_name.into(),
            open_operation,
            validate_operation,
            compile_operation: None,
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_compile_operation(mut self, operation: EditorOperationPath) -> Self {
        self.compile_operation = Some(operation);
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        push_capabilities(&mut self.required_capabilities, capabilities);
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

    pub fn open_operation(&self) -> &EditorOperationPath {
        &self.open_operation
    }

    pub fn validate_operation(&self) -> &EditorOperationPath {
        &self.validate_operation
    }

    pub fn compile_operation(&self) -> Option<&EditorOperationPath> {
        self.compile_operation.as_ref()
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineTrackDescriptor {
    id: String,
    display_name: String,
    value_kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl TimelineTrackDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        value_kind: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            value_kind: value_kind.into(),
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        push_capabilities(&mut self.required_capabilities, capabilities);
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn value_kind(&self) -> &str {
        &self.value_kind
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineEditorDescriptor {
    asset_kind: String,
    view_id: String,
    display_name: String,
    open_operation: EditorOperationPath,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    track_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl TimelineEditorDescriptor {
    pub fn new(
        asset_kind: impl Into<String>,
        view_id: impl Into<String>,
        display_name: impl Into<String>,
        open_operation: EditorOperationPath,
    ) -> Self {
        Self {
            asset_kind: asset_kind.into(),
            view_id: view_id.into(),
            display_name: display_name.into(),
            open_operation,
            track_types: Vec::new(),
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_track_type(mut self, track_type: impl Into<String>) -> Self {
        self.track_types.push(track_type.into());
        self.track_types.sort();
        self.track_types.dedup();
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        push_capabilities(&mut self.required_capabilities, capabilities);
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

    pub fn open_operation(&self) -> &EditorOperationPath {
        &self.open_operation
    }

    pub fn track_types(&self) -> &[String] {
        &self.track_types
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

fn push_capabilities<I, S>(target: &mut Vec<String>, capabilities: I)
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    target.extend(capabilities.into_iter().map(Into::into));
    target.sort();
    target.dedup();
}
