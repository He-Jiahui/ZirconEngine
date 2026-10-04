//! 状态专用 glyph 不决定身份或布局，仅把已解析的 icon/signal 语义落到资产或简单圆点。

mod geometry;
mod icon_glyphs;
mod signals;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::{
    centered_rect, has_paintable_status_glyph_extent,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use icon_glyphs::{
    push_status_icon_glyph, StatusIconKind,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use signals::push_status_signal_icon;
