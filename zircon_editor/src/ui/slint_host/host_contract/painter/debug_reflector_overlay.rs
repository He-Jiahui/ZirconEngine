use super::super::data::{FrameRect, UiDebugOverlayPrimitiveData};
use super::frame::HostRgbaFrame;
use super::geometry::{intersect, is_visible_frame, translated};
use super::primitives::{draw_border_clipped, draw_rect_clipped, draw_text_bars_clipped};
use zircon_runtime_interface::ui::surface::UiDebugOverlayPrimitiveKind;

const SELECTED_FRAME: [u8; 4] = [92, 156, 255, 255];
const CLIP_FRAME: [u8; 4] = [148, 117, 255, 220];
const WIREFRAME: [u8; 4] = [64, 196, 255, 180];
const HIT_CELL: [u8; 4] = [64, 220, 142, 96];
const HIT_PATH: [u8; 4] = [64, 220, 142, 220];
const REJECTED_BOUNDS: [u8; 4] = [190, 198, 214, 120];
const OVERDRAW: [u8; 4] = [255, 167, 38, 104];
const MATERIAL_BATCH: [u8; 4] = [64, 188, 255, 96];
const TEXT_DEBUG: [u8; 4] = [186, 104, 200, 176];
const RESOURCE_ATLAS: [u8; 4] = [38, 166, 154, 148];
const DAMAGE_REGION: [u8; 4] = [255, 88, 112, 128];
const LABEL_TEXT: [u8; 4] = [230, 236, 246, 230];

pub(in crate::ui::slint_host::host_contract) fn draw_debug_reflector_overlay(
    frame: &mut HostRgbaFrame,
    primitives: &[UiDebugOverlayPrimitiveData],
    origin: &FrameRect,
    clip: &FrameRect,
) -> bool {
    if primitives.is_empty() || !is_visible_frame(origin) || !is_visible_frame(clip) {
        return false;
    }

    let mut painted = false;
    for primitive in primitives {
        painted |= draw_overlay_primitive(frame, primitive, origin, clip);
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

fn overlay_color(kind: UiDebugOverlayPrimitiveKind) -> [u8; 4] {
    match kind {
        UiDebugOverlayPrimitiveKind::SelectedFrame => SELECTED_FRAME,
        UiDebugOverlayPrimitiveKind::ClipFrame => CLIP_FRAME,
        UiDebugOverlayPrimitiveKind::Wireframe => WIREFRAME,
        UiDebugOverlayPrimitiveKind::HitCell => HIT_CELL,
        UiDebugOverlayPrimitiveKind::HitPath => HIT_PATH,
        UiDebugOverlayPrimitiveKind::RejectedBounds => REJECTED_BOUNDS,
        UiDebugOverlayPrimitiveKind::OverdrawCell => OVERDRAW,
        UiDebugOverlayPrimitiveKind::MaterialBatchBounds => MATERIAL_BATCH,
        UiDebugOverlayPrimitiveKind::TextGlyphBounds
        | UiDebugOverlayPrimitiveKind::TextBaseline => TEXT_DEBUG,
        UiDebugOverlayPrimitiveKind::ResourceAtlas => RESOURCE_ATLAS,
        UiDebugOverlayPrimitiveKind::DamageRegion => DAMAGE_REGION,
    }
}

fn solid_border_color(mut color: [u8; 4]) -> [u8; 4] {
    color[3] = color[3].saturating_add(80).max(180);
    color
}

#[cfg(test)]
mod tests {
    use slint::SharedString;

    use super::*;

    #[test]
    fn debug_reflector_overlay_draws_snapshot_primitive_inside_clip() {
        let mut frame = HostRgbaFrame::filled(80, 60, [0, 0, 0, 255]);
        let primitive = UiDebugOverlayPrimitiveData {
            kind: UiDebugOverlayPrimitiveKind::DamageRegion,
            frame: FrameRect {
                x: 10.0,
                y: 12.0,
                width: 24.0,
                height: 18.0,
            },
            label: SharedString::from("damage"),
            ..UiDebugOverlayPrimitiveData::default()
        };

        let painted = draw_debug_reflector_overlay(
            &mut frame,
            &[primitive],
            &FrameRect {
                x: 4.0,
                y: 5.0,
                width: 70.0,
                height: 50.0,
            },
            &FrameRect {
                x: 4.0,
                y: 5.0,
                width: 70.0,
                height: 50.0,
            },
        );

        assert!(painted);
        assert_ne!(pixel(&frame, 15, 18), [0, 0, 0, 255]);
    }

    fn pixel(frame: &HostRgbaFrame, x: u32, y: u32) -> [u8; 4] {
        let offset = ((y as usize * frame.width() as usize) + x as usize) * 4;
        let bytes = frame.as_bytes();
        [
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]
    }
}
