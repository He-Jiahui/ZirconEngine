#[path = "text_field/colors.rs"]
mod colors;
#[path = "text_field/model.rs"]
mod model;
#[path = "text_field/palette.rs"]
mod palette;
#[path = "text_field/selection.rs"]
mod selection;
#[path = "text_field/state.rs"]
mod state;
#[path = "text_field/surface.rs"]
mod surface;
#[path = "text_field/text.rs"]
mod text;

#[cfg(test)]
#[path = "text_field/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::WorkbenchTextFieldStyle;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_TEXT_FIELD_BORDER, WORKBENCH_TEXT_FIELD_DISABLED_BORDER,
    WORKBENCH_TEXT_FIELD_DISABLED_SURFACE, WORKBENCH_TEXT_FIELD_DISABLED_TEXT,
    WORKBENCH_TEXT_FIELD_FOCUSED_BORDER, WORKBENCH_TEXT_FIELD_FOCUSED_SURFACE,
    WORKBENCH_TEXT_FIELD_PLACEHOLDER, WORKBENCH_TEXT_FIELD_STEPPER_DIVIDER,
    WORKBENCH_TEXT_FIELD_SURFACE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_text_field_style;
