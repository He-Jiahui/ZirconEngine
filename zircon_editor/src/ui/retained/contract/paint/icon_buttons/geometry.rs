//! 图标按钮外框、图标槽和完整容纳检查；声明尺寸已由窗口挂载转换为物理像素，未声明时保留密度门槛。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::icon_button_glyph_metrics;
use super::style::IconButtonContext;
use crate::ui::retained_host::host_contract::paint_geometry::corner_radius_for_frame;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: rect.width,
        height: rect.height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_icon_button_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn frame_is_within(
    inner: &FrameRect,
    outer: &FrameRect,
) -> bool {
    if !has_paintable_icon_button_extent(inner) || !has_paintable_icon_button_extent(outer) {
        return false;
    }

    let inner_right = inner.x + inner.width;
    let inner_bottom = inner.y + inner.height;
    let outer_right = outer.x + outer.width;
    let outer_bottom = outer.y + outer.height;
    inner_right.is_finite()
        && inner_bottom.is_finite()
        && outer_right.is_finite()
        && outer_bottom.is_finite()
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner_right <= outer_right
        && inner_bottom <= outer_bottom
}

fn authored_icon_glyph_size(
    node: &TemplatePaneNodeData,
    context: IconButtonContext,
) -> Option<f32> {
    (context != IconButtonContext::TabClose
        && node.layout_icon_size.is_finite()
        && node.layout_icon_size > 0.0)
        .then_some(node.layout_icon_size)
}

/// layout_icon_size 已为物理尺寸；旧 value_number 只保留尺寸分支，不能降低默认密度门槛。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_glyph_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    context: IconButtonContext,
) -> FrameRect {
    let max_size = rect.width.min(rect.height).max(0.0);
    let metrics = icon_button_glyph_metrics();
    let default_size = metrics.glyph_size_for_context(context);
    let size = if context == IconButtonContext::TabClose {
        default_size.min(max_size)
    } else if let Some(size) = authored_icon_glyph_size(node, context) {
        size.min(metrics.max_glyph_size(max_size))
    } else if node.value_number.is_finite() && node.value_number > 0.0 {
        node.value_number.min(metrics.max_glyph_size(max_size))
    } else {
        default_size.min(metrics.max_glyph_size(max_size))
    };
    FrameRect {
        x: rect.x + (rect.width - size).max(0.0) * 0.5,
        y: rect.y + (rect.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_surface_radius(
    rect: &FrameRect,
    requested_radius: f32,
) -> f32 {
    corner_radius_for_frame(rect, requested_radius)
}

/// 声明尺寸须完整容纳；未声明时仍要求默认密度尺寸，窄控件保留表面但省略图标。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_glyph_is_paintable(
    node: &TemplatePaneNodeData,
    glyph: &FrameRect,
    button: &FrameRect,
    context: IconButtonContext,
) -> bool {
    let required_size = authored_icon_glyph_size(node, context)
        .unwrap_or_else(|| icon_button_glyph_metrics().glyph_size_for_context(context));
    required_size.is_finite()
        && required_size > 0.0
        && frame_is_within(glyph, button)
        && glyph.width >= required_size
        && glyph.height >= required_size
}
