use std::str::FromStr;

use thiserror::Error;
use zircon_runtime_interface::ui::{
    design_tokens::EditorDesignTokens,
    layout::{
        UiAlign, UiDimension, UiEdges, UiFlexDirection, UiFlexWrap, UiGap, UiGridPlacement,
        UiGridTrack, UiGridTrackBreadth, UiJustify, UiLayoutDisplay, UiLayoutEngineFamily,
        UiLayoutSize, UiLayoutStyle, UiOverflow, UiOverflowPair, UiPositionMode, UiSlotKind,
    },
};

use super::WorkbenchConstraintTokenName;

/// Author-side layout vocabulary that normalizes into the runtime-neutral DTO.
///
/// This type deliberately owns only parsing and validation. Taffy translation remains
/// in the runtime layout bridge so editor templates cannot acquire a second solver path.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLikeConstraint {
    pub display: UiLayoutDisplay,
    pub direction: UiFlexDirection,
    pub wrap: UiFlexWrap,
    pub justify_content: Option<UiJustify>,
    pub align_items: Option<UiAlign>,
    pub align_self: Option<UiAlign>,
    pub align_content: Option<UiAlign>,
    pub gap: Option<CssLikeGap>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: CssLikeDimension,
    pub grid_template_columns: Vec<CssLikeGridTrack>,
    pub grid_template_rows: Vec<CssLikeGridTrack>,
    pub grid_row: Option<UiGridPlacement>,
    pub grid_column: Option<UiGridPlacement>,
    pub size: CssLikeSize,
    pub min_size: CssLikeSize,
    pub max_size: CssLikeSize,
    pub aspect_ratio: Option<f32>,
    pub margin: CssLikeEdges,
    pub padding: Option<CssLikeEdges>,
    pub position: UiPositionMode,
    pub inset: CssLikeEdges,
    pub overflow: CssLikeOverflowPair,
}

impl Default for CssLikeConstraint {
    fn default() -> Self {
        Self {
            display: UiLayoutDisplay::Flex,
            direction: UiFlexDirection::Row,
            wrap: UiFlexWrap::NoWrap,
            justify_content: None,
            align_items: None,
            align_self: None,
            align_content: None,
            gap: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: CssLikeDimension::Auto,
            grid_template_columns: Vec::new(),
            grid_template_rows: Vec::new(),
            grid_row: None,
            grid_column: None,
            size: CssLikeSize::auto(),
            min_size: CssLikeSize::auto(),
            max_size: CssLikeSize::auto(),
            aspect_ratio: None,
            margin: CssLikeEdges::zero(),
            padding: None,
            position: UiPositionMode::Relative,
            inset: CssLikeEdges::zero(),
            overflow: CssLikeOverflowPair::default(),
        }
    }
}

impl CssLikeConstraint {
    pub fn family(&self) -> UiLayoutEngineFamily {
        match self.display {
            UiLayoutDisplay::Flex if self.wrap != UiFlexWrap::NoWrap => {
                family_for_slot_kind(UiSlotKind::Flow)
            }
            UiLayoutDisplay::Flex => family_for_slot_kind(UiSlotKind::Linear),
            UiLayoutDisplay::Grid => family_for_slot_kind(UiSlotKind::Grid),
            UiLayoutDisplay::Block => UiLayoutEngineFamily::Block,
            UiLayoutDisplay::Overlay => family_for_slot_kind(UiSlotKind::Overlay),
            UiLayoutDisplay::Canvas => family_for_slot_kind(UiSlotKind::Canvas),
            UiLayoutDisplay::Scroll => family_for_slot_kind(UiSlotKind::Scrollable),
            UiLayoutDisplay::Virtual => UiLayoutEngineFamily::VirtualizedList,
            UiLayoutDisplay::None => UiLayoutEngineFamily::Free,
        }
    }

