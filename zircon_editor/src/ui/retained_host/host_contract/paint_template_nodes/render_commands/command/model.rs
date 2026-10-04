use zircon_runtime_interface::ui::surface::{UiRenderFrameCommandRef, UiTextRunPaintStyle};

use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_text::HostTextLayoutPolicy;
use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};
use super::super::super::visual_assets::HostPaintImagePixels;
use super::kind::HostPaintCommandKind;

#[derive(Clone)]
/// 宿主绘制命令同时携带可见框、裁剪、来源键与层级；后续 CPU 绘制和录制回放共享这份契约。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct HostPaintCommand {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) owner:
        Option<PaintNodeIdentity>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) box_shadow:
        Option<PaintBoxShadowGeometry>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) kind:
        HostPaintCommandKind,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) frame: FrameRect,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) clip_frame:
        Option<FrameRect>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) z_index: i32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) source_surface_frame:
        Option<std::sync::Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) source_render_command_ref:
        Option<UiRenderFrameCommandRef>,
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

#[derive(Clone, Copy, Debug)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct PaintBoxShadowGeometry
{
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) offset_x: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) offset_y: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) blur_radius: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) spread_radius: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) radius: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) inset: bool,
}

/// Authored source identity attached to every command emitted for one semantic
/// node. The retained host's generated ids stay separate from this provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct PaintNodeIdentity {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) node_id: String,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) parent_node_id:
        Option<String>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) source_path: String,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) source_node_id: String,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) instance_path: String,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) parent_source_path:
        Option<String>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) parent_source_node_id:
        Option<String>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) parent_instance_path:
        Option<String>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) control_id:
        Option<String>,
}

impl HostPaintCommand {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn with_box_shadow(
        mut self,
        offset_x: f32,
        offset_y: f32,
        blur_radius: f32,
        spread_radius: f32,
        radius: f32,
        inset: bool,
    ) -> Self {
        self.box_shadow = Some(PaintBoxShadowGeometry {
            offset_x,
            offset_y,
            blur_radius,
            spread_radius,
            radius,
            inset,
        });
        self
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn set_owner(
        &mut self,
        owner: PaintNodeIdentity,
    ) {
        self.owner = Some(owner);
    }

    pub(super) fn fallback_text_metrics_from_host(metrics: HostControlMetrics) -> (f32, f32) {
        (metrics.font_body, metrics.line_height(metrics.font_body))
    }

    pub(super) fn fallback_text_metrics() -> (f32, f32) {
        Self::fallback_text_metrics_from_host(current_host_metrics())
    }
}

#[cfg(test)]
#[path = "tests/model.rs"]
mod tests;
