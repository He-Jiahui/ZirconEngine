use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

use super::style::IconButtonContext;

const MIN_GLYPH_SIZE: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WorkbenchIconButtonMetrics {
    toolbar_glyph_size: f32,
    panel_glyph_size: f32,
    rail_glyph_size: f32,
    min_glyph_inset: f32,
}

impl WorkbenchIconButtonMetrics {
    pub(super) fn glyph_size_for_context(&self, context: IconButtonContext) -> f32 {
        match context {
            IconButtonContext::Rail => self.rail_glyph_size,
            IconButtonContext::Toolbar => self.toolbar_glyph_size,
            IconButtonContext::Panel => self.panel_glyph_size,
        }
    }

    pub(super) fn max_glyph_size(&self, max_side: f32) -> f32 {
        (max_side - self.min_glyph_inset).max(MIN_GLYPH_SIZE)
    }
}

pub(super) fn icon_button_glyph_metrics() -> WorkbenchIconButtonMetrics {
    icon_button_glyph_metrics_from_host(current_host_metrics())
}

fn icon_button_glyph_metrics_from_host(metrics: HostControlMetrics) -> WorkbenchIconButtonMetrics {
    WorkbenchIconButtonMetrics {
        toolbar_glyph_size: glyph_size(
            metrics.font_large + metrics.button_icon_gap - metrics.border_width,
        ),
        panel_glyph_size: glyph_size(metrics.font_large + metrics.border_width * 2.0),
        rail_glyph_size: glyph_size(
            metrics.font_large + metrics.button_icon_gap + metrics.border_width * 3.0,
        ),
        min_glyph_inset: (metrics.button_icon_gap - metrics.border_width).max(0.0),
    }
}

fn glyph_size(size: f32) -> f32 {
    size.max(MIN_GLYPH_SIZE)
}
