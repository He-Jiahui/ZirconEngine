//! secondary 专用链的图标按钮边界；按工具栏、面板、侧轨语境选择密度与状态外观。

#[path = "icon_buttons/commands.rs"]
mod commands;
#[path = "icon_buttons/content.rs"]
mod content;
#[path = "icon_buttons/geometry.rs"]
mod geometry;
#[path = "icon_buttons/identity.rs"]
mod identity;
#[path = "icon_buttons/layers.rs"]
mod layers;
#[path = "icon_buttons/metrics.rs"]
mod metrics;
#[path = "icon_buttons/style.rs"]
mod style;
#[path = "icon_buttons/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_icon_button_commands;

#[cfg(test)]
#[path = "icon_buttons_tests/tests/mod.rs"]
mod tests;

#[cfg(test)]
use geometry::{icon_button_paint_rect, icon_glyph_rect};
#[cfg(test)]
use identity::is_workbench_icon_button;
#[cfg(test)]
use style::{icon_button_context, icon_button_style, IconButtonContext};
