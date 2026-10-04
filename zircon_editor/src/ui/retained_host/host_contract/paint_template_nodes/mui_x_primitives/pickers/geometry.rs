use super::super::super::super::{data::FrameRect, paint_geometry::bounded_extent};
use super::metrics::{
    PICKER_FIELD_HEIGHT_FRACTION, PICKER_FIELD_MIN_HEIGHT, PICKER_ICON_INSET, PICKER_INSET,
    PICKER_POPUP_CELL_COLUMNS, PICKER_POPUP_CELL_MIN_SIZE, PICKER_POPUP_CELL_RESERVED_HEIGHT,
    PICKER_POPUP_CELL_Y_FACTOR, PICKER_POPUP_HEADER_HEIGHT, PICKER_POPUP_MIN_HEIGHT,
};

pub(super) fn picker_field_frame(rect: &FrameRect) -> FrameRect {
    let rect = picker_root_frame(rect);
    let available_width = bounded_extent(rect.width - PICKER_INSET * 2.0);
    let available_height = bounded_extent(rect.height - PICKER_INSET * 2.0);
    let field_height = if available_width <= 0.0 || available_height < PICKER_FIELD_MIN_HEIGHT {
        0.0
    } else {
        bounded_extent(rect.height * PICKER_FIELD_HEIGHT_FRACTION)
            .max(PICKER_FIELD_MIN_HEIGHT)
            .min(available_height)
    };
    FrameRect {
        x: finite_offset(rect.x, PICKER_INSET),
        y: finite_offset(rect.y, PICKER_INSET),
        width: if field_height > 0.0 {
            available_width
        } else {
            0.0
        },
        height: field_height,
    }
}

pub(super) fn picker_field_icon_frame(field: &FrameRect) -> FrameRect {
    let field = picker_root_frame(field);
    let available_width = bounded_extent(field.width - PICKER_ICON_INSET * 2.0);
    let available_height = bounded_extent(field.height - PICKER_ICON_INSET * 2.0);
    let size = available_width.min(available_height);
    FrameRect {
        x: finite_offset(
            field.x,
            bounded_extent(field.width - PICKER_ICON_INSET - size),
        ),
        y: finite_offset(field.y, PICKER_ICON_INSET),
        width: size,
        height: size,
    }
}

/// 弹层预览由字段框定位；尺寸在窄布局下受节点范围约束，供后续单元格排列复用。
pub(super) fn picker_popup_frame(rect: &FrameRect, field: &FrameRect) -> FrameRect {
    let rect = picker_root_frame(rect);
    let field = picker_root_frame(field);
    let available_width = bounded_extent(rect.width - PICKER_INSET * 2.0);
    let popup_y = finite_offset(finite_offset(field.y, field.height), PICKER_INSET);
    let popup_bottom = finite_offset(rect.y, rect.height);
    let available_height = bounded_extent(popup_bottom - popup_y - PICKER_INSET);
    let height = if available_width <= 0.0 || available_height < PICKER_POPUP_MIN_HEIGHT {
        0.0
    } else {
        available_height
    };
    FrameRect {
        x: finite_offset(rect.x, PICKER_INSET),
        y: popup_y,
        width: if height > 0.0 { available_width } else { 0.0 },
        height,
    }
}

pub(super) fn picker_popup_header_frame(layout: &FrameRect) -> FrameRect {
    let layout = picker_root_frame(layout);
    FrameRect {
        x: layout.x,
        y: layout.y,
        width: bounded_extent(layout.width),
        height: bounded_extent(PICKER_POPUP_HEADER_HEIGHT).min(bounded_extent(layout.height)),
    }
}

pub(super) fn picker_popup_cell_frame(layout: &FrameRect) -> FrameRect {
    let layout = picker_root_frame(layout);
    let available_width = bounded_extent(layout.width) / PICKER_POPUP_CELL_COLUMNS;
    let available_height = bounded_extent(layout.height - PICKER_POPUP_CELL_RESERVED_HEIGHT);
    let available_size = available_width.min(available_height);
    let cell_size = (available_size >= PICKER_POPUP_CELL_MIN_SIZE)
        .then_some(available_size)
        .unwrap_or(0.0);
    FrameRect {
        x: finite_offset(
            finite_offset(layout.x, bounded_extent(layout.width) * 0.5),
            -cell_size * 0.5,
        ),
        y: finite_offset(
            finite_offset(
                layout.y,
                bounded_extent(layout.height) * PICKER_POPUP_CELL_Y_FACTOR,
            ),
            -cell_size * 0.5,
        ),
        width: cell_size,
        height: cell_size,
    }
}

pub(super) fn picker_root_frame(rect: &FrameRect) -> FrameRect {
    let x = finite_coordinate(rect.x);
    let y = finite_coordinate(rect.y);
    let width = bounded_extent(rect.width);
    let height = bounded_extent(rect.height);
    FrameRect {
        x,
        y,
        width: extent_that_fits_origin(x, width),
        height: extent_that_fits_origin(y, height),
    }
}

fn finite_coordinate(value: f32) -> f32 {
    value.is_finite().then_some(value).unwrap_or(0.0)
}

fn finite_offset(origin: f32, offset: f32) -> f32 {
    finite_coordinate(finite_coordinate(origin) + finite_coordinate(offset))
}

fn extent_that_fits_origin(origin: f32, extent: f32) -> f32 {
    (finite_coordinate(origin) + bounded_extent(extent))
        .is_finite()
        .then_some(bounded_extent(extent))
        .unwrap_or(0.0)
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
