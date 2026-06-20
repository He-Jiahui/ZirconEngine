mod commands;

#[cfg(test)]
mod test_frame;

pub(in crate::ui::retained_host::host_contract) use commands::draw_workbench_presentation_commands;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use test_frame::{
    paint_host_frame, repaint_host_frame_region,
};
