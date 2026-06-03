use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiComponentDescriptorKind {
    #[default]
    Primitive,
    Layout,
    Composite,
    EditorOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiComponentLayoutRole {
    #[default]
    Leaf,
    Overlay,
    Flex,
    Grid,
    Canvas,
    Size,
    VirtualList,
    Popup,
    EditorDock,
}

impl UiComponentLayoutRole {
    /// Stable host-facing token used by retained UI contracts and diagnostics.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Leaf => "leaf",
            Self::Overlay => "overlay",
            Self::Flex => "flex",
            Self::Grid => "grid",
            Self::Canvas => "canvas",
            Self::Size => "size",
            Self::VirtualList => "virtual-list",
            Self::Popup => "popup",
            Self::EditorDock => "editor-dock",
        }
    }
}
