//! 节标题图标仅把已识别的工作台语义映射到资源名、尺寸和主题色；节点内容与布局由标题绘制器负责。

#[path = "section_title_glyphs/identity.rs"]
mod identity;
#[path = "section_title_glyphs/metrics.rs"]
mod metrics;
#[path = "section_title_glyphs/shapes.rs"]
mod shapes;
#[path = "section_title_glyphs/style.rs"]
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::{
    section_title_icon, SectionTitleIcon,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    section_title_glyph_metrics, section_title_glyph_metrics_from_host,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use shapes::push_section_icon;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use style::{
    section_icon_color, section_title_glyph_palette_from_host,
};
