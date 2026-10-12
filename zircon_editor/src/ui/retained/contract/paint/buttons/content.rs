//! 按钮内容域统一管理图标、文字和测量，使表面绘制只持有外框与交互样式。

#[path = "content/entry.rs"]
mod entry;
#[path = "content/glyph.rs"]
mod glyph;
#[path = "content/layout.rs"]
mod layout;
#[path = "content/metrics.rs"]
mod metrics;
#[path = "content/style.rs"]
mod style;
#[path = "content/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_button_content;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use glyph::button_glyph;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_buttons) use metrics::{
    button_content_metrics_from_host, button_label_paint_style_with_preferences,
};
