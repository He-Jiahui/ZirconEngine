use taffy::geometry::{Line, Rect, Size};
use taffy::style::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap,
    GridPlacement, GridTemplateComponent, JustifyContent, LengthPercentage, LengthPercentageAuto,
    MaxTrackSizingFunction, MinTrackSizingFunction, Overflow, Position, Style, TrackSizingFunction,
};
use zircon_runtime_interface::ui::layout::{
    AxisConstraint, BoxConstraints, StretchMode, UiAlign, UiContainerKind, UiDimension, UiEdges,
    UiFlexDirection, UiFlexWrap, UiGap, UiGridLine, UiGridPlacement, UiGridTrack,
    UiGridTrackBreadth, UiJustify, UiLayoutDisplay, UiLayoutEngineFallbackReason,
    UiLayoutEngineFamily, UiLayoutSize, UiLayoutStyle, UiOverflow, UiOverflowPair, UiPositionMode,
};

pub fn taffy_style_from_ui_layout_style(
    style: &UiLayoutStyle,
) -> Result<Style, UiLayoutEngineFallbackReason> {
    validate_layout_style(style)?;
    let mut taffy_style = Style {
        display: taffy_display_for_layout_display(style.display)?,
        flex_direction: taffy_flex_direction(style.direction),
        flex_wrap: taffy_flex_wrap(style.wrap),
        justify_content: style.justify_content.map(taffy_justify_content),
        align_items: style.align_items.map(taffy_align_items),
        align_self: style.align_self.map(taffy_align_self),
        align_content: style.align_content.map(taffy_align_content).transpose()?,
        gap: Size {
            width: taffy_length_percentage(style.gap.column)?,
            height: taffy_length_percentage(style.gap.row)?,
        },
        flex_grow: style.flex_grow,
        flex_shrink: style.flex_shrink,
        flex_basis: taffy_dimension(style.flex_basis)?,
        grid_template_columns: style
            .grid_template_columns
            .iter()
            .map(taffy_grid_template_component)
            .collect::<Result<Vec<_>, _>>()?,
        grid_template_rows: style
            .grid_template_rows
            .iter()
            .map(taffy_grid_template_component)
            .collect::<Result<Vec<_>, _>>()?,
        grid_row: style
            .grid_row
            .map(taffy_grid_placement_line)
            .transpose()?
            .unwrap_or_default(),
        grid_column: style
            .grid_column
            .map(taffy_grid_placement_line)
            .transpose()?
            .unwrap_or_default(),
        size: taffy_size(style.size)?,
        min_size: taffy_size(style.min_size)?,
        max_size: taffy_size(style.max_size)?,
        aspect_ratio: style.aspect_ratio,
        margin: taffy_edges_auto(style.margin)?,
        padding: taffy_edges(style.padding)?,
        position: taffy_position(style.position),
        inset: taffy_edges_auto(style.inset)?,
        overflow: taffy_overflow_pair(style.overflow),
        ..Style::default()
    };

    if style.display == UiLayoutDisplay::None {
        taffy_style.display = Display::None;
    }

    Ok(taffy_style)
}

pub fn ui_layout_style_from_container(
    container: UiContainerKind,
    constraints: BoxConstraints,
) -> Result<UiLayoutStyle, UiLayoutEngineFallbackReason> {
    let mut style = UiLayoutStyle {
        display: display_for_container(container)?,
        size: UiLayoutSize {
            width: dimension_for_axis(constraints.width.preferred),
            height: dimension_for_axis(constraints.height.preferred),
        },
        min_size: UiLayoutSize {
            width: dimension_for_axis(constraints.width.min),
            height: dimension_for_axis(constraints.height.min),
        },
        max_size: UiLayoutSize {
            width: optional_dimension_for_axis(constraints.width.max),
            height: optional_dimension_for_axis(constraints.height.max),
        },
        ..UiLayoutStyle::default()
    };

    match container {
        UiContainerKind::HorizontalBox(config) => {
            style.direction = UiFlexDirection::Row;
            style.gap = UiGap {
                row: UiDimension::Px(0.0),
                column: UiDimension::Px(config.gap.max(0.0)),
            };
        }
        UiContainerKind::VerticalBox(config) => {
            style.direction = UiFlexDirection::Column;
            style.gap = UiGap {
                row: UiDimension::Px(config.gap.max(0.0)),
                column: UiDimension::Px(0.0),
            };
        }
        UiContainerKind::WrapBox(config) => {
            style.display = UiLayoutDisplay::Flex;
            style.wrap = UiFlexWrap::Wrap;
            style.gap = UiGap {
                row: UiDimension::Px(config.vertical_gap.max(0.0)),
                column: UiDimension::Px(config.horizontal_gap.max(0.0)),
            };
        }
        UiContainerKind::GridBox(config) => {
            style.display = UiLayoutDisplay::Grid;
            style.gap = UiGap {
                row: UiDimension::Px(config.row_gap.max(0.0)),
                column: UiDimension::Px(config.column_gap.max(0.0)),
            };
        }
        _ => {}
    }

    validate_container_style_inputs(container, constraints)?;
    Ok(style)
}

