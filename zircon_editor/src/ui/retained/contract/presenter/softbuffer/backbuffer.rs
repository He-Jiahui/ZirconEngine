use super::super::super::chrome_command_stream::{
    paint_chrome_command_stream_to_frame, repaint_chrome_command_stream_region, ChromeCommandStream,
};
use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::surface_io::damage_pixel_count;
use super::SoftbufferHostPresenter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) enum NativeResizeSnapshotAcquisition {
    Reused,
    CapturedBackbuffer,
    BuiltFallback,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct RepaintOutcome {
    pub(in crate::ui::retained_host::host_contract) damage: Option<FrameRect>,
    pub(in crate::ui::retained_host::host_contract) painted_pixels: u64,
    pub(in crate::ui::retained_host::host_contract) full_paint: bool,
    pub(in crate::ui::retained_host::host_contract) region_paint: bool,
}

pub(in crate::ui::retained_host::host_contract) fn can_region_repaint(
    presenter: &SoftbufferHostPresenter,
) -> bool {
    presenter.backbuffer.as_ref().is_some_and(|frame| {
        frame.width() == presenter.size.0 && frame.height() == presenter.size.1
    })
}

pub(in crate::ui::retained_host::host_contract) fn capture_native_resize_snapshot(
    snapshot: &mut Option<HostRgbaFrame>,
    backbuffer: &mut Option<HostRgbaFrame>,
) -> bool {
    // Freeze the first presented image; scaled transaction frames must never replace its source.
    let captured = if snapshot.is_none() {
        *snapshot = backbuffer.take();
        snapshot.is_some()
    } else {
        false
    };
    *backbuffer = None;
    captured
}

pub(in crate::ui::retained_host::host_contract) fn acquire_native_resize_snapshot(
    snapshot: &mut Option<HostRgbaFrame>,
    backbuffer: &mut Option<HostRgbaFrame>,
    build_fallback: impl FnOnce() -> HostRgbaFrame,
) -> NativeResizeSnapshotAcquisition {
    if snapshot.is_some() {
        *backbuffer = None;
        return NativeResizeSnapshotAcquisition::Reused;
    }
    if capture_native_resize_snapshot(snapshot, backbuffer) {
        return NativeResizeSnapshotAcquisition::CapturedBackbuffer;
    }
    *snapshot = Some(build_fallback());
    NativeResizeSnapshotAcquisition::BuiltFallback
}

pub(in crate::ui::retained_host::host_contract) fn repaint_backbuffer(
    presenter: &mut SoftbufferHostPresenter,
    stream: &ChromeCommandStream,
    size: (u32, u32),
) -> RepaintOutcome {
    if can_region_repaint(presenter) {
        if !stream.is_full_rebuild() {
            if let Some(frame) = presenter.backbuffer.as_mut() {
                if let Some(damage) = repaint_chrome_command_stream_region(frame, stream) {
                    return RepaintOutcome {
                        painted_pixels: damage_pixel_count(&damage, size),
                        damage: Some(damage),
                        full_paint: false,
                        region_paint: true,
                    };
                }
            }
        }
    }

    presenter.backbuffer = Some(paint_chrome_command_stream_to_frame(
        presenter.size.0,
        presenter.size.1,
        stream,
    ));
    RepaintOutcome {
        damage: None,
        painted_pixels: (size.0 as u64) * (size.1 as u64),
        full_paint: true,
        region_paint: false,
    }
}
