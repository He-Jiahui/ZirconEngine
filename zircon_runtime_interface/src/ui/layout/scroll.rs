use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiAxis {
    Horizontal,
    #[default]
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiScrollbarVisibility {
    Always,
    Never,
    #[default]
    Auto,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiScrollState {
    pub offset: f32,
    pub viewport_extent: f32,
    pub content_extent: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiVirtualListConfig {
    pub item_extent: f32,
    pub overscan: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiLinearBoxConfig {
    pub gap: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiScrollableBoxConfig {
    #[serde(default)]
    pub axis: UiAxis,
    pub gap: f32,
    #[serde(default)]
    pub scrollbar_visibility: UiScrollbarVisibility,
    pub virtualization: Option<UiVirtualListConfig>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum UiContainerKind {
    #[default]
    Free,
    Container,
    Overlay,
    Space,
    HorizontalBox(UiLinearBoxConfig),
    VerticalBox(UiLinearBoxConfig),
    ScrollableBox(UiScrollableBoxConfig),
}

impl UiContainerKind {
    pub const fn clips_to_bounds(self) -> bool {
        matches!(self, Self::ScrollableBox(_))
    }

    pub const fn is_scrollable(self) -> bool {
        matches!(self, Self::ScrollableBox(_))
    }
}