pub fn ui_layout_style_from_axis_constraints(
    horizontal: AxisConstraint,
    vertical: AxisConstraint,
    stretch: StretchMode,
) -> UiLayoutStyle {
    UiLayoutStyle {
        size: UiLayoutSize {
            width: dimension_for_axis(horizontal.preferred),
            height: dimension_for_axis(vertical.preferred),
        },
        min_size: UiLayoutSize {
            width: dimension_for_axis(horizontal.min),
            height: dimension_for_axis(vertical.min),
        },
        max_size: UiLayoutSize {
            width: optional_dimension_for_axis(horizontal.max),
            height: optional_dimension_for_axis(vertical.max),
        },
        flex_grow: if stretch == StretchMode::Stretch {
            horizontal.weight.max(vertical.weight).max(0.0)
        } else {
            0.0
        },
        flex_shrink: if stretch == StretchMode::Stretch {
            1.0
        } else {
            0.0
        },
        ..UiLayoutStyle::default()
    }
}

pub fn taffy_display_for_family(family: UiLayoutEngineFamily) -> Option<Display> {
    match family {
        UiLayoutEngineFamily::Flex | UiLayoutEngineFamily::Wrap => Some(Display::Flex),
        UiLayoutEngineFamily::Grid => Some(Display::Grid),
        UiLayoutEngineFamily::Block => Some(Display::Block),
        _ => None,
    }
}

fn display_for_container(
    container: UiContainerKind,
) -> Result<UiLayoutDisplay, UiLayoutEngineFallbackReason> {
    match container {
        UiContainerKind::HorizontalBox(_)
        | UiContainerKind::VerticalBox(_)
        | UiContainerKind::WrapBox(_) => Ok(UiLayoutDisplay::Flex),
        UiContainerKind::GridBox(_) => Ok(UiLayoutDisplay::Grid),
        UiContainerKind::BlockBox => Ok(UiLayoutDisplay::Block),
        _ => Err(UiLayoutEngineFallbackReason::ZirconOwnedSemantics),
    }
}

fn taffy_display_for_layout_display(
    display: UiLayoutDisplay,
) -> Result<Display, UiLayoutEngineFallbackReason> {
    match display {
        UiLayoutDisplay::Flex => Ok(Display::Flex),
        UiLayoutDisplay::Grid => Ok(Display::Grid),
        UiLayoutDisplay::Block => Ok(Display::Block),
        UiLayoutDisplay::None => Ok(Display::None),
        UiLayoutDisplay::Overlay
        | UiLayoutDisplay::Canvas
        | UiLayoutDisplay::Scroll
        | UiLayoutDisplay::Virtual => Err(UiLayoutEngineFallbackReason::ZirconOwnedSemantics),
    }
}

fn taffy_flex_direction(direction: UiFlexDirection) -> FlexDirection {
    match direction {
        UiFlexDirection::Row => FlexDirection::Row,
        UiFlexDirection::Column => FlexDirection::Column,
        UiFlexDirection::RowReverse => FlexDirection::RowReverse,
        UiFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
    }
}

fn taffy_flex_wrap(wrap: UiFlexWrap) -> FlexWrap {
    match wrap {
        UiFlexWrap::NoWrap => FlexWrap::NoWrap,
        UiFlexWrap::Wrap => FlexWrap::Wrap,
        UiFlexWrap::WrapReverse => FlexWrap::WrapReverse,
    }
}

fn taffy_justify_content(justify: UiJustify) -> JustifyContent {
    match justify {
        UiJustify::Start => JustifyContent::Start,
        UiJustify::End => JustifyContent::End,
        UiJustify::Center => JustifyContent::Center,
        UiJustify::SpaceBetween => JustifyContent::SpaceBetween,
        UiJustify::SpaceAround => JustifyContent::SpaceAround,
        UiJustify::SpaceEvenly => JustifyContent::SpaceEvenly,
    }
}

fn taffy_align_items(align: UiAlign) -> AlignItems {
    match align {
        UiAlign::Start => AlignItems::Start,
        UiAlign::End => AlignItems::End,
        UiAlign::Center => AlignItems::Center,
        UiAlign::Stretch => AlignItems::Stretch,
        UiAlign::Baseline => AlignItems::Baseline,
    }
}

