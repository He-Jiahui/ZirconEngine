mod colors;
mod model;
mod palette;
mod selection;
mod state;
mod surface;
mod text;

#[cfg(test)]
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
