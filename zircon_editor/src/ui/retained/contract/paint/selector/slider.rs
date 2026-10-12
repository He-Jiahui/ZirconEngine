#[path = "slider/colors.rs"]
mod colors;
#[path = "slider/model.rs"]
mod model;
#[path = "slider/palette.rs"]
mod palette;
#[path = "slider/selection.rs"]
mod selection;
#[path = "slider/state.rs"]
mod state;
#[path = "slider/text.rs"]
mod text;
#[path = "slider/thumb.rs"]
mod thumb;
#[path = "slider/track.rs"]
mod track;
#[path = "slider/value.rs"]
mod value;

#[cfg(test)]
#[path = "slider/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::WorkbenchSliderStyle;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_SLIDER_FILL, WORKBENCH_SLIDER_THUMB, WORKBENCH_SLIDER_TRACK,
    WORKBENCH_SLIDER_TRACK_DISABLED,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_slider_style;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use state::is_workbench_slider_state_hot;