fn taffy_align_self(align: UiAlign) -> AlignSelf {
    match align {
        UiAlign::Start => AlignSelf::Start,
        UiAlign::End => AlignSelf::End,
        UiAlign::Center => AlignSelf::Center,
        UiAlign::Stretch => AlignSelf::Stretch,
        UiAlign::Baseline => AlignSelf::Baseline,
    }
}

fn taffy_align_content(align: UiAlign) -> Result<AlignContent, UiLayoutEngineFallbackReason> {
    match align {
        UiAlign::Start => Ok(AlignContent::Start),
        UiAlign::End => Ok(AlignContent::End),
        UiAlign::Center => Ok(AlignContent::Center),
        UiAlign::Stretch => Ok(AlignContent::Stretch),
        UiAlign::Baseline => Err(UiLayoutEngineFallbackReason::InvalidLayoutValue),
    }
}

fn taffy_size(size: UiLayoutSize) -> Result<Size<Dimension>, UiLayoutEngineFallbackReason> {
    Ok(Size {
        width: taffy_dimension(size.width)?,
        height: taffy_dimension(size.height)?,
    })
}

fn taffy_dimension(dimension: UiDimension) -> Result<Dimension, UiLayoutEngineFallbackReason> {
    match dimension {
        UiDimension::Auto => Ok(Dimension::auto()),
        UiDimension::Px(value) => finite_non_negative(value).map(Dimension::length),
        UiDimension::Percent(value) => finite_non_negative(value).map(Dimension::percent),
    }
}

fn taffy_length_percentage(
    dimension: UiDimension,
) -> Result<LengthPercentage, UiLayoutEngineFallbackReason> {
    match dimension {
        UiDimension::Auto => Err(UiLayoutEngineFallbackReason::InvalidLayoutValue),
        UiDimension::Px(value) => finite_non_negative(value).map(LengthPercentage::length),
        UiDimension::Percent(value) => finite_non_negative(value).map(LengthPercentage::percent),
    }
}

fn taffy_length_percentage_auto(
    dimension: UiDimension,
) -> Result<LengthPercentageAuto, UiLayoutEngineFallbackReason> {
    match dimension {
        UiDimension::Auto => Ok(LengthPercentageAuto::auto()),
        UiDimension::Px(value) => finite_non_negative(value).map(LengthPercentageAuto::length),
        UiDimension::Percent(value) => {
            finite_non_negative(value).map(LengthPercentageAuto::percent)
        }
    }
}

fn taffy_edges(edges: UiEdges) -> Result<Rect<LengthPercentage>, UiLayoutEngineFallbackReason> {
    Ok(Rect {
        left: taffy_length_percentage(edges.left)?,
        right: taffy_length_percentage(edges.right)?,
        top: taffy_length_percentage(edges.top)?,
        bottom: taffy_length_percentage(edges.bottom)?,
    })
}

fn taffy_edges_auto(
    edges: UiEdges,
) -> Result<Rect<LengthPercentageAuto>, UiLayoutEngineFallbackReason> {
    Ok(Rect {
        left: taffy_length_percentage_auto(edges.left)?,
        right: taffy_length_percentage_auto(edges.right)?,
        top: taffy_length_percentage_auto(edges.top)?,
        bottom: taffy_length_percentage_auto(edges.bottom)?,
    })
}

fn taffy_position(position: UiPositionMode) -> Position {
    match position {
        UiPositionMode::Relative => Position::Relative,
        UiPositionMode::Absolute => Position::Absolute,
    }
}

fn taffy_overflow_pair(overflow: UiOverflowPair) -> taffy::geometry::Point<Overflow> {
    taffy::geometry::Point {
        x: taffy_overflow(overflow.x),
        y: taffy_overflow(overflow.y),
    }
}

fn taffy_overflow(overflow: UiOverflow) -> Overflow {
    match overflow {
        UiOverflow::Visible => Overflow::Visible,
        UiOverflow::Hidden => Overflow::Hidden,
        UiOverflow::Scroll => Overflow::Scroll,
    }
}

fn taffy_grid_template_component(
    track: &UiGridTrack,
) -> Result<GridTemplateComponent<String>, UiLayoutEngineFallbackReason> {
    Ok(GridTemplateComponent::Single(taffy_grid_track(track)?))
}

