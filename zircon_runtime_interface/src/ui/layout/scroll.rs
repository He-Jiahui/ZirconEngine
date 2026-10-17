use serde::{Deserialize, Serialize};

use super::UiLayoutEngineFamily;

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

/// Runtime 布局更新使用的滚动几何值；更新计划会把偏移限制在内容可滚范围内，此类型本身只保存数据。
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiScrollState {
    /// 沿配置滚动轴的当前内容起点；更新时按内容与视口范围钳制。
    pub offset: f32,
    /// 视口沿滚动轴可显示的长度。
    pub viewport_extent: f32,
    /// 滚动内容沿滚动轴的总长度。
    pub content_extent: f32,
}

/// 固定尺寸虚拟列表配置；Runtime 将 item_extent 与容器 gap 合成步长，并按 overscan 扩展逻辑窗口。
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiVirtualListConfig {
    /// 每个逻辑项沿滚动轴的固定长度。
    pub item_extent: f32,
    /// 可视窗口前后额外保留的逻辑项数量。
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
pub struct UiWrapBoxConfig {
    pub horizontal_gap: f32,
    pub vertical_gap: f32,
    pub item_min_width: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiGridBoxConfig {
    pub columns: usize,
    pub rows: usize,
    pub column_gap: f32,
    pub row_gap: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiMasonryBoxConfig {
    pub columns: usize,
    pub gap: f32,
    pub sequential: bool,
}

impl Default for UiMasonryBoxConfig {
    fn default() -> Self {
        Self {
            columns: 4,
            gap: 0.0,
            sequential: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSizeBoxConfig {
    pub aspect_ratio: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum UiContainerKind {
    #[default]
    Free,
    Canvas,
    Container,
    BlockBox,
    Overlay,
    Space,
    SizeBox(UiSizeBoxConfig),
    HorizontalBox(UiLinearBoxConfig),
    VerticalBox(UiLinearBoxConfig),
    ScrollableBox(UiScrollableBoxConfig),
    WrapBox(UiWrapBoxConfig),
    GridBox(UiGridBoxConfig),
    MasonryBox(UiMasonryBoxConfig),
}

impl UiContainerKind {
    pub const fn child_slot_kind(self) -> Option<super::UiSlotKind> {
        match self {
            Self::Free => Some(super::UiSlotKind::Free),
            Self::Canvas => Some(super::UiSlotKind::Canvas),
            Self::Container | Self::BlockBox | Self::SizeBox(_) => {
                Some(super::UiSlotKind::Container)
            }
            Self::Overlay => Some(super::UiSlotKind::Overlay),
            Self::Space => None,
            Self::HorizontalBox(_) | Self::VerticalBox(_) => Some(super::UiSlotKind::Linear),
            Self::ScrollableBox(_) => Some(super::UiSlotKind::Scrollable),
            Self::WrapBox(_) | Self::MasonryBox(_) => Some(super::UiSlotKind::Flow),
            Self::GridBox(_) => Some(super::UiSlotKind::Grid),
        }
    }

    /// Selects the owning layout algorithm while preserving the portable child-slot
    /// contract. Block, virtual list, and Masonry behavior require parent context
    /// that a generic child placement kind intentionally does not encode.
    pub const fn layout_engine_family(self) -> UiLayoutEngineFamily {
        match self {
            Self::BlockBox => UiLayoutEngineFamily::Block,
            Self::ScrollableBox(UiScrollableBoxConfig {
                virtualization: Some(_),
                ..
            }) => UiLayoutEngineFamily::VirtualizedList,
            Self::MasonryBox(_) => UiLayoutEngineFamily::Masonry,
            _ => match self.child_slot_kind() {
                Some(slot_kind) => slot_kind.layout_engine_family(),
                None => UiLayoutEngineFamily::Container,
            },
        }
    }

    pub const fn clips_to_bounds(self) -> bool {
        matches!(self, Self::ScrollableBox(_))
    }

    pub const fn is_scrollable(self) -> bool {
        matches!(self, Self::ScrollableBox(_))
    }

    pub const fn is_auto_layout_container(self) -> bool {
        matches!(
            self,
            Self::HorizontalBox(_)
                | Self::VerticalBox(_)
                | Self::BlockBox
                | Self::SizeBox(_)
                | Self::ScrollableBox(_)
                | Self::WrapBox(_)
                | Self::GridBox(_)
                | Self::MasonryBox(_)
        )
    }
}
