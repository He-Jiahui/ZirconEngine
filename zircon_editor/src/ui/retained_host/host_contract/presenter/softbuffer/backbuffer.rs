use super::super::super::chrome_command_stream::{
    paint_chrome_command_stream_to_frame, repaint_chrome_command_stream_region, ChromeCommandStream,
};
use super::super::super::data::FrameRect;
use super::surface_io::damage_pixel_count;
use super::SoftbufferHostPresenter;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RepaintOutcome {
    pub(super) damage: Option<FrameRect>,
    pub(super) painted_pixels: u64,
    pub(super) full_paint: bool,
    pub(super) region_paint: bool,
}

pub(super) fn can_region_repaint(presenter: &SoftbufferHostPresenter) -> bool {
    presenter.backbuffer.as_ref().is_some_and(|frame| {
        frame.width() == presenter.size.0 && frame.height() == presenter.size.1
    })
}

pub(super) fn repaint_backbuffer(
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