fn taffy_grid_track(
    track: &UiGridTrack,
) -> Result<TrackSizingFunction, UiLayoutEngineFallbackReason> {
    Ok(match track {
        UiGridTrack::Px(value) => TrackSizingFunction {
            min: MinTrackSizingFunction::length(finite_non_negative(*value)?),
            max: MaxTrackSizingFunction::length(finite_non_negative(*value)?),
        },
        UiGridTrack::Percent(value) => TrackSizingFunction {
            min: MinTrackSizingFunction::percent(finite_non_negative(*value)?),
            max: MaxTrackSizingFunction::percent(finite_non_negative(*value)?),
        },
        UiGridTrack::Fr(value) => TrackSizingFunction {
            min: MinTrackSizingFunction::auto(),
            max: MaxTrackSizingFunction::fr(finite_non_negative(*value)?),
        },
        UiGridTrack::Auto => TrackSizingFunction {
            min: MinTrackSizingFunction::auto(),
            max: MaxTrackSizingFunction::auto(),
        },
        UiGridTrack::MinMax { min, max } => TrackSizingFunction {
            min: taffy_grid_min_track(*min)?,
            max: taffy_grid_max_track(*max)?,
        },
    })
}

fn taffy_grid_min_track(
    track: UiGridTrackBreadth,
) -> Result<MinTrackSizingFunction, UiLayoutEngineFallbackReason> {
    Ok(match track {
        UiGridTrackBreadth::Px(value) => {
            MinTrackSizingFunction::length(finite_non_negative(value)?)
        }
        UiGridTrackBreadth::Percent(value) => {
            MinTrackSizingFunction::percent(finite_non_negative(value)?)
        }
        UiGridTrackBreadth::Fr(_) => MinTrackSizingFunction::auto(),
        UiGridTrackBreadth::Auto => MinTrackSizingFunction::auto(),
    })
}

fn taffy_grid_max_track(
    track: UiGridTrackBreadth,
) -> Result<MaxTrackSizingFunction, UiLayoutEngineFallbackReason> {
    Ok(match track {
        UiGridTrackBreadth::Px(value) => {
            MaxTrackSizingFunction::length(finite_non_negative(value)?)
        }
        UiGridTrackBreadth::Percent(value) => {
            MaxTrackSizingFunction::percent(finite_non_negative(value)?)
        }
        UiGridTrackBreadth::Fr(value) => MaxTrackSizingFunction::fr(finite_non_negative(value)?),
        UiGridTrackBreadth::Auto => MaxTrackSizingFunction::auto(),
    })
}

fn taffy_grid_placement_line(
    placement: UiGridPlacement,
) -> Result<Line<GridPlacement<String>>, UiLayoutEngineFallbackReason> {
    Ok(Line {
        start: taffy_grid_line(placement.start)?,
        end: taffy_grid_line(placement.end)?,
    })
}

fn taffy_grid_line(
    line: UiGridLine,
) -> Result<GridPlacement<String>, UiLayoutEngineFallbackReason> {
    Ok(match line {
        UiGridLine::Auto => GridPlacement::Auto,
        UiGridLine::Line(line) => GridPlacement::Line(line.into()),
        UiGridLine::Span(span) => GridPlacement::Span(span),
    })
}

fn validate_layout_style(style: &UiLayoutStyle) -> Result<(), UiLayoutEngineFallbackReason> {
    for value in [
        style.flex_grow,
        style.flex_shrink,
        style.aspect_ratio.unwrap_or(1.0),
    ] {
        finite_non_negative(value)?;
    }
    Ok(())
}

fn validate_container_style_inputs(
    container: UiContainerKind,
    constraints: BoxConstraints,
) -> Result<(), UiLayoutEngineFallbackReason> {
    for constraint in [constraints.width, constraints.height] {
        finite_value(constraint.min)?;
        finite_value(constraint.max)?;
        finite_value(constraint.preferred)?;
    }

    match container {
        UiContainerKind::HorizontalBox(config) | UiContainerKind::VerticalBox(config) => {
            finite_value(config.gap)?;
        }
        UiContainerKind::WrapBox(config) => {
            finite_value(config.horizontal_gap)?;
            finite_value(config.vertical_gap)?;
            finite_value(config.item_min_width)?;
        }
        UiContainerKind::GridBox(config) => {
            finite_value(config.column_gap)?;
            finite_value(config.row_gap)?;
        }
        _ => {}
    }
    Ok(())
}

fn finite_value(value: f32) -> Result<f32, UiLayoutEngineFallbackReason> {
    value
        .is_finite()
        .then_some(value)
        .ok_or(UiLayoutEngineFallbackReason::InvalidLayoutValue)
}

fn finite_non_negative(value: f32) -> Result<f32, UiLayoutEngineFallbackReason> {
    (value.is_finite() && value >= 0.0)
        .then_some(value)
        .ok_or(UiLayoutEngineFallbackReason::InvalidLayoutValue)
}

fn dimension_for_axis(value: f32) -> UiDimension {
    if value > 0.0 {
        UiDimension::Px(value)
    } else {
        UiDimension::Auto
    }
}

fn optional_dimension_for_axis(value: f32) -> UiDimension {
    if value > 0.0 {
        UiDimension::Px(value)
    } else {
        UiDimension::Auto
    }
}
