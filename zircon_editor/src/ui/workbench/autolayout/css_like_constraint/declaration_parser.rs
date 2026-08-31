use std::str::FromStr;

use zircon_runtime_interface::ui::layout::{
    UiAlign, UiFlexDirection, UiFlexWrap, UiGridLine, UiGridPlacement, UiJustify, UiLayoutDisplay,
    UiPositionMode,
};

use super::{
    finite_non_negative, parse_author_number, CssLikeConstraint, CssLikeConstraintError,
    CssLikeConstraintProperty, CssLikeDimension, CssLikeEdges, CssLikeGap, CssLikeGridTrack,
    CssLikeOverflow,
};

impl CssLikeConstraint {
    /// Builds a constraint from a CSS-like author declaration list.
    ///
    /// Declarations are applied in source order, so a later declaration for the same
    /// property replaces the earlier value. This preserves the authoring model without
    /// allowing runtime stylesheet evaluation into the retained UI path.
    pub fn from_declarations<'a>(
        declarations: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self, CssLikeConstraintError> {
        let mut constraint = Self::default();
        for (property, value) in declarations {
            constraint.apply_declaration(property, value)?;
        }
        Ok(constraint)
    }

    /// Applies one supported CSS-like declaration to this constraint.
    pub fn apply_declaration(
        &mut self,
        property: &str,
        value: &str,
    ) -> Result<(), CssLikeConstraintError> {
        let property = CssLikeConstraintProperty::from_str(property)?;
        let value = value.trim();

        match property {
            CssLikeConstraintProperty::Display => self.display = parse_display(value)?,
            CssLikeConstraintProperty::FlexDirection => {
                self.direction = parse_flex_direction(value)?
            }
            CssLikeConstraintProperty::FlexWrap => self.wrap = parse_flex_wrap(value)?,
            CssLikeConstraintProperty::JustifyContent => {
                self.justify_content = Some(parse_justify(value)?)
            }
            CssLikeConstraintProperty::AlignItems => {
                self.align_items = Some(parse_align(value, "align-items")?)
            }
            CssLikeConstraintProperty::AlignSelf => {
                self.align_self = Some(parse_align(value, "align-self")?)
            }
            CssLikeConstraintProperty::AlignContent => {
                self.align_content = Some(parse_align(value, "align-content")?)
            }
            CssLikeConstraintProperty::Gap => {
                self.gap = Some(CssLikeGap::uniform(parse_dimension(value, "gap", false)?));
            }
            CssLikeConstraintProperty::RowGap => {
                let row = parse_dimension(value, "row-gap", false)?;
                let column = self
                    .gap
                    .as_ref()
                    .map(|gap| gap.column.clone())
                    .unwrap_or(CssLikeDimension::Px(0.0));
                self.gap = Some(CssLikeGap { row, column });
            }
            CssLikeConstraintProperty::ColumnGap => {
                let column = parse_dimension(value, "column-gap", false)?;
                let row = self
                    .gap
                    .as_ref()
                    .map(|gap| gap.row.clone())
                    .unwrap_or(CssLikeDimension::Px(0.0));
                self.gap = Some(CssLikeGap { row, column });
            }
            CssLikeConstraintProperty::FlexGrow => {
                self.flex_grow = parse_non_negative_number(value, "flex-grow")?
            }
            CssLikeConstraintProperty::FlexShrink => {
                self.flex_shrink = parse_non_negative_number(value, "flex-shrink")?
            }
            CssLikeConstraintProperty::FlexBasis => {
                self.flex_basis = parse_dimension(value, "flex-basis", true)?
            }
            CssLikeConstraintProperty::GridTemplateColumns => {
                self.grid_template_columns = parse_grid_tracks(value, "grid-template-columns")?
            }
            CssLikeConstraintProperty::GridTemplateRows => {
                self.grid_template_rows = parse_grid_tracks(value, "grid-template-rows")?
            }
            CssLikeConstraintProperty::GridColumn => {
                self.grid_column = Some(parse_grid_placement(value, "grid-column")?)
            }
            CssLikeConstraintProperty::GridRow => {
                self.grid_row = Some(parse_grid_placement(value, "grid-row")?)
            }
            CssLikeConstraintProperty::Width => {
                self.size.width = parse_dimension(value, "width", true)?
            }
            CssLikeConstraintProperty::Height => {
                self.size.height = parse_dimension(value, "height", true)?
            }
            CssLikeConstraintProperty::MinWidth => {
                self.min_size.width = parse_dimension(value, "min-width", false)?
            }
            CssLikeConstraintProperty::MinHeight => {
                self.min_size.height = parse_dimension(value, "min-height", false)?
            }
            CssLikeConstraintProperty::MaxWidth => {
                self.max_size.width = parse_dimension(value, "max-width", false)?
            }
            CssLikeConstraintProperty::MaxHeight => {
                self.max_size.height = parse_dimension(value, "max-height", false)?
            }
            CssLikeConstraintProperty::AspectRatio => {
                self.aspect_ratio = Some(parse_aspect_ratio(value)?)
            }
            CssLikeConstraintProperty::Margin => {
                self.margin = parse_edge_values(value, "margin", true)?
            }
            CssLikeConstraintProperty::Padding => {
                self.padding = Some(parse_edge_values(value, "padding", false)?)
            }
            CssLikeConstraintProperty::Position => self.position = parse_position(value)?,
            CssLikeConstraintProperty::Inset => {
                self.inset = parse_edge_values(value, "inset", true)?
            }
            CssLikeConstraintProperty::Top => set_inset_edge(self, value, "top", InsetEdge::Top)?,
            CssLikeConstraintProperty::Right => {
                set_inset_edge(self, value, "right", InsetEdge::Right)?
            }
            CssLikeConstraintProperty::Bottom => {
                set_inset_edge(self, value, "bottom", InsetEdge::Bottom)?
            }
            CssLikeConstraintProperty::Left => {
                set_inset_edge(self, value, "left", InsetEdge::Left)?
            }
            CssLikeConstraintProperty::Overflow => {
                let values = split_top_level_whitespace(value, "overflow")?;
                let (x, y) = match values.as_slice() {
                    [single] => {
                        let overflow = parse_overflow(single)?;
                        (overflow, overflow)
                    }
                    [x, y] => (parse_overflow(x)?, parse_overflow(y)?),
                    _ => return Err(invalid_value("overflow", value)),
                };
                self.overflow.x = x;
                self.overflow.y = y;
            }
            CssLikeConstraintProperty::OverflowX => self.overflow.x = parse_overflow(value)?,
            CssLikeConstraintProperty::OverflowY => self.overflow.y = parse_overflow(value)?,
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
enum InsetEdge {
    Top,
    Right,
    Bottom,
    Left,
}

fn set_inset_edge(
    constraint: &mut CssLikeConstraint,
    value: &str,
    property: &'static str,
    edge: InsetEdge,
) -> Result<(), CssLikeConstraintError> {
    let dimension = parse_dimension(value, property, true)?;
    match edge {
        InsetEdge::Top => constraint.inset.top = dimension,
        InsetEdge::Right => constraint.inset.right = dimension,
        InsetEdge::Bottom => constraint.inset.bottom = dimension,
        InsetEdge::Left => constraint.inset.left = dimension,
    }
    Ok(())
}

fn parse_display(value: &str) -> Result<UiLayoutDisplay, CssLikeConstraintError> {
    match value {
        "flex" => Ok(UiLayoutDisplay::Flex),
        "grid" => Ok(UiLayoutDisplay::Grid),
        "block" => Ok(UiLayoutDisplay::Block),
        "overlay" => Ok(UiLayoutDisplay::Overlay),
        "canvas" => Ok(UiLayoutDisplay::Canvas),
        "scroll" => Ok(UiLayoutDisplay::Scroll),
        "virtual" => Ok(UiLayoutDisplay::Virtual),
        "none" => Ok(UiLayoutDisplay::None),
        _ => Err(invalid_value("display", value)),
    }
}

fn parse_flex_direction(value: &str) -> Result<UiFlexDirection, CssLikeConstraintError> {
    match value {
        "row" => Ok(UiFlexDirection::Row),
        "column" => Ok(UiFlexDirection::Column),
        "row-reverse" => Ok(UiFlexDirection::RowReverse),
        "column-reverse" => Ok(UiFlexDirection::ColumnReverse),
        _ => Err(invalid_value("flex-direction", value)),
    }
}

fn parse_flex_wrap(value: &str) -> Result<UiFlexWrap, CssLikeConstraintError> {
    match value {
        "nowrap" => Ok(UiFlexWrap::NoWrap),
        "wrap" => Ok(UiFlexWrap::Wrap),
        "wrap-reverse" => Ok(UiFlexWrap::WrapReverse),
        _ => Err(invalid_value("flex-wrap", value)),
    }
}

fn parse_justify(value: &str) -> Result<UiJustify, CssLikeConstraintError> {
    match value {
        "start" | "flex-start" => Ok(UiJustify::Start),
        "end" | "flex-end" => Ok(UiJustify::End),
        "center" => Ok(UiJustify::Center),
        "space-between" => Ok(UiJustify::SpaceBetween),
        "space-around" => Ok(UiJustify::SpaceAround),
        "space-evenly" => Ok(UiJustify::SpaceEvenly),
        _ => Err(invalid_value("justify-content", value)),
    }
}

fn parse_align(value: &str, property: &'static str) -> Result<UiAlign, CssLikeConstraintError> {
    match value {
        "start" | "flex-start" => Ok(UiAlign::Start),
        "end" | "flex-end" => Ok(UiAlign::End),
        "center" => Ok(UiAlign::Center),
        "stretch" => Ok(UiAlign::Stretch),
        "baseline" => Ok(UiAlign::Baseline),
        _ => Err(invalid_value(property, value)),
    }
}

fn parse_grid_tracks(
    value: &str,
    property: &'static str,
) -> Result<Vec<CssLikeGridTrack>, CssLikeConstraintError> {
    if value == "none" {
        return Ok(Vec::new());
    }

    let tracks = split_top_level_whitespace(value, property)?;
    if tracks.is_empty() {
        return Err(invalid_value(property, value));
    }

    tracks
        .into_iter()
        .map(|track| match CssLikeGridTrack::from_author_value(track) {
            Ok(track) => Ok(track),
            Err(error @ CssLikeConstraintError::KnownUnsupportedSyntax { .. }) => Err(error),
            Err(error @ CssLikeConstraintError::KnownUnsupportedUnit { .. }) => Err(error),
            Err(_) => Err(invalid_value(property, value)),
        })
        .collect()
}

fn parse_grid_placement(
    value: &str,
    property: &'static str,
) -> Result<UiGridPlacement, CssLikeConstraintError> {
    let mut lines = value.split('/').map(str::trim);
    let start = lines.next().ok_or_else(|| invalid_value(property, value))?;
    let end = lines.next().unwrap_or("auto");
    if lines.next().is_some() || start.is_empty() || end.is_empty() {
        return Err(invalid_value(property, value));
    }

    Ok(UiGridPlacement {
        start: parse_grid_line(start, property)?,
        end: parse_grid_line(end, property)?,
    })
}

fn parse_grid_line(
    value: &str,
    property: &'static str,
) -> Result<UiGridLine, CssLikeConstraintError> {
    if value == "auto" {
        return Ok(UiGridLine::Auto);
    }

    if let Some(span) = value.strip_prefix("span ") {
        let span = span
            .trim()
            .parse::<u16>()
            .map_err(|_| invalid_value(property, value))?;
        return (span > 0)
            .then_some(UiGridLine::Span(span))
            .ok_or_else(|| invalid_value(property, value));
    }

    let line = value
        .parse::<i16>()
        .map_err(|_| invalid_value(property, value))?;
    (line != 0)
        .then_some(UiGridLine::Line(line))
        .ok_or_else(|| invalid_value(property, value))
}

fn parse_dimension(
    value: &str,
    property: &'static str,
    allow_auto: bool,
) -> Result<CssLikeDimension, CssLikeConstraintError> {
    let dimension = match CssLikeDimension::from_str(value) {
        Ok(dimension) => dimension,
        Err(error @ CssLikeConstraintError::KnownUnsupportedUnit { .. }) => return Err(error),
        Err(_) => return Err(invalid_value(property, value)),
    };
    if !allow_auto && dimension == CssLikeDimension::Auto {
        return Err(CssLikeConstraintError::AutoNotAllowed { property });
    }
    Ok(dimension)
}

fn parse_non_negative_number(
    value: &str,
    property: &'static str,
) -> Result<f32, CssLikeConstraintError> {
    let value = parse_author_number(value, value).map_err(|_| invalid_value(property, value))?;
    finite_non_negative(value, property)
}

fn parse_aspect_ratio(value: &str) -> Result<f32, CssLikeConstraintError> {
    let mut parts = value.split('/').map(str::trim);
    let first = parts.next().unwrap_or_default();
    let second = parts.next();
    if parts.next().is_some() {
        return Err(invalid_value("aspect-ratio", value));
    }

    let ratio = match second {
        None if !first.is_empty() => parse_non_negative_number(first, "aspect-ratio")?,
        Some(denominator) if !first.is_empty() && !denominator.is_empty() => {
            let numerator = parse_non_negative_number(first, "aspect-ratio")?;
            let denominator = parse_non_negative_number(denominator, "aspect-ratio")?;
            if denominator == 0.0 {
                return Err(invalid_value("aspect-ratio", value));
            }
            numerator / denominator
        }
        _ => return Err(invalid_value("aspect-ratio", value)),
    };
    finite_non_negative(ratio, "aspect-ratio")
}

fn parse_edge_values(
    value: &str,
    property: &'static str,
    allow_auto: bool,
) -> Result<CssLikeEdges, CssLikeConstraintError> {
    let values = split_top_level_whitespace(value, property)?;
    let dimensions = values
        .iter()
        .map(|value| parse_dimension(value, property, allow_auto))
        .collect::<Result<Vec<_>, _>>()?;

    match dimensions.as_slice() {
        [all] => Ok(CssLikeEdges::all(all.clone())),
        [vertical, horizontal] => Ok(CssLikeEdges {
            top: vertical.clone(),
            right: horizontal.clone(),
            bottom: vertical.clone(),
            left: horizontal.clone(),
        }),
        [top, horizontal, bottom] => Ok(CssLikeEdges {
            top: top.clone(),
            right: horizontal.clone(),
            bottom: bottom.clone(),
            left: horizontal.clone(),
        }),
        [top, right, bottom, left] => Ok(CssLikeEdges {
            top: top.clone(),
            right: right.clone(),
            bottom: bottom.clone(),
            left: left.clone(),
        }),
        _ => Err(invalid_value(property, value)),
    }
}

fn parse_position(value: &str) -> Result<UiPositionMode, CssLikeConstraintError> {
    match value {
        "relative" => Ok(UiPositionMode::Relative),
        "absolute" => Ok(UiPositionMode::Absolute),
        _ => Err(invalid_value("position", value)),
    }
}

fn parse_overflow(value: &str) -> Result<CssLikeOverflow, CssLikeConstraintError> {
    match CssLikeOverflow::from_str(value) {
        Ok(overflow) => Ok(overflow),
        Err(error @ CssLikeConstraintError::KnownUnsupportedSyntax { .. }) => Err(error),
        Err(_) => Err(invalid_value("overflow", value)),
    }
}

fn split_top_level_whitespace<'a>(
    value: &'a str,
    property: &'static str,
) -> Result<Vec<&'a str>, CssLikeConstraintError> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = None;

    for (index, character) in value.char_indices() {
        match character {
            '(' => {
                depth += 1;
                start.get_or_insert(index);
            }
            ')' => {
                if depth == 0 {
                    return Err(invalid_value(property, value));
                }
                depth -= 1;
            }
            character if character.is_whitespace() && depth == 0 => {
                if let Some(start_index) = start.take() {
                    parts.push(&value[start_index..index]);
                }
            }
            _ => {
                start.get_or_insert(index);
            }
        }
    }

    if depth != 0 {
        return Err(invalid_value(property, value));
    }
    if let Some(start_index) = start {
        parts.push(&value[start_index..]);
    }

    Ok(parts)
}

fn invalid_value(property: &'static str, value: &str) -> CssLikeConstraintError {
    CssLikeConstraintError::InvalidValue {
        property,
        value: value.to_owned(),
    }
}

#[cfg(test)]
#[path = "declaration_parser/aspect_ratio_tests.rs"]
mod aspect_ratio_tests;
