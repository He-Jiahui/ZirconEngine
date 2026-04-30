use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiDragPayloadKind {
    Asset,
    SceneInstance,
    Object,
}

impl UiDragPayloadKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::SceneInstance => "scene-instance",
            Self::Object => "object",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDragPayload {
    pub kind: UiDragPayloadKind,
    pub reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<UiDragSourceMetadata>,
}

impl UiDragPayload {
    pub fn new(kind: UiDragPayloadKind, reference: impl Into<String>) -> Self {
        Self {
            kind,
            reference: reference.into(),
            source: None,
        }
    }

    pub fn with_source(mut self, source: UiDragSourceMetadata) -> Self {
        self.source = Some(source);
        self
    }

    pub fn source_summary(&self) -> Option<String> {
        self.source.as_ref().and_then(UiDragSourceMetadata::summary)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDragSourceMetadata {
    pub source_surface: String,
    pub source_control_id: String,
    pub asset_uuid: Option<String>,
    pub locator: Option<String>,
    pub display_name: Option<String>,
    pub asset_kind: Option<String>,
    pub extension: Option<String>,
}

impl UiDragSourceMetadata {
    pub fn asset(
        source_surface: impl Into<String>,
        source_control_id: impl Into<String>,
        asset_uuid: impl Into<String>,
        locator: impl Into<String>,
        display_name: impl Into<String>,
        asset_kind: impl Into<String>,
        extension: impl Into<String>,
    ) -> Self {
        Self {
            source_surface: source_surface.into(),
            source_control_id: source_control_id.into(),
            asset_uuid: Some(asset_uuid.into()),
            locator: Some(locator.into()),
            display_name: Some(display_name.into()),
            asset_kind: Some(asset_kind.into()),
            extension: Some(extension.into()),
        }
    }

    pub fn summary(&self) -> Option<String> {
        match (&self.asset_kind, &self.display_name) {
            (Some(kind), Some(name)) if !kind.is_empty() && !name.is_empty() => {
                Some(format!("{kind}: {name}"))
            }
            (_, Some(name)) if !name.is_empty() => Some(name.clone()),
            (_, _) => self.locator.clone().filter(|locator| !locator.is_empty()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDropPolicy {
    pub accepts: Vec<UiDragPayloadKind>,
}

impl UiDropPolicy {
    pub fn new(accepts: impl IntoIterator<Item = UiDragPayloadKind>) -> Self {
        Self {
            accepts: accepts.into_iter().collect(),
        }
    }

    pub fn accepts(&self, kind: UiDragPayloadKind) -> bool {
        self.accepts.iter().any(|accepted| *accepted == kind)
    }
}
