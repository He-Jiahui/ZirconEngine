use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiLayoutStyle {
    pub display: UiLayoutDisplay,
    pub direction: UiFlexDirection,
    pub wrap: UiFlexWrap,
    pub justify_content: Option<UiJustify>,
    pub align_items: Option<UiAlign>,
    pub align_self: Option<UiAlign>,
    pub align_content: Option<UiAlign>,
    pub gap: UiGap,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: UiDimension,
    pub grid_template_columns: Vec<UiGridTrack>,
    pub grid_template_rows: Vec<UiGridTrack>,
    pub grid_row: Option<UiGridPlacement>,
    pub grid_column: Option<UiGridPlacement>,
    pub size: UiLayoutSize,
    pub min_size: UiLayoutSize,
    pub max_size: UiLayoutSize,
    pub aspect_ratio: Option<f32>,
    pub margin: UiEdges,
    pub padding: UiEdges,
    pub position: UiPositionMode,
    pub inset: UiEdges,
    pub overflow: UiOverflowPair,
}

impl Default for UiLayoutStyle {
    fn default() -> Self {
        Self {
            display: UiLayoutDisplay::Flex,
            direction: UiFlexDirection::Row,
            wrap: UiFlexWrap::NoWrap,
            justify_content: None,
            align_items: None,
            align_self: None,
            align_content: None,
            gap: UiGap::default(),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: UiDimension::Auto,
            grid_template_columns: Vec::new(),
            grid_template_rows: Vec::new(),
            grid_row: None,
            grid_column: None,
            size: UiLayoutSize::auto(),
            min_size: UiLayoutSize::auto(),
            max_size: UiLayoutSize::auto(),
            aspect_ratio: None,
            margin: UiEdges::zero(),
            padding: UiEdges::zero(),
            position: UiPositionMode::Relative,
            inset: UiEdges::zero(),
            overflow: UiOverflowPair::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLayoutDisplay {
    #[default]
    Flex,
    Grid,
    Block,
    Overlay,
    Canvas,
    Scroll,
    Virtual,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiJustify {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAlign {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiGap {
    #[serde(default = "zero_dimension")]
    pub row: UiDimension,
    #[serde(default = "zero_dimension")]
    pub column: UiDimension,
}

impl Default for UiGap {
    fn default() -> Self {
        Self {
            row: UiDimension::Px(0.0),
            column: UiDimension::Px(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDimension {
    #[default]
    Auto,
    Px(f32),
    /// Normalized percentage in the 0.0..=1.0 range.
    Percent(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiLayoutSize {
    #[serde(default)]
    pub width: UiDimension,
    #[serde(default)]
    pub height: UiDimension,
}

impl UiLayoutSize {
    pub const fn auto() -> Self {
        Self {
            width: UiDimension::Auto,
            height: UiDimension::Auto,
        }
    }
}

impl Default for UiLayoutSize {
    fn default() -> Self {
        Self::auto()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiGridTrack {
    Px(f32),
    Percent(f32),
    Fr(f32),
    Auto,
    MinMax {
        min: UiGridTrackBreadth,
        max: UiGridTrackBreadth,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiGridTrackBreadth {
    Px(f32),
    Percent(f32),
    Fr(f32),
    Auto,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiGridPlacement {
    pub start: UiGridLine,
    pub end: UiGridLine,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiGridLine {
    #[default]
    Auto,
    Line(i16),
    Span(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiEdges {
    #[serde(default = "zero_dimension")]
    pub left: UiDimension,
    #[serde(default = "zero_dimension")]
    pub right: UiDimension,
    #[serde(default = "zero_dimension")]
    pub top: UiDimension,
    #[serde(default = "zero_dimension")]
    pub bottom: UiDimension,
}

impl UiEdges {
    pub const fn zero() -> Self {
        Self {
            left: UiDimension::Px(0.0),
            right: UiDimension::Px(0.0),
            top: UiDimension::Px(0.0),
            bottom: UiDimension::Px(0.0),
        }
    }
}

impl Default for UiEdges {
    fn default() -> Self {
        Self::zero()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPositionMode {
    #[default]
    Relative,
    Absolute,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiOverflowPair {
    #[serde(default)]
    pub x: UiOverflow,
    #[serde(default)]
    pub y: UiOverflow,
}

fn zero_dimension() -> UiDimension {
    UiDimension::Px(0.0)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiLayoutStyleSourceRef {
    pub source: UiLayoutStyleSourceKind,
    pub id: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLayoutStyleSourceKind {
    Asset,
    Class,
    #[default]
    Inline,
    RuntimeState,
}