    pub fn into_layout_style(
        &self,
        tokens: &EditorDesignTokens,
    ) -> Result<UiLayoutStyle, CssLikeConstraintError> {
        if self.align_content == Some(UiAlign::Baseline) {
            return Err(CssLikeConstraintError::UnsupportedAlignment {
                property: "align-content",
                value: "baseline",
            });
        }

        let gap = match &self.gap {
            Some(gap) => gap.resolve(tokens)?,
            None => UiGap::default(),
        };
        let padding = match &self.padding {
            Some(padding) => padding.resolve(tokens, "padding", false)?,
            None => UiEdges::zero(),
        };

        Ok(UiLayoutStyle {
            display: self.display,
            direction: self.direction,
            wrap: self.wrap,
            justify_content: self.justify_content,
            align_items: self.align_items,
            align_self: self.align_self,
            align_content: self.align_content,
            gap,
            flex_grow: finite_non_negative(self.flex_grow, "flex-grow")?,
            flex_shrink: finite_non_negative(self.flex_shrink, "flex-shrink")?,
            flex_basis: self.flex_basis.resolve(tokens, "flex-basis", true)?,
            grid_template_columns: self
                .grid_template_columns
                .iter()
                .map(|track| track.resolve(tokens))
                .collect::<Result<Vec<_>, _>>()?,
            grid_template_rows: self
                .grid_template_rows
                .iter()
                .map(|track| track.resolve(tokens))
                .collect::<Result<Vec<_>, _>>()?,
            grid_row: self.grid_row,
            grid_column: self.grid_column,
            size: self.size.resolve(tokens, "size")?,
            min_size: self.min_size.resolve(tokens, "min-size")?,
            max_size: self.max_size.resolve(tokens, "max-size")?,
            aspect_ratio: self
                .aspect_ratio
                .map(|value| finite_non_negative(value, "aspect-ratio"))
                .transpose()?,
            margin: self.margin.resolve(tokens, "margin", true)?,
            padding,
            position: self.position,
            inset: self.inset.resolve(tokens, "inset", true)?,
            overflow: self.overflow.resolve(),
        })
    }
}

