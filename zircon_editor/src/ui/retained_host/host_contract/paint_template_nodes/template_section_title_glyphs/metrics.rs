use super::super::super::paint_theme::{HostControlMetrics, current_host_metrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSectionTitleGlyphMetrics
{
    pub icon_size: f32,
    pub icon_gap: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_title_glyph_metrics()
-> WorkbenchSectionTitleGlyphMetrics {
    section_title_glyph_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_title_glyph_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchSectionTitleGlyphMetrics {
    WorkbenchSectionTitleGlyphMetrics {
        icon_size: metrics.font_body,
        icon_gap: metrics.gap_m,
    }
}
