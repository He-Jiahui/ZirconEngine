use crate::ui::layout::taffy_style_from_ui_layout_style;
use taffy::style::{
    AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap, JustifyContent,
    LengthPercentage, LengthPercentageAuto, Overflow, Position,
};
use zircon_runtime_interface::ui::layout::{
    UiAlign, UiDimension, UiEdges, UiFlexDirection, UiFlexWrap, UiGap, UiGridLine, UiGridPlacement,
    UiGridTrack, UiGridTrackBreadth, UiJustify, UiLayoutDisplay, UiLayoutSize, UiLayoutStyle,
    UiOverflow, UiOverflowPair, UiPositionMode,
};

#[test]
fn ui_layout_style_maps_core_css_fields_to_taffy() {
    let style = UiLayoutStyle {
        display: UiLayoutDisplay::Flex,
        direction: UiFlexDirection::ColumnReverse,
        wrap: UiFlexWrap::Wrap,
        justify_content: Some(UiJustify::SpaceBetween),
        align_items: Some(UiAlign::Stretch),
        align_self: Some(UiAlign::Center),
        gap: UiGap {
            row: UiDimension::Px(8.0),
            column: UiDimension::Percent(0.25),
        },
        flex_grow: 2.0,
        flex_shrink: 0.5,
        flex_basis: UiDimension::Px(96.0),
        size: UiLayoutSize {
            width: UiDimension::Percent(1.0),
            height: UiDimension::Px(44.0),
        },
        min_size: UiLayoutSize {
            width: UiDimension::Px(24.0),
            height: UiDimension::Auto,
        },
        max_size: UiLayoutSize {
            width: UiDimension::Auto,
            height: UiDimension::Px(120.0),
        },
        margin: UiEdges {
            left: UiDimension::Px(1.0),
            right: UiDimension::Px(2.0),
            top: UiDimension::Px(3.0),
            bottom: UiDimension::Px(4.0),
        },
        padding: UiEdges {
            left: UiDimension::Px(5.0),
            right: UiDimension::Px(6.0),
            top: UiDimension::Px(7.0),
            bottom: UiDimension::Px(8.0),
        },
        position: UiPositionMode::Absolute,
        inset: UiEdges {
            left: UiDimension::Px(9.0),
            right: UiDimension::Auto,
            top: UiDimension::Px(10.0),
            bottom: UiDimension::Auto,
        },
        overflow: UiOverflowPair {
            x: UiOverflow::Hidden,
            y: UiOverflow::Scroll,
        },
        ..UiLayoutStyle::default()
    };

    let taffy = taffy_style_from_ui_layout_style(&style).unwrap();

    assert_eq!(taffy.display, Display::Flex);
    assert_eq!(taffy.flex_direction, FlexDirection::ColumnReverse);
    assert_eq!(taffy.flex_wrap, FlexWrap::Wrap);
    assert_eq!(taffy.justify_content, Some(JustifyContent::SpaceBetween));
    assert_eq!(taffy.align_items, Some(AlignItems::Stretch));
    assert_eq!(taffy.align_self, Some(AlignSelf::Center));
    assert_eq!(taffy.gap.width, LengthPercentage::percent(0.25));
    assert_eq!(taffy.gap.height, LengthPercentage::length(8.0));
    assert_eq!(taffy.flex_grow, 2.0);
    assert_eq!(taffy.flex_shrink, 0.5);
    assert_eq!(taffy.flex_basis, Dimension::length(96.0));
    assert_eq!(taffy.size.width, Dimension::percent(1.0));
    assert_eq!(taffy.size.height, Dimension::length(44.0));
    assert_eq!(taffy.min_size.width, Dimension::length(24.0));
    assert_eq!(taffy.max_size.height, Dimension::length(120.0));
    assert_eq!(taffy.margin.left, LengthPercentageAuto::length(1.0));
    assert_eq!(taffy.padding.bottom, LengthPercentage::length(8.0));
    assert_eq!(taffy.position, Position::Absolute);
    assert_eq!(taffy.inset.left, LengthPercentageAuto::length(9.0));
    assert_eq!(taffy.overflow.x, Overflow::Hidden);
    assert_eq!(taffy.overflow.y, Overflow::Scroll);
}

#[test]
fn ui_layout_style_maps_grid_tracks_and_placements() {
    let style = UiLayoutStyle {
        display: UiLayoutDisplay::Grid,
        grid_template_columns: vec![
            UiGridTrack::Px(80.0),
            UiGridTrack::Fr(1.0),
            UiGridTrack::MinMax {
                min: UiGridTrackBreadth::Px(120.0),
                max: UiGridTrackBreadth::Fr(2.0),
            },
        ],
        grid_template_rows: vec![UiGridTrack::Auto, UiGridTrack::Percent(0.5)],
        grid_column: Some(UiGridPlacement {
            start: UiGridLine::Line(1),
            end: UiGridLine::Span(2),
        }),
        grid_row: Some(UiGridPlacement {
            start: UiGridLine::Auto,
            end: UiGridLine::Line(3),
        }),
        ..UiLayoutStyle::default()
    };

    let taffy = taffy_style_from_ui_layout_style(&style).unwrap();

    assert_eq!(taffy.display, Display::Grid);
    assert_eq!(taffy.grid_template_columns.len(), 3);
    assert_eq!(taffy.grid_template_rows.len(), 2);
    assert_eq!(
        taffy.grid_column.start,
        taffy::style::GridPlacement::Line(1.into())
    );
    assert!(matches!(
        taffy.grid_column.end,
        taffy::style::GridPlacement::Span(2)
    ));
    assert!(matches!(
        taffy.grid_row.start,
        taffy::style::GridPlacement::Auto
    ));
    assert_eq!(
        taffy.grid_row.end,
        taffy::style::GridPlacement::Line(3.into())
    );
}