/// Maps a slot declaration to the layout executor family selected by its parent.
pub const fn family_for_slot_kind(kind: UiSlotKind) -> UiLayoutEngineFamily {
    kind.layout_engine_family()
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssLikeGap {
    pub row: CssLikeDimension,
    pub column: CssLikeDimension,
}

impl CssLikeGap {
    pub fn uniform(value: CssLikeDimension) -> Self {
        Self {
            row: value.clone(),
            column: value,
        }
    }

    fn resolve(&self, tokens: &EditorDesignTokens) -> Result<UiGap, CssLikeConstraintError> {
        Ok(UiGap {
            row: self.row.resolve(tokens, "gap", false)?,
            column: self.column.resolve(tokens, "gap", false)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssLikeSize {
    pub width: CssLikeDimension,
    pub height: CssLikeDimension,
}

impl CssLikeSize {
    pub const fn auto() -> Self {
        Self {
            width: CssLikeDimension::Auto,
            height: CssLikeDimension::Auto,
        }
    }

    fn resolve(
        &self,
        tokens: &EditorDesignTokens,
        property: &'static str,
    ) -> Result<UiLayoutSize, CssLikeConstraintError> {
        Ok(UiLayoutSize {
            width: self.width.resolve(tokens, property, true)?,
            height: self.height.resolve(tokens, property, true)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssLikeEdges {
    pub left: CssLikeDimension,
    pub right: CssLikeDimension,
    pub top: CssLikeDimension,
    pub bottom: CssLikeDimension,
}

impl CssLikeEdges {
    pub const fn zero() -> Self {
        Self {
            left: CssLikeDimension::Px(0.0),
            right: CssLikeDimension::Px(0.0),
            top: CssLikeDimension::Px(0.0),
            bottom: CssLikeDimension::Px(0.0),
        }
    }

    pub fn all(value: CssLikeDimension) -> Self {
        Self {
            left: value.clone(),
            right: value.clone(),
            top: value.clone(),
            bottom: value,
        }
    }

    fn resolve(
        &self,
        tokens: &EditorDesignTokens,
        property: &'static str,
        allows_auto: bool,
    ) -> Result<UiEdges, CssLikeConstraintError> {
        Ok(UiEdges {
            left: self.left.resolve(tokens, property, allows_auto)?,
            right: self.right.resolve(tokens, property, allows_auto)?,
            top: self.top.resolve(tokens, property, allows_auto)?,
            bottom: self.bottom.resolve(tokens, property, allows_auto)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CssLikeDimension {
    Auto,
    Px(f32),
    Percent(f32),
    Token(WorkbenchConstraintTokenName),
}

impl CssLikeDimension {
    fn resolve(
        &self,
        tokens: &EditorDesignTokens,
        property: &'static str,
        allows_auto: bool,
    ) -> Result<UiDimension, CssLikeConstraintError> {
        match self {
            Self::Auto if allows_auto => Ok(UiDimension::Auto),
            Self::Auto => Err(CssLikeConstraintError::AutoNotAllowed { property }),
            Self::Px(value) => Ok(UiDimension::Px(finite_non_negative(*value, property)?)),
            Self::Percent(value) => Ok(UiDimension::Percent(normalized_percent(*value, property)?)),
            Self::Token(token) => {
                let token_name = canonical_token_name(token.as_str());
                let value = tokens
                    .density_value_for_token_name(token_name)
                    .ok_or_else(|| CssLikeConstraintError::UnknownToken {
                        token: token.as_str().to_string(),
                    })?;
                Ok(UiDimension::Px(finite_non_negative(value, property)?))
            }
        }
    }
}

impl FromStr for CssLikeDimension {
    type Err = CssLikeConstraintError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let value = source.trim();
        if value.eq_ignore_ascii_case("auto") {
            return Ok(Self::Auto);
        }
        if let Some(token) = value.strip_prefix('$') {
            if token.is_empty() {
                return Err(CssLikeConstraintError::InvalidDimension {
                    value: value.to_string(),
                });
            }
            return Ok(Self::Token(WorkbenchConstraintTokenName::new(token)));
        }
        if let Some(percent) = value.strip_suffix('%') {
            let percent = parse_author_number(percent, value)?;
            return Ok(Self::Percent(normalized_percent(
                percent / 100.0,
                "percent",
            )?));
        }
        if let Some(unit) = unsupported_viewport_unit(value) {
            return Err(CssLikeConstraintError::KnownUnsupportedUnit { unit });
        }
        if let Some(px) = value.strip_suffix("px") {
            return Ok(Self::Px(finite_non_negative(
                parse_author_number(px, value)?,
                "px",
            )?));
        }
        Err(CssLikeConstraintError::InvalidDimension {
            value: value.to_string(),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CssLikeOverflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
}

impl CssLikeOverflow {
    pub const fn resolve(self) -> UiOverflow {
        match self {
            Self::Visible => UiOverflow::Visible,
            Self::Hidden => UiOverflow::Hidden,
            Self::Scroll => UiOverflow::Scroll,
        }
    }
}

impl FromStr for CssLikeOverflow {
    type Err = CssLikeConstraintError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match source.trim() {
            "visible" => Ok(Self::Visible),
            "hidden" => Ok(Self::Hidden),
            "scroll" => Ok(Self::Scroll),
            "clip" => Err(CssLikeConstraintError::KnownUnsupportedSyntax {
                property: "overflow",
                value: "clip",
            }),
            value => Err(CssLikeConstraintError::UnknownProperty {
                property: value.to_string(),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CssLikeOverflowPair {
    pub x: CssLikeOverflow,
    pub y: CssLikeOverflow,
}

impl CssLikeOverflowPair {
    pub const fn resolve(self) -> UiOverflowPair {
        UiOverflowPair {
            x: self.x.resolve(),
            y: self.y.resolve(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CssLikeGridTrack {
    Dimension(CssLikeDimension),
    Fr(f32),
    MinMax {
        min: CssLikeGridTrackBreadth,
        max: CssLikeGridTrackBreadth,
    },
}

impl CssLikeGridTrack {
    pub fn from_author_value(source: &str) -> Result<Self, CssLikeConstraintError> {
        let value = source.trim();
        if let Some(arguments) = function_arguments(value, "minmax") {
            let Some((min, max)) = arguments.split_once(',') else {
                return Err(CssLikeConstraintError::InvalidDimension {
                    value: value.to_string(),
                });
            };
            let min = CssLikeGridTrackBreadth::from_author_value(min)?;
            if matches!(min, CssLikeGridTrackBreadth::Fr(_)) {
                return Err(CssLikeConstraintError::KnownUnsupportedSyntax {
                    property: "grid-template",
                    value: "minmax-fr-minimum",
                });
            }
            return Ok(Self::MinMax {
                min,
                max: CssLikeGridTrackBreadth::from_author_value(max)?,
            });
        }
        if function_arguments(value, "repeat").is_some() {
            return Err(CssLikeConstraintError::KnownUnsupportedSyntax {
                property: "grid-template",
                value: "repeat",
            });
        }
        if function_arguments(value, "fit-content").is_some() {
            return Err(CssLikeConstraintError::KnownUnsupportedSyntax {
                property: "grid-template",
                value: "fit-content",
            });
        }
        if let Some(fr) = value.strip_suffix("fr") {
            return Ok(Self::Fr(finite_non_negative(
                parse_author_number(fr, value)?,
                "grid-fr",
            )?));
        }
        Ok(Self::Dimension(CssLikeDimension::from_str(value)?))
    }

    fn resolve(&self, tokens: &EditorDesignTokens) -> Result<UiGridTrack, CssLikeConstraintError> {
        match self {
            Self::Dimension(dimension) => match dimension.resolve(tokens, "grid-track", true)? {
                UiDimension::Auto => Ok(UiGridTrack::Auto),
                UiDimension::Px(value) => Ok(UiGridTrack::Px(value)),
                UiDimension::Percent(value) => Ok(UiGridTrack::Percent(value)),
            },
            Self::Fr(value) => Ok(UiGridTrack::Fr(finite_non_negative(*value, "grid-fr")?)),
            Self::MinMax { min, max } => Ok(UiGridTrack::MinMax {
                min: min.resolve(tokens)?,
                max: max.resolve(tokens)?,
            }),
        }
    }
}

impl FromStr for CssLikeGridTrack {
    type Err = CssLikeConstraintError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::from_author_value(source)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CssLikeGridTrackBreadth {
    Dimension(CssLikeDimension),
    Fr(f32),
}

impl CssLikeGridTrackBreadth {
    pub fn from_author_value(source: &str) -> Result<Self, CssLikeConstraintError> {
        let value = source.trim();
        if function_arguments(value, "fit-content").is_some() {
            return Err(CssLikeConstraintError::KnownUnsupportedSyntax {
                property: "grid-template",
                value: "fit-content",
            });
        }
        if let Some(fr) = value.strip_suffix("fr") {
            return Ok(Self::Fr(finite_non_negative(
                parse_author_number(fr, value)?,
                "grid-fr",
            )?));
        }
        Ok(Self::Dimension(CssLikeDimension::from_str(value)?))
    }

    fn resolve(
        &self,
        tokens: &EditorDesignTokens,
    ) -> Result<UiGridTrackBreadth, CssLikeConstraintError> {
        match self {
            Self::Dimension(dimension) => match dimension.resolve(tokens, "grid-track", true)? {
                UiDimension::Auto => Ok(UiGridTrackBreadth::Auto),
                UiDimension::Px(value) => Ok(UiGridTrackBreadth::Px(value)),
                UiDimension::Percent(value) => Ok(UiGridTrackBreadth::Percent(value)),
            },
            Self::Fr(value) => Ok(UiGridTrackBreadth::Fr(finite_non_negative(
                *value, "grid-fr",
            )?)),
        }
    }
}

impl FromStr for CssLikeGridTrackBreadth {
    type Err = CssLikeConstraintError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::from_author_value(source)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CssLikeConstraintProperty {
    Display,
    FlexDirection,
    FlexWrap,
    JustifyContent,
    AlignItems,
    AlignSelf,
    AlignContent,
    Gap,
    RowGap,
    ColumnGap,
    FlexGrow,
    FlexShrink,
    FlexBasis,
    GridTemplateColumns,
    GridTemplateRows,
    GridRow,
    GridColumn,
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    AspectRatio,
    Margin,
    Padding,
    Position,
    Inset,
    Top,
    Right,
    Bottom,
    Left,
    Overflow,
    OverflowX,
    OverflowY,
}

impl FromStr for CssLikeConstraintProperty {
    type Err = CssLikeConstraintError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let property = source.trim();
        if let Some(property) = known_unsupported_property(property) {
            return Err(CssLikeConstraintError::KnownUnsupportedProperty { property });
        }
        let parsed = match property {
            "display" => Self::Display,
            "flex-direction" => Self::FlexDirection,
            "flex-wrap" => Self::FlexWrap,
            "justify-content" => Self::JustifyContent,
            "align-items" => Self::AlignItems,
            "align-self" => Self::AlignSelf,
            "align-content" => Self::AlignContent,
            "gap" => Self::Gap,
            "row-gap" => Self::RowGap,
            "column-gap" => Self::ColumnGap,
            "flex-grow" => Self::FlexGrow,
            "flex-shrink" => Self::FlexShrink,
            "flex-basis" => Self::FlexBasis,
            "grid-template-columns" => Self::GridTemplateColumns,
            "grid-template-rows" => Self::GridTemplateRows,
            "grid-row" => Self::GridRow,
            "grid-column" => Self::GridColumn,
            "width" => Self::Width,
            "height" => Self::Height,
            "min-width" => Self::MinWidth,
            "min-height" => Self::MinHeight,
            "max-width" => Self::MaxWidth,
            "max-height" => Self::MaxHeight,
            "aspect-ratio" => Self::AspectRatio,
            "margin" => Self::Margin,
            "padding" => Self::Padding,
            "position" => Self::Position,
            "inset" => Self::Inset,
            "top" => Self::Top,
            "right" => Self::Right,
            "bottom" => Self::Bottom,
            "left" => Self::Left,
            "overflow" => Self::Overflow,
            "overflow-x" => Self::OverflowX,
            "overflow-y" => Self::OverflowY,
            _ => {
                return Err(CssLikeConstraintError::UnknownProperty {
                    property: property.to_string(),
                });
            }
        };
        Ok(parsed)
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum CssLikeConstraintError {
    #[error("invalid CSS-like layout dimension `{value}`")]
    InvalidDimension { value: String },
    #[error("invalid CSS-like layout value `{value}` for `{property}")]
    InvalidValue {
        property: &'static str,
        value: String,
    },
    #[error("layout property `{property}` requires a finite non-negative value, got {value}")]
    InvalidNumericValue { property: &'static str, value: f32 },
    #[error("layout property `{property}` does not allow auto")]
    AutoNotAllowed { property: &'static str },
    #[error("layout property `{property}` does not support `{value}`")]
    UnsupportedAlignment {
        property: &'static str,
        value: &'static str,
    },
    #[error("unknown layout token `{token}`")]
    UnknownToken { token: String },
    #[error(
        "CSS-like layout unit `{unit}` is a registered extension candidate and is not supported"
    )]
    KnownUnsupportedUnit { unit: &'static str },
    #[error(
        "CSS-like layout property `{property}` is a registered extension candidate and is not supported"
    )]
    KnownUnsupportedProperty { property: &'static str },
    #[error(
        "CSS-like layout value `{value}` for `{property}` is a registered extension candidate and is not supported"
    )]
    KnownUnsupportedSyntax {
        property: &'static str,
        value: &'static str,
    },
    #[error("unknown CSS-like layout property `{property}`")]
    UnknownProperty { property: String },
}

fn finite_non_negative(value: f32, property: &'static str) -> Result<f32, CssLikeConstraintError> {
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(CssLikeConstraintError::InvalidNumericValue { property, value })
    }
}

fn normalized_percent(value: f32, property: &'static str) -> Result<f32, CssLikeConstraintError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        Err(CssLikeConstraintError::InvalidNumericValue { property, value })
    }
}

fn parse_author_number(value: &str, source: &str) -> Result<f32, CssLikeConstraintError> {
    value
        .trim()
        .parse::<f32>()
        .map_err(|_| CssLikeConstraintError::InvalidDimension {
            value: source.to_string(),
        })
}

fn function_arguments<'a>(value: &'a str, function: &str) -> Option<&'a str> {
    value
        .strip_prefix(function)?
        .strip_prefix('(')?
        .strip_suffix(')')
}

fn unsupported_viewport_unit(value: &str) -> Option<&'static str> {
    let bytes = value.as_bytes();
    match bytes.last().copied()? {
        b'w' if bytes.ends_with(b"vw") => Some("vw"),
        b'h' if bytes.ends_with(b"vh") => Some("vh"),
        b'n' if bytes.ends_with(b"vmin") => Some("vmin"),
        b'x' if bytes.ends_with(b"vmax") => Some("vmax"),
        _ => None,
    }
}

fn canonical_token_name(token_name: &str) -> &str {
    match token_name {
        "gap.xs" => "editor.density.gap.xsmall",
        "gap.s" => "editor.density.gap.small",
        "gap.m" => "editor.density.gap.medium",
        "gap.l" => "editor.density.gap.large",
        "pad.s" => "editor.density.drawer_padding",
        "pad.m" => "editor.density.panel_padding",
        _ => token_name,
    }
}

fn known_unsupported_property(property: &str) -> Option<&'static str> {
    match property {
        "vw" => Some("vw"),
        "vh" => Some("vh"),
        "vmin" => Some("vmin"),
        "vmax" => Some("vmax"),
        "justify-items" => Some("justify-items"),
        "justify-self" => Some("justify-self"),
        "grid-auto-flow" => Some("grid-auto-flow"),
        "grid-auto-rows" => Some("grid-auto-rows"),
        "grid-auto-columns" => Some("grid-auto-columns"),
        "fit-content" => Some("fit-content"),
        "repeat" => Some("repeat"),
        "overflow-clip-margin" => Some("overflow-clip-margin"),
        "box-sizing" => Some("box-sizing"),
        "direction" => Some("direction"),
        "object-fit" => Some("object-fit"),
        "z-index" => Some("z-index"),
        _ => None,
    }
}

mod declaration_parser;

#[cfg(test)]
mod tests;
