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
}

impl UiDragPayload {
    pub fn new(kind: UiDragPayloadKind, reference: impl Into<String>) -> Self {
        Self {
            kind,
            reference: reference.into(),
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
