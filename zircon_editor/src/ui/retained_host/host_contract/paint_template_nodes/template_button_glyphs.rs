//! 为普通按钮内容提供有限的语义图标和共享密度尺寸；实际像素由 template_icon_assets 统一加载。

mod identity;
mod metrics;
mod shapes;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::{
    button_glyph_for_key, ButtonGlyph,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::button_icon_size;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::button_icon_size_from_host;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use shapes::push_button_glyph;
