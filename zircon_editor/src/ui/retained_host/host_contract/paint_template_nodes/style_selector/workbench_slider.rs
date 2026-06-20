mod colors;
mod model;
mod palette;
mod selection;
mod state;
mod text;
mod thumb;
mod track;
mod value;

#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::WorkbenchSliderStyle;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_SLIDER_HALO, WORKBENCH_SLIDER_TEXT, WORKBENCH_SLIDER_THUMB, WORKBENCH_SLIDER_TICK,
    WORKBENCH_SLIDER_TRACK, WORKBENCH_SLIDER_TRACK_DISABLED,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_slider_style;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use state::is_workbench_slider_state_hot;
