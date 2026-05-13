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
