use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics, HostMaterialPalette,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSectionTitleMetrics
{
    pub font_size: f32,
    pub line_height: f32,
    pub text_left: f32,
    pub strong_offset_x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSectionTitlePalette
{
    pub text: [u8; 4],
    pub text_muted: [u8; 4],
    pub mesh_text: [u8; 4],
    pub text_disabled: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_title_metrics(
) -> WorkbenchSectionTitleMetrics {
    section_title_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_title_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchSectionTitleMetrics {
    let font_size = (metrics.font_large - metrics.border_width).max(metrics.font_body);
    WorkbenchSectionTitleMetrics {
        font_size,
        line_height: metrics.line_height(font_size),
        text_left: metrics.gap_m,
        strong_offset_x: metrics.border_width * 0.5,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_title_palette(
) -> WorkbenchSectionTitlePalette {
    section_title_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_title_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchSectionTitlePalette {
    WorkbenchSectionTitlePalette {
        text: palette.text,
        text_muted: palette.text_muted,
        mesh_text: palette.text_muted,
        text_disabled: palette.text_disabled,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = section_title_palette();
    if node.disabled {
        palette.text_disabled
    } else if let Some(color) = declared_color(node.label_color) {
        color
    } else if node.control_id == "WorkbenchMeshLabel" {
        palette.mesh_text
    } else if matches!(node.text_tone.as_str(), "muted" | "subtle") {
        palette.text_muted
    } else {
        palette.text
    }
}

fn declared_color(color: crate::ui::retained_host::primitives::Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}
