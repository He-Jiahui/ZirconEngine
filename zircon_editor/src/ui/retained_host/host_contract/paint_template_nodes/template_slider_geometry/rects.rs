mod alignment;
mod range;
mod track;
mod value;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use alignment::{
    centered_rect, pixel_aligned_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use range::slider_range_min_value_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use track::slider_track_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use value::slider_value_rect;
