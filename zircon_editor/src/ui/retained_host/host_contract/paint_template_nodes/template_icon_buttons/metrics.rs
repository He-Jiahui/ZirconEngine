use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

use super::style::IconButtonContext;

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
        (max_side - self.min_glyph_inset).max(0.0)
    }
}

pub(super) fn icon_button_glyph_metrics() -> WorkbenchIconButtonMetrics {
    icon_button_glyph_metrics_from_host(current_host_metrics())
}

fn icon_button_glyph_metrics_from_host(metrics: HostControlMetrics) -> WorkbenchIconButtonMetrics {
    // Slate icon classes are density slots: changing typography must not resize
    // glyphs independently from the 28 px row that owns their hit targets.
    WorkbenchIconButtonMetrics {
        toolbar_glyph_size: glyph_size(metrics.row_height - metrics.gap_m),
        panel_glyph_size: glyph_size(metrics.row_height - metrics.gap_l),
        rail_glyph_size: glyph_size(metrics.row_height - metrics.gap_s),
        min_glyph_inset: (metrics.button_icon_gap - metrics.border_width).max(0.0),
    }
}

fn glyph_size(size: f32) -> f32 {
    size.max(0.0)
}
