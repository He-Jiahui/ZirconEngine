//! 普通编辑器按钮的专用绘制边界，由 template_nodes 的 primary 链接管；图标按钮等独立家族交给后续绘制器。

#[path = "buttons/commands.rs"]
mod commands;
#[path = "buttons/content.rs"]
mod content;
#[path = "buttons/geometry.rs"]
mod geometry;
#[path = "buttons/identity.rs"]
mod identity;
#[path = "buttons/layers.rs"]
mod layers;
#[path = "buttons/metrics.rs"]
mod metrics;
#[path = "buttons/style.rs"]
mod style;
#[path = "buttons/surface.rs"]
mod surface;
#[path = "buttons/surface_indicator.rs"]
mod surface_indicator;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_button_commands;

#[cfg(test)]
use super::template_button_glyphs::button_icon_size_from_host;
#[cfg(test)]
use content::{button_content_metrics_from_host, button_glyph};
#[cfg(test)]
use geometry::button_radius;
#[cfg(test)]
use geometry::{add_component_button_offset_y_from_host, button_paint_rect};
#[cfg(test)]
use identity::{button_kind, is_workbench_button};
#[cfg(test)]
use metrics::button_geometry_metrics_from_host;
#[cfg(test)]
use style::button_opacity;
#[cfg(test)]
use style::button_style;

#[cfg(test)]
#[path = "buttons_tests/tests/mod.rs"]
mod tests;
