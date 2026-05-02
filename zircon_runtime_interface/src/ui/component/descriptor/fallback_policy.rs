use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWidgetEditorFallback {
    Hidden,
    Placeholder,
    DisableInteractions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWidgetRuntimeFallback {
    RejectNode,
    PlaceholderNode,
    OmitNode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiWidgetFallbackPolicy {
    pub editor: UiWidgetEditorFallback,
    pub runtime: UiWidgetRuntimeFallback,
}

impl UiWidgetFallbackPolicy {
    pub const fn new(editor: UiWidgetEditorFallback, runtime: UiWidgetRuntimeFallback) -> Self {
        Self { editor, runtime }
    }
}

impl Default for UiWidgetFallbackPolicy {
    fn default() -> Self {
        Self::new(
            UiWidgetEditorFallback::Placeholder,
            UiWidgetRuntimeFallback::RejectNode,
        )
    }
}
