//! 图标内容与稳定表面分离；按压反馈只影响内容位置和颜色。

#[path = "content/style.rs"]
mod style;

pub(super) use style::icon_button_content_style;
