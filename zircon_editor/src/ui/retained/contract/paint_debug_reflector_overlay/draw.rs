use std::borrow::Borrow;

use super::super::data::{FrameRect, UiDebugOverlayPrimitiveData};
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::{intersect, is_visible_frame, translated};
use super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_text_bars_clipped,
};
use super::colors::{overlay_color, solid_border_color, LABEL_TEXT};
use zircon_runtime_interface::ui::surface::UiDebugOverlayPrimitiveKind;

pub(in crate::ui::retained_host::host_contract) fn draw_debug_reflector_overlay(
    frame: &mut HostRgbaFrame,
    primitives: &[UiDebugOverlayPrimitiveData],
    origin: &FrameRect,
    clip: &FrameRect,
) -> bool {
    draw_debug_reflector_overlay_iter(frame, primitives, origin, clip)
}

pub(in crate::ui::retained_host::host_contract) fn draw_debug_reflector_overlay_iter<I>(
    frame: &mut HostRgbaFrame,
    primitives: I,
    origin: &FrameRect,
    clip: &FrameRect,
) -> bool
where
    I: IntoIterator,
    I::Item: Borrow<UiDebugOverlayPrimitiveData>,
{
    if !is_visible_frame(origin) || !is_visible_frame(clip) {
        return false;
    }

    let mut painted = false;
    for primitive in primitives {
        painted |= draw_overlay_primitive(frame, primitive.borrow(), origin, clip);
    }
    painted
}

fn draw_overlay_primitive(
    frame: &mut HostRgbaFrame,
    primitive: &UiDebugOverlayPrimitiveData,
    origin: &FrameRect,
    clip: &FrameRect,
) -> bool {
    let rect = translated(&primitive.frame, origin.x, origin.y);
    let Some(visible) = intersect(&rect, clip) else {
        return false;
    };

    let color = overlay_color(primitive.kind);
    match primitive.kind {
        UiDebugOverlayPrimitiveKind::SelectedFrame
        | UiDebugOverlayPrimitiveKind::ClipFrame
        | UiDebugOverlayPrimitiveKind::Wireframe
        | UiDebugOverlayPrimitiveKind::HitPath
        | UiDebugOverlayPrimitiveKind::RejectedBounds
        | UiDebugOverlayPrimitiveKind::TextBaseline => {
            draw_border_clipped(frame, rect.clone(), Some(clip), color);
        }
        UiDebugOverlayPrimitiveKind::HitCell
        | UiDebugOverlayPrimitiveKind::OverdrawCell
        | UiDebugOverlayPrimitiveKind::MaterialBatchBounds
        | UiDebugOverlayPrimitiveKind::TextGlyphBounds
        | UiDebugOverlayPrimitiveKind::ResourceAtlas
        | UiDebugOverlayPrimitiveKind::DamageRegion => {
            draw_rect_clipped(frame, rect.clone(), Some(clip), color);
            draw_border_clipped(frame, rect.clone(), Some(clip), solid_border_color(color));
        }
    }

    if !primitive.label.trim().is_empty() {
        draw_text_bars_clipped(
            frame,
            visible.x + 3.0,
            visible.y + 3.0,
            primitive.label.as_str(),
            Some(&visible),
            LABEL_TEXT,
        );
    }
    true
}
