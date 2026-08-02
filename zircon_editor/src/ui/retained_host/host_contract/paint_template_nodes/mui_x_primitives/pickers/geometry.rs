use super::super::super::super::{data::FrameRect, paint_geometry::bounded_extent};
use super::metrics::{
    PICKER_FIELD_HEIGHT_FRACTION, PICKER_FIELD_MIN_HEIGHT, PICKER_ICON_INSET, PICKER_INSET,
    PICKER_POPUP_CELL_COLUMNS, PICKER_POPUP_CELL_MIN_SIZE, PICKER_POPUP_CELL_RESERVED_HEIGHT,
    PICKER_POPUP_CELL_Y_FACTOR, PICKER_POPUP_HEADER_HEIGHT, PICKER_POPUP_MIN_HEIGHT,
};

pub(super) fn picker_field_frame(rect: &FrameRect) -> FrameRect {
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
        x: rect.x + PICKER_INSET,
        y: rect.y + PICKER_INSET,
        width: if field_height > 0.0 {
            available_width
        } else {
            0.0
        },
        height: field_height,
    }
}

pub(super) fn picker_field_icon_frame(field: &FrameRect) -> FrameRect {
    let available_width = bounded_extent(field.width - PICKER_ICON_INSET * 2.0);
    let available_height = bounded_extent(field.height - PICKER_ICON_INSET * 2.0);
    let size = available_width.min(available_height);
    FrameRect {
        x: field.x + bounded_extent(field.width - PICKER_ICON_INSET - size),
        y: field.y + PICKER_ICON_INSET,
        width: size,
        height: size,
    }
}

pub(super) fn picker_popup_frame(rect: &FrameRect, field: &FrameRect) -> FrameRect {
    let available_width = bounded_extent(rect.width - PICKER_INSET * 2.0);
    let available_height = bounded_extent(
        rect.y + bounded_extent(rect.height)
            - field.y
            - bounded_extent(field.height)
            - PICKER_INSET * 2.0,
    );
    let height = if available_width <= 0.0 || available_height < PICKER_POPUP_MIN_HEIGHT {
        0.0
    } else {
        available_height
    };
    FrameRect {
        x: rect.x + PICKER_INSET,
        y: field.y + field.height + PICKER_INSET,
        width: if height > 0.0 { available_width } else { 0.0 },
        height,
    }
}

pub(super) fn picker_popup_header_frame(layout: &FrameRect) -> FrameRect {
    FrameRect {
        x: layout.x,
        y: layout.y,
        width: bounded_extent(layout.width),
        height: bounded_extent(PICKER_POPUP_HEADER_HEIGHT).min(bounded_extent(layout.height)),
    }
}

pub(super) fn picker_popup_cell_frame(layout: &FrameRect) -> FrameRect {
    let available_width = bounded_extent(layout.width) / PICKER_POPUP_CELL_COLUMNS;
    let available_height = bounded_extent(layout.height - PICKER_POPUP_CELL_RESERVED_HEIGHT);
    let available_size = available_width.min(available_height);
    let cell_size = (available_size >= PICKER_POPUP_CELL_MIN_SIZE).then_some(available_size).unwrap_or(0.0);
    FrameRect {
        x: layout.x + bounded_extent(layout.width) * 0.5 - cell_size * 0.5,
        y: layout.y + bounded_extent(layout.height) * PICKER_POPUP_CELL_Y_FACTOR - cell_size * 0.5,
        width: cell_size,
        height: cell_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_picker_surfaces_do_not_expand_into_drawable_frames() {
        let outer = FrameRect {
            x: 12.0,
            y: 8.0,
            width: 0.0,
            height: 0.0,
        };
        let field = picker_field_frame(&outer);
        let icon = picker_field_icon_frame(&field);
        let popup = picker_popup_frame(&outer, &field);
        let header = picker_popup_header_frame(&popup);
        let cell = picker_popup_cell_frame(&popup);

        for frame in [field, icon, popup, header, cell] {
            assert_eq!((frame.width, frame.height), (0.0, 0.0));
        }
    }
}
