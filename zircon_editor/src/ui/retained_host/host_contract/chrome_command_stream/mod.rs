use super::data::{FrameRect, HostWindowPresentationData};
pub(in crate::ui::retained_host::host_contract) use command::{
    ChromeCommand, ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload, ChromeImageUvRect,
};
use extraction::extract_chrome_commands;
pub(in crate::ui::retained_host::host_contract) use replay::{
    paint_chrome_command_stream_to_frame, repaint_chrome_command_stream_region,
};
pub(in crate::ui::retained_host::host_contract) use runtime_draw_list::{
    ui_surface_draw_list_from_owned_stream, ui_surface_draw_list_from_owned_stream_with_generation,
    ui_surface_draw_list_from_stream,
};
pub(in crate::ui::retained_host::host_contract) use stream::ChromeCommandStream;
use stream::clamp_surface_size;

#[cfg(test)]
use extraction::chrome_command_from_recorded_for_test;

mod atlas;
mod command;
mod extraction;
mod replay;
mod runtime_draw_list;
mod stats;
mod stream;

pub(in crate::ui::retained_host::host_contract) fn build_chrome_command_stream(
    presentation: &HostWindowPresentationData,
    surface_size: (u32, u32),
    damage: Option<&FrameRect>,
    include_image_bytes: bool,
) -> ChromeCommandStream {
    let surface_size = clamp_surface_size(surface_size);
    let extraction =
        extract_chrome_commands(presentation, surface_size, damage, include_image_bytes);
    let mut stream = if let Some(damage) = extraction.clipped_damage.clone() {
        ChromeCommandStream::patch(surface_size, damage.clone())
    } else {
        ChromeCommandStream::full_rebuild(surface_size)
    };
    if let Some(damage) = extraction.clipped_damage {
        stream.push_clip(ChromeCommandLayer::Dynamic, 0, damage);
    }
    stream.extend_commands(extraction.commands);
    stream
}

#[cfg(test)]
mod atlas_tests;

#[cfg(test)]
mod tests;
