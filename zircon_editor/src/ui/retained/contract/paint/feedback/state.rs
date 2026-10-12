#[path = "state/colors.rs"]
mod colors;
#[path = "state/identity.rs"]
mod identity;
#[path = "state/mode.rs"]
mod mode;
#[path = "state/tone.rs"]
mod tone;
#[path = "state/values.rs"]
mod values;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use colors::{
    progress_fill_color, progress_track_color,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::is_material_progress_node;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use mode::{
    progress_is_circular, progress_is_indeterminate,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use values::progress_percent;
