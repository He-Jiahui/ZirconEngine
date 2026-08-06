mod dispatch;
mod node;
mod surface;

pub(super) use dispatch::is_dispatchable;
pub(in crate::ui::retained_host::host_contract) use surface::build_template_surface_frame;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use surface::{
    reset_template_surface_frame_build_count, template_surface_frame_build_count,
};
