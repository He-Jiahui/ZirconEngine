use taffy::style::{Dimension, Display, FlexDirection, LengthPercentage, Style};
use zircon_runtime_interface::ui::layout::{
    AxisConstraint, BoxConstraints, UiContainerKind, UiLayoutEngineFamily, UiLayoutEngineRequest,
};

/// Converts the subset of Zircon-owned layout contracts that can be solved by Taffy.
/// Overlay, Canvas, Popup, and VirtualList stay outside this bridge by design.
pub fn taffy_style_for_container(
    container: UiContainerKind,
    constraints: BoxConstraints,
) -> Option<Style> {
    let request = UiLayoutEngineRequest::from_container_kind(container);
    taffy_owned_family(request.family)?;
    taffy_style_inputs_are_finite(container, constraints)?;

    let mut style = Style {
        display: taffy_display_for_family(request.family)?,
        size: taffy::Size {
            width: dimension_for_axis(constraints.width.preferred),
            height: dimension_for_axis(constraints.height.preferred),
        },
        min_size: taffy::Size {
            width: dimension_for_axis(constraints.width.min),
            height: dimension_for_axis(constraints.height.min),
        },
        max_size: taffy::Size {
            width: optional_dimension_for_axis(constraints.width.max),
            height: optional_dimension_for_axis(constraints.height.max),
        },
        ..Style::default()
    };

    match container {
        UiContainerKind::HorizontalBox(config) => {
            style.flex_direction = FlexDirection::Row;
            style.gap = taffy::Size {
                width: LengthPercentage::length(config.gap.max(0.0)),
                height: LengthPercentage::length(0.0),
            };
        }
        UiContainerKind::VerticalBox(config) => {
            style.flex_direction = FlexDirection::Column;
            style.gap = taffy::Size {
                width: LengthPercentage::length(0.0),
                height: LengthPercentage::length(config.gap.max(0.0)),
            };
        }
        UiContainerKind::WrapBox(config) => {
            style.display = Display::Flex;
            style.flex_wrap = taffy::style::FlexWrap::Wrap;
            style.gap = taffy::Size {
                width: LengthPercentage::length(config.horizontal_gap.max(0.0)),
                height: LengthPercentage::length(config.vertical_gap.max(0.0)),
            };
        }
        UiContainerKind::GridBox(config) => {
            style.display = Display::Grid;
            style.gap = taffy::Size {
                width: LengthPercentage::length(config.column_gap.max(0.0)),
                height: LengthPercentage::length(config.row_gap.max(0.0)),
            };
        }
        _ => {}
    }

    Some(style)
}

fn taffy_style_inputs_are_finite(
    container: UiContainerKind,
    constraints: BoxConstraints,
) -> Option<()> {
    (axis_input_values_are_finite(constraints.width)
        && axis_input_values_are_finite(constraints.height)
        && container_style_values_are_finite(container))
    .then_some(())
}

fn axis_input_values_are_finite(constraint: AxisConstraint) -> bool {
    constraint.min.is_finite() && constraint.max.is_finite() && constraint.preferred.is_finite()
}

fn container_style_values_are_finite(container: UiContainerKind) -> bool {
    match container {
        UiContainerKind::HorizontalBox(config) | UiContainerKind::VerticalBox(config) => {
            config.gap.is_finite()
        }
        UiContainerKind::WrapBox(config) => {
            config.horizontal_gap.is_finite()
                && config.vertical_gap.is_finite()
                && config.item_min_width.is_finite()
        }
        UiContainerKind::GridBox(config) => {
            config.column_gap.is_finite() && config.row_gap.is_finite()
        }
        _ => true,
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

fn taffy_owned_family(family: UiLayoutEngineFamily) -> Option<()> {
    matches!(
        family,
        UiLayoutEngineFamily::Flex
            | UiLayoutEngineFamily::Grid
            | UiLayoutEngineFamily::Block
            | UiLayoutEngineFamily::Wrap
    )
    .then_some(())
}

fn dimension_for_axis(value: f32) -> Dimension {
    if value > 0.0 {
        Dimension::length(value)
    } else {
        Dimension::auto()
    }
}

fn optional_dimension_for_axis(value: f32) -> Dimension {
    if value > 0.0 {
        Dimension::length(value)
    } else {
        Dimension::auto()
    }
}
