use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::dispatch::UiWindowId;

use crate::core::document::ProjectSessionId;
use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::DocumentId;

pub const MAX_TOOL_RESOURCE_IDENTIFIER_BYTES: usize = 128;

const VIEWPORT_INPUT_KIND: &str = "editor.viewport-input";
const MODAL_SURFACE_KIND: &str = "editor.modal-surface";
const SCENE_MODE_SLOT_KIND: &str = "editor.scene-mode-slot";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ToolResourceKindId(String);

impl ToolResourceKindId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ToolResourceIdError> {
        let value = value.into();
        validate_resource_identifier(&value)?;
        Ok(Self(value))
    }

    pub fn viewport_input() -> Self {
        Self(VIEWPORT_INPUT_KIND.to_string())
    }

    pub fn modal_surface() -> Self {
        Self(MODAL_SURFACE_KIND.to_string())
    }

    pub fn scene_mode_slot() -> Self {
        Self(SCENE_MODE_SLOT_KIND.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ToolResourceKindId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ToolResourceChannelId(String);

impl ToolResourceChannelId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ToolResourceIdError> {
        let value = value.into();
        validate_resource_identifier(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ToolResourceChannelId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolResourceIdError {
    Empty,
    TooLong {
        actual_bytes: usize,
        max_bytes: usize,
    },
    InvalidBoundary,
    InvalidCharacter {
        byte_index: usize,
        character: char,
    },
}

impl std::fmt::Display for ToolResourceIdError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(formatter, "a tool resource identifier cannot be empty"),
            Self::TooLong {
                actual_bytes,
                max_bytes,
            } => write!(
                formatter,
                "tool resource identifier uses {actual_bytes} bytes; maximum is {max_bytes}"
            ),
            Self::InvalidBoundary => write!(
                formatter,
                "a tool resource identifier must start and end with an ASCII letter or digit"
            ),
            Self::InvalidCharacter {
                byte_index,
                character,
            } => write!(
                formatter,
                "invalid tool resource identifier character {character:?} at byte {byte_index}"
            ),
        }
    }
}

impl std::error::Error for ToolResourceIdError {}

fn validate_resource_identifier(value: &str) -> Result<(), ToolResourceIdError> {
    if value.is_empty() {
        return Err(ToolResourceIdError::Empty);
    }
    if value.len() > MAX_TOOL_RESOURCE_IDENTIFIER_BYTES {
        return Err(ToolResourceIdError::TooLong {
            actual_bytes: value.len(),
            max_bytes: MAX_TOOL_RESOURCE_IDENTIFIER_BYTES,
        });
    }
    if !value
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_alphanumeric())
        || !value
            .as_bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
    {
        return Err(ToolResourceIdError::InvalidBoundary);
    }
    if let Some((byte_index, character)) = value.char_indices().find(|(_, character)| {
        !(character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '.' | '-' | '_'))
    }) {
        return Err(ToolResourceIdError::InvalidCharacter {
            byte_index,
            character,
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolScopeKind {
    Editor,
    Project,
    Document,
    Window,
    Viewport,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ToolScope {
    Editor,
    Project {
        project_session: ProjectSessionId,
    },
    Document {
        project_session: ProjectSessionId,
        document_id: DocumentId,
    },
    Window {
        window_id: UiWindowId,
    },
    Viewport {
        viewport_id: ViewInstanceId,
    },
}

impl ToolScope {
    pub const fn kind(&self) -> ToolScopeKind {
        match self {
            Self::Editor => ToolScopeKind::Editor,
            Self::Project { .. } => ToolScopeKind::Project,
            Self::Document { .. } => ToolScopeKind::Document,
            Self::Window { .. } => ToolScopeKind::Window,
            Self::Viewport { .. } => ToolScopeKind::Viewport,
        }
    }

    pub const fn project_session(&self) -> Option<ProjectSessionId> {
        match self {
            Self::Project { project_session }
            | Self::Document {
                project_session, ..
            } => Some(*project_session),
            Self::Editor | Self::Window { .. } | Self::Viewport { .. } => None,
        }
    }

    pub const fn document_id(&self) -> Option<DocumentId> {
        match self {
            Self::Document { document_id, .. } => Some(*document_id),
            Self::Editor | Self::Project { .. } | Self::Window { .. } | Self::Viewport { .. } => {
                None
            }
        }
    }

    pub fn window_id(&self) -> Option<&UiWindowId> {
        match self {
            Self::Window { window_id } => Some(window_id),
            Self::Editor | Self::Project { .. } | Self::Document { .. } | Self::Viewport { .. } => {
                None
            }
        }
    }

    pub fn viewport_id(&self) -> Option<&ViewInstanceId> {
        match self {
            Self::Viewport { viewport_id } => Some(viewport_id),
            Self::Editor | Self::Project { .. } | Self::Document { .. } | Self::Window { .. } => {
                None
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ToolResourceKey {
    kind: ToolResourceKindId,
    scope: ToolScope,
    channel: Option<ToolResourceChannelId>,
}

impl<'de> Deserialize<'de> for ToolResourceKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SerializedToolResourceKey {
            kind: ToolResourceKindId,
            scope: ToolScope,
            channel: Option<ToolResourceChannelId>,
        }

        let serialized = SerializedToolResourceKey::deserialize(deserializer)?;
        Self::new(serialized.kind, serialized.scope, serialized.channel)
            .map_err(serde::de::Error::custom)
    }
}

impl ToolResourceKey {
    pub fn new(
        kind: ToolResourceKindId,
        scope: ToolScope,
        channel: Option<ToolResourceChannelId>,
    ) -> Result<Self, ToolResourceKeyError> {
        if let Some(required) = required_builtin_scope(&kind) {
            let actual = scope.kind();
            if actual != required {
                return Err(ToolResourceKeyError::BuiltinScopeMismatch {
                    kind,
                    required,
                    actual,
                });
            }
        }
        Ok(Self {
            kind,
            scope,
            channel,
        })
    }

    pub fn viewport_input(viewport_id: ViewInstanceId) -> Self {
        Self {
            kind: ToolResourceKindId::viewport_input(),
            scope: ToolScope::Viewport { viewport_id },
            channel: None,
        }
    }

    pub fn modal_surface(window_id: UiWindowId) -> Self {
        Self {
            kind: ToolResourceKindId::modal_surface(),
            scope: ToolScope::Window { window_id },
            channel: None,
        }
    }

    pub fn scene_mode_slot(viewport_id: ViewInstanceId) -> Self {
        Self {
            kind: ToolResourceKindId::scene_mode_slot(),
            scope: ToolScope::Viewport { viewport_id },
            channel: None,
        }
    }

    pub fn kind(&self) -> &ToolResourceKindId {
        &self.kind
    }

    pub fn scope(&self) -> &ToolScope {
        &self.scope
    }

    pub fn channel(&self) -> Option<&ToolResourceChannelId> {
        self.channel.as_ref()
    }

    pub(crate) fn estimated_retained_bytes(&self) -> usize {
        let scope_bytes = match &self.scope {
            ToolScope::Window { window_id } => window_id.0.len(),
            ToolScope::Viewport { viewport_id } => viewport_id.0.len(),
            ToolScope::Editor | ToolScope::Project { .. } | ToolScope::Document { .. } => 0,
        };
        std::mem::size_of::<Self>()
            .saturating_add(self.kind.as_str().len())
            .saturating_add(
                self.channel
                    .as_ref()
                    .map(|channel| channel.as_str().len())
                    .unwrap_or_default(),
            )
            .saturating_add(scope_bytes)
    }
}

fn required_builtin_scope(kind: &ToolResourceKindId) -> Option<ToolScopeKind> {
    match kind.as_str() {
        VIEWPORT_INPUT_KIND | SCENE_MODE_SLOT_KIND => Some(ToolScopeKind::Viewport),
        MODAL_SURFACE_KIND => Some(ToolScopeKind::Window),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolResourceKeyError {
    BuiltinScopeMismatch {
        kind: ToolResourceKindId,
        required: ToolScopeKind,
        actual: ToolScopeKind,
    },
}

impl std::fmt::Display for ToolResourceKeyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BuiltinScopeMismatch {
                kind,
                required,
                actual,
            } => write!(
                formatter,
                "built-in resource kind {} requires {required:?} scope, got {actual:?}",
                kind.as_str()
            ),
        }
    }
}

impl std::error::Error for ToolResourceKeyError {}

/// Immutable, nonempty, canonically sorted resources acquired as one scheduler lease.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolResourceSet(Vec<ToolResourceKey>);

impl ToolResourceSet {
    pub fn single(resource: ToolResourceKey) -> Self {
        Self(vec![resource])
    }

    pub fn pair(first: ToolResourceKey, second: ToolResourceKey) -> Self {
        if first == second {
            return Self::single(first);
        }
        let mut resources = vec![first, second];
        resources.sort_unstable();
        Self(resources)
    }

    pub fn new<I>(resources: I) -> Result<Self, ToolResourceSetError>
    where
        I: IntoIterator<Item = ToolResourceKey>,
    {
        let mut resources = resources.into_iter().collect::<Vec<_>>();
        resources.sort_unstable();
        resources.dedup();
        if resources.is_empty() {
            return Err(ToolResourceSetError::Empty);
        }
        Ok(Self(resources))
    }

    pub fn as_slice(&self) -> &[ToolResourceKey] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Serialize for ToolResourceSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ToolResourceSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let resources = Vec::<ToolResourceKey>::deserialize(deserializer)?;
        Self::new(resources).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolResourceSetError {
    Empty,
}

impl std::fmt::Display for ToolResourceSetError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(formatter, "a tool resource set cannot be empty"),
        }
    }
}

impl std::error::Error for ToolResourceSetError {}
