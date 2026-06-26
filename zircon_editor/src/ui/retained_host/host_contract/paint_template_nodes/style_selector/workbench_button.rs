mod brightness;
mod command;
mod model;
mod palette;
mod selection;
mod states;
mod tab_like;
#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchButtonKind, WorkbenchButtonStyle,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    ADD_COMPONENT_GLYPH, ADD_COMPONENT_TEXT, OUTLINED_BORDER, OUTLINED_SURFACE, OUTLINED_TEXT,
    PRIMARY_SURFACE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_button_style;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use tab_like::is_tab_like_workbench_button;
