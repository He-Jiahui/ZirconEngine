use super::super::data::{FrameRect, UiDebugOverlayPrimitiveData};
use super::super::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::surface::UiDebugOverlayPrimitiveKind;

use super::draw_debug_reflector_overlay;

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
