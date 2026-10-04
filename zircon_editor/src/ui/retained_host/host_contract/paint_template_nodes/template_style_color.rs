//! 将 runtime typed 颜色和主题角色解析为宿主 RGBA；None 保留上层继承/回退，透明颜色保留明确清空语义。

mod buttons;
mod roles;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use buttons::{
    is_primary_contained_button, typed_button_border_color, typed_button_tone_color,
    typed_button_variant_background,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use roles::resolved_style_color;
