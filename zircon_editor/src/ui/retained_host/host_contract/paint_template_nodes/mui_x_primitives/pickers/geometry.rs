use super::super::super::super::data::FrameRect;
use super::metrics::{
    PICKER_FIELD_HEIGHT_FRACTION, PICKER_FIELD_MIN_HEIGHT, PICKER_ICON_INSET, PICKER_INSET,
    PICKER_POPUP_CELL_COLUMNS, PICKER_POPUP_CELL_MIN_SIZE, PICKER_POPUP_CELL_RESERVED_HEIGHT,
    PICKER_POPUP_CELL_Y_FACTOR, PICKER_POPUP_HEADER_HEIGHT, PICKER_POPUP_MIN_HEIGHT,
};

pub(super) fn picker_field_frame(rect: &FrameRect) -> FrameRect {
    let field_height = (rect.height * PICKER_FIELD_HEIGHT_FRACTION).max(PICKER_FIELD_MIN_HEIGHT);
    FrameRect {
        x: rect.x + PICKER_INSET,
        y: rect.y + PICKER_INSET,
        width: (rect.width - PICKER_INSET * 2.0).max(1.0),
        height: field_height,
    }
}

pub(super) fn picker_field_icon_frame(field: &FrameRect) -> FrameRect {
    FrameRect {
        x: field.x + field.width - field.height + PICKER_ICON_INSET,
        y: field.y + PICKER_ICON_INSET,
        width: (field.height - PICKER_ICON_INSET * 2.0).max(1.0),
        height: (field.height - PICKER_ICON_INSET * 2.0).max(1.0),
    }
}

pub(super) fn picker_popup_frame(rect: &FrameRect, field: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + PICKER_INSET,
        y: field.y + field.height + PICKER_INSET,
        width: (rect.width - PICKER_INSET * 2.0).max(1.0),
        height: (rect.y + rect.height - field.y - field.height - PICKER_INSET * 2.0)
            .max(PICKER_POPUP_MIN_HEIGHT),
    }
}

pub(super) fn picker_popup_header_frame(layout: &FrameRect) -> FrameRect {
    FrameRect {
        x: layout.x,
        y: layout.y,
        width: layout.width,
        height: PICKER_POPUP_HEADER_HEIGHT.min(layout.height),
    }
}

pub(super) fn picker_popup_cell_frame(layout: &FrameRect) -> FrameRect {
    let cell_size = (layout.width / PICKER_POPUP_CELL_COLUMNS)
        .min(layout.height - PICKER_POPUP_CELL_RESERVED_HEIGHT)
        .max(PICKER_POPUP_CELL_MIN_SIZE);
    FrameRect {
        x: layout.x + layout.width * 0.5 - cell_size * 0.5,
        y: layout.y + layout.height * PICKER_POPUP_CELL_Y_FACTOR - cell_size * 0.5,
        width: cell_size,
        height: cell_size,
    }
}
