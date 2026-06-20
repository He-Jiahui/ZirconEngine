mod anchor;
mod metrics;
mod model;
mod overlay;
mod root_text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use overlay::{
    badge_overlay_frame, badge_overlay_radius, badge_overlay_text_frame,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use root_text::badge_root_text_frame;
