mod background;
mod border;
mod colors;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use background::axis_field_background;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use border::{
    axis_field_border, axis_field_border_width,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use colors::{
    AXIS_FIELD_BACKGROUND, AXIS_FIELD_BORDER, AXIS_FIELD_DISABLED_BACKGROUND,
    AXIS_FIELD_DISABLED_BORDER, AXIS_FIELD_HOVER_BACKGROUND, AXIS_FIELD_HOVER_BORDER,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::axis_field_text_color;
