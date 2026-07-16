use serde::{Deserialize, Serialize};

use crate::core::editor_event::EditorEvent;
use crate::core::editor_operation::EditorOperationPath;

use super::{AssetWriteTargetDescriptor, CommandEvalCtx, EditorKeyChord, WhenClause};

/// Canonical metadata and executable route for every editor command.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorCommandDescriptor {
    id: EditorOperationPath,
    display_name: String,
    description: String,
    category: EditorCommandCategory,
    menu_path: Option<String>,
    menu_projection: EditorCommandMenuProjection,
    action: EditorCommandAction,
    default_chord: Option<EditorKeyChord>,
    #[serde(default)]
    when: WhenClause,
    keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    payload_schema_id: Option<String>,
    #[serde(default = "default_callable_from_remote")]
    callable_from_remote: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    asset_write_target: Option<AssetWriteTargetDescriptor>,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_capabilities"
    )]
    required_capabilities: Vec<String>,
}

impl EditorCommandDescriptor {
    pub fn new(
        id: EditorOperationPath,
        display_name: impl Into<String>,
        category: EditorCommandCategory,
        action: EditorCommandAction,
    ) -> Self {
        let display_name = display_name.into();
        Self {
            id,
            description: display_name.clone(),
            display_name,
            category,
            menu_path: None,
            menu_projection: EditorCommandMenuProjection::CommandRegistry,
            action,
            default_chord: None,
            when: WhenClause::Always,
            keywords: Vec::new(),
            payload_schema_id: None,
            callable_from_remote: true,
            asset_write_target: None,
            required_capabilities: Vec::new(),
        }
    }

    pub fn operation(id: EditorOperationPath, display_name: impl Into<String>) -> Self {
        Self::new(
            id,
            display_name,
            EditorCommandCategory::Command,
            EditorCommandAction::Operation,
        )
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_category(mut self, category: EditorCommandCategory) -> Self {
        self.category = category;
        self
    }

    pub fn with_menu_path(mut self, menu_path: impl Into<String>) -> Self {
        self.menu_path = Some(menu_path.into());
        self
    }

    pub fn with_menu_projection(mut self, projection: EditorCommandMenuProjection) -> Self {
        self.menu_projection = projection;
        self
    }

    pub fn with_default_chord(mut self, chord: EditorKeyChord) -> Self {
        self.default_chord = Some(chord);
        self
    }

    pub fn with_when(mut self, when: WhenClause) -> Self {
        self.when = when;
        self
    }

    pub fn with_keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self.keywords.sort();
        self.keywords.dedup();
        self
    }

    pub fn with_payload_schema_id(mut self, schema_id: impl Into<String>) -> Self {
        self.payload_schema_id = Some(schema_id.into());
        self
    }

    pub fn with_callable_from_remote(mut self, callable_from_remote: bool) -> Self {
        self.callable_from_remote = callable_from_remote;
        self
    }

    pub fn with_asset_write_target_arguments(
        mut self,
        asset_type_argument: impl Into<String>,
        locator_argument: impl Into<String>,
    ) -> Self {
        self.asset_write_target = Some(AssetWriteTargetDescriptor::new(
            asset_type_argument,
            locator_argument,
        ));
        self
    }

    pub fn with_event(mut self, event: EditorEvent) -> Self {
        self.action = EditorCommandAction::Emit(event);
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

    pub fn id(&self) -> &EditorOperationPath {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn category(&self) -> EditorCommandCategory {
        self.category
    }

    pub fn menu_path(&self) -> Option<&str> {
        self.menu_path.as_deref()
    }

    pub fn menu_projection(&self) -> EditorCommandMenuProjection {
        self.menu_projection
    }

    pub fn action(&self) -> &EditorCommandAction {
        &self.action
    }

    pub fn event(&self) -> Option<&EditorEvent> {
        match &self.action {
            EditorCommandAction::Emit(event) => Some(event),
            EditorCommandAction::Operation => None,
        }
    }

    pub fn default_chord(&self) -> Option<&EditorKeyChord> {
        self.default_chord.as_ref()
    }

    pub fn when(&self) -> &WhenClause {
        &self.when
    }

    pub fn effective_when(&self) -> WhenClause {
        WhenClause::all(
            std::iter::once(self.when.clone())
                .chain(
                    self.required_capabilities
                        .iter()
                        .cloned()
                        .map(WhenClause::Capability),
                )
                .chain(
                    self.asset_write_target
                        .as_ref()
                        .map(|_| WhenClause::AssetWritable),
                ),
        )
    }

    pub fn is_enabled(&self, context: &CommandEvalCtx) -> bool {
        self.effective_when().eval(context)
    }

    pub(crate) fn missing_required_capabilities(&self, context: &CommandEvalCtx) -> Vec<String> {
        self.required_capabilities
            .iter()
            .filter(|capability| !context.has_capability(capability))
            .cloned()
            .collect()
    }

    pub fn keywords(&self) -> &[String] {
        &self.keywords
    }

    pub fn payload_schema_id(&self) -> Option<&str> {
        self.payload_schema_id.as_deref()
    }

    pub fn callable_from_remote(&self) -> bool {
        self.callable_from_remote
    }

    pub fn asset_write_target(&self) -> Option<&AssetWriteTargetDescriptor> {
        self.asset_write_target.as_ref()
    }

    pub(super) fn set_asset_write_target(&mut self, target: AssetWriteTargetDescriptor) {
        self.asset_write_target = Some(target);
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

/// Selects the single owner that materializes a command's menu metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorCommandMenuProjection {
    CommandRegistry,
    ExtensionRegistry,
}

fn default_callable_from_remote() -> bool {
    true
}

fn deserialize_capabilities<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut capabilities = Vec::<String>::deserialize(deserializer)?;
    capabilities.sort();
    capabilities.dedup();
    Ok(capabilities)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorCommandCategory {
    File,
    Edit,
    Selection,
    Runtime,
    View,
    Window,
    Help,
    Command,
}

impl EditorCommandCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::Selection => "Selection",
            Self::Runtime => "Play",
            Self::View => "View",
            Self::Window => "Window",
            Self::Help => "Help",
            Self::Command => "Command",
        }
    }

    pub fn source_tag(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Edit => "edit",
            Self::Selection => "selection",
            Self::Runtime => "runtime",
            Self::View => "view",
            Self::Window => "window",
            Self::Help => "help",
            Self::Command => "command",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorCommandAction {
    Emit(EditorEvent),
    Operation,
}
