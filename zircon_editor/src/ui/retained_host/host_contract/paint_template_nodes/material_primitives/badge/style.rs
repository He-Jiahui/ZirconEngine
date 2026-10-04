// 根表面和计数层使用不同颜色规则；样式汇总只暴露各自绘制命令所需令牌。
mod overlay;
mod root;
mod tokens;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use overlay::{
    badge_overlay_background_color, badge_overlay_border_color, badge_overlay_border_width,
    badge_overlay_text_color,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use root::{
    badge_root_background_color, badge_root_border_color, badge_root_border_width,
    badge_root_text_color,
};
