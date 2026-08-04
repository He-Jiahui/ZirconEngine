use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_text::HostTextLayoutPolicy;
use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};
use super::super::super::visual_assets::HostPaintImagePixels;
use super::kind::HostPaintCommandKind;

#[derive(Clone)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct HostPaintCommand {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) kind:
        HostPaintCommandKind,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) frame: FrameRect,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) clip_frame:
        Option<FrameRect>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) z_index: i32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) background_color:
        Option<[u8; 4]>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) foreground_color:
        Option<[u8; 4]>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) border_color:
        Option<[u8; 4]>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) border_width: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) corner_radius: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) text: Option<String>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) font_size: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) line_height: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) text_style:
        UiTextRunPaintStyle,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) text_layout_policy:
        HostTextLayoutPolicy,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) image_key: Option<String>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) image_pixels:
        Option<HostPaintImagePixels>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) opacity: f32,
}

impl HostPaintCommand {
    pub(super) fn fallback_text_metrics_from_host(metrics: HostControlMetrics) -> (f32, f32) {
        (metrics.font_body, metrics.line_height(metrics.font_body))
    }

    pub(super) fn fallback_text_metrics() -> (f32, f32) {
        Self::fallback_text_metrics_from_host(current_host_metrics())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn fallback_text_metrics_project_from_host_control_metrics() {
        assert_eq!(
            HostPaintCommand::fallback_text_metrics_from_host(METRICS),
            (METRICS.font_body, METRICS.line_height(METRICS.font_body))
        );
    }
}
